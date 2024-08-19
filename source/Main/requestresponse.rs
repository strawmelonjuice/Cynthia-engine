/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */
use crate::tell::CynthiaColors;
use actix_web::web::Data;
use actix_web::{get, post, HttpRequest, HttpResponse, Responder};
use log::{debug, trace, warn};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::CynthiaConfig;
use crate::externalpluginservers::{contact_eps, EPSRequestBody};
use crate::files::CynthiaCacheExtraction;
use crate::renders::render_from_pgid;
use crate::LockCallback;
use crate::{renders, ServerContext};

fn urlspace() -> (usize, usize) {
    let fullwidth = termsize::get().unwrap().cols as usize;

    // let w_a = if fullwidth < 210 {
    //     fullwidth
    //         .checked_div(10)
    //         .unwrap_or(10)
    //         .checked_mul(4)
    //         .unwrap_or(30)
    // } else {
    //     140
    // };

    let w_a: usize = 30;
    let w_s = w_a.checked_sub(3).unwrap_or(27);
    // let w_s = 0;

    debug!("Full widht is {fullwidth} cols. Any request urls will be printed in a space of {} characters, with the actual url being {} characters long.", w_a, w_s);
    (w_s, w_a)
    // (53, 55)
}

#[get("/{a:.*}")]
#[doc = r"Serves pages included in CynthiaConfig, or a default page if not found."]
pub(crate) async fn serve(
    server_context_mutex: Data<Arc<Mutex<ServerContext>>>,
    req: HttpRequest,
) -> impl Responder {
    let (w_s, w_a) = urlspace();
    // We can't lock the mutex here because it wouldn't be usable by EPS, so we need to use a callback.
    // let mut server_context: MutexGuard<ServerContext> = server_context_mutex.lock().await;
    let config_clone = server_context_mutex
        .lock_callback(|a| {
            a.request_count += 1;
            a.config.clone()
        })
        .await;

    let page_uri = if req.uri() == "" {
        "root".to_string()
    } else {
        req.uri().to_string()
    };
    let page_id = page_uri.trim_start_matches('/');
    let headers = {
        // Transform it into makeshift JSON!
        let json_kinda = format!("{:?}", &req.headers().iter().collect::<Vec<_>>())
            .replace("(\"", "[\"")
            .replace("\")", "\"]");
        // And then parse it back into an object.
        serde_json::from_str(&json_kinda).unwrap_or_default()
    };
    trace!("{}", serde_json::to_string(&headers).unwrap());
    let pluginsresponse = contact_eps(
        server_context_mutex.clone(),
        EPSRequestBody::WebRequest {
            uri: page_uri.clone(),
            headers,
            method: "get".to_string(),
        },
    )
    .await;
    match pluginsresponse {
        crate::externalpluginservers::EPSResponseBody::WebResponse {
            append_headers,
            response_body,
        } => {
            let coninfo = req.connection_info();
            let ip = coninfo.realip_remote_addr().unwrap_or("<unknown IP>");
            config_clone.tell(format!(
                "{}\t{:>w_s$.w_a$}\t\t\t{}\t{}",
                "GET:200".color_ok_green(),
                {
                    let uri = req.uri().to_string();
                    if uri == *"" {
                        "/".to_string()
                    } else {
                        uri
                    }
                },
                ip.color_lightblue(),
                "extern".color_pink()
            ));
            let mut response = HttpResponse::build(actix_web::http::StatusCode::OK);
            for (k, v) in append_headers {
                response.append_header((k, v));
            }
            return response.body(response_body);
        }
        crate::externalpluginservers::EPSResponseBody::NoneOk => {}
        crate::externalpluginservers::EPSResponseBody::Disabled => {}
        _ => return HttpResponse::InternalServerError().body("Internal server error."),
    }
    let s = server_context_mutex
        .lock_callback(|servercontext| renders::check_pgid(page_id.to_string(), servercontext))
        .await;
    match s {
        renders::PGIDCheckResponse::Ok => {
            let from_cache: bool;
            let cache_result = server_context_mutex
                .lock_callback(|servercontext| servercontext.get_cache(page_id, 0))
                .await;
            let page = match cache_result {
                Some(c) => {
                    from_cache = true;
                    c
                }
                None => {
                    from_cache = false;
                    // Now that we're past the EPS, we can lock the mutex for this scope.
                    let page =
                        render_from_pgid(page_id.parse().unwrap(), server_context_mutex.clone())
                            .await;
                    let mut server_context = server_context_mutex.lock().await;
                    server_context
                        .store_cache(
                            page_id,
                            page.clone().unwrap().as_bytes(),
                            config_clone.clone().cache.lifetimes.served,
                        )
                        .unwrap();
                    server_context
                        .get_cache(page_id, config_clone.clone().cache.lifetimes.served)
                        .unwrap_or(CynthiaCacheExtraction(page.unwrap().as_bytes().to_vec(), 0))
                }
            };

            let coninfo = req.connection_info();
            let ip = coninfo.realip_remote_addr().unwrap_or("<unknown IP>");
            config_clone.tell(format!(
                "{}\t{:>w_s$.w_a$}\t\t\t{}\t{}",
                "GET:200".color_ok_green(),
                {
                    let uri = req.uri().to_string();
                    if uri == *"" {
                        "/".to_string()
                    } else {
                        uri
                    }
                },
                ip.color_lightblue(),
                {
                    if from_cache {
                        "cache".color_green()
                    } else {
                        "generated".color_yellow()
                    }
                }
            ));
            HttpResponse::Ok()
                .append_header(("Content-Type", "text/html; charset=utf-8"))
                .body(page.0)
        }
        renders::PGIDCheckResponse::Error => {
            HttpResponse::InternalServerError().body("Internal server error.")
        }
        renders::PGIDCheckResponse::NotFound => {
            let coninfo = req.connection_info().clone();
            let ip = coninfo.realip_remote_addr().unwrap_or("<unknown IP>");
            warn!(
                "{}\t{:>w_s$.w_a$}\t\t\t{}\t{}",
                "GET:404".color_error_red(),
                {
                    let uri = req.uri().to_string();
                    if uri == *"" {
                        "/".to_string()
                    } else {
                        uri
                    }
                },
                ip.color_lightblue(),
                "not found".color_red()
            );

            HttpResponse::NotFound()
                .append_header(("Content-Type", "text/html; charset=utf-8"))
                .body(
                    render_from_pgid(
                        config_clone.site.notfound_page.clone(),
                        server_context_mutex.clone(),
                    )
                    .await
                    .unwrap(),
                )
        }
    }
}

#[get("/assets/{reqfile:.*}")]
pub(crate) async fn assets_with_cache(
    server_context_mutex: Data<Arc<Mutex<ServerContext>>>,
    req: HttpRequest,
) -> impl Responder {
    let (w_s, w_a) = urlspace();
    let path = req.match_info().get("reqfile").unwrap();
    let cacheresulr = server_context_mutex
        .lock_callback(|servercontext| servercontext.get_cache(path, 0))
        .await;
    match cacheresulr {
        None => {
            let config_clone = server_context_mutex
                .lock_callback(|a| {
                    a.request_count += 1;
                    a.config.clone()
                })
                .await;
            let filepath: PathBuf = std::env::current_dir()
                .unwrap()
                .canonicalize()
                .unwrap()
                .join("cynthiaFiles/assets/")
                .join(path);
            debug!("Requested asset: {:?}", filepath);
            if filepath.exists() && filepath.is_file() {
                let contents: Vec<u8> = std::fs::read(filepath).unwrap();
                let mut server_context = server_context_mutex.lock().await;
                server_context
                    .store_cache(path, &contents, config_clone.cache.lifetimes.assets)
                    .unwrap();
                let coninfo = req.connection_info();
                let ip = coninfo.realip_remote_addr().unwrap_or("<unknown IP>");
                server_context.tell(format!(
                    "{}\t{:>w_s$.w_a$}\t\t\t{}\t{}",
                    "GET:200".color_ok_green(),
                    {
                        let uri = req.uri().to_string();
                        if uri == *"" {
                            "/".to_string()
                        } else {
                            uri
                        }
                    },
                    ip.color_lightblue(),
                    "filesystem".color_lilac()
                ));
                HttpResponse::Ok()
                    .append_header(("Content-Type", "text/html; charset=utf-8"))
                    .body(contents)
            } else {
                let coninfo = req.connection_info();
                let ip = coninfo.realip_remote_addr().unwrap_or("<unknown IP>");
                config_clone.tell(format!(
                    "{}\t{:>w_s$.w_a$}\t\t\t{}\t{}",
                    "GET:404".color_error_red(),
                    {
                        let uri = req.uri().to_string();
                        if uri == *"" {
                            "/".to_string()
                        } else {
                            uri
                        }
                    },
                    ip.color_lightblue(),
                    "not found".color_red()
                ));
                HttpResponse::NotFound().body("404 Not Found")
            }
        }
        Some(c) => {
            let config_clone = server_context_mutex
                .lock_callback(|a| {
                    a.request_count += 1;
                    a.config.clone()
                })
                .await;
            let coninfo = req.connection_info();
            let ip = coninfo.realip_remote_addr().unwrap_or("<unknown IP>");
            config_clone.tell(format!(
                "{}\t{:>w_s$.w_a$}\t\t\t{}\t{}",
                "GET:200".color_ok_green(),
                {
                    let uri = req.uri().to_string();
                    if uri == *"" {
                        "/".to_string()
                    } else {
                        uri
                    }
                },
                ip.color_lightblue(),
                "cache".color_green()
            ));
            HttpResponse::Ok()
                .append_header(("Content-Type", "text/html; charset=utf-8"))
                .body(c.0)
        }
    }
}

/// Cynthia doesn't respond to POST requests, but it's plugins might.
/// Support for form data is planned but not yet implemented.
#[post("/{a:.*}")]
pub(crate) async fn post(
    server_context_mutex: Data<Arc<Mutex<ServerContext>>>,
    req: HttpRequest,
) -> impl Responder {
    let (w_s, w_a) = urlspace();
    // We can't lock the mutex here because it wouldn't be usable by EPS, so we need to use a callback.
    // let mut server_context: MutexGuard<ServerContext> = server_context_mutex.lock().await;
    let config_clone = server_context_mutex
        .lock_callback(|a| {
            a.request_count += 1;
            a.config.clone()
        })
        .await;

    let page_uri = if req.uri() == "" {
        "root".to_string()
    } else {
        req.uri().to_string()
    };
    let headers = {
        // Transform it into makeshift JSON!
        let json_kinda = format!("{:?}", &req.headers().iter().collect::<Vec<_>>())
            .replace("(\"", "[\"")
            .replace("\")", "\"]");
        // And then parse it back into an object.
        serde_json::from_str(&json_kinda).unwrap_or_default()
    };
    trace!("{}", serde_json::to_string(&headers).unwrap());
    let pluginsresponse = contact_eps(
        server_context_mutex.clone(),
        EPSRequestBody::WebRequest {
            uri: page_uri.clone(),
            headers,
            method: "get".to_string(),
        },
    )
    .await;
    match pluginsresponse {
        crate::externalpluginservers::EPSResponseBody::WebResponse {
            append_headers,
            response_body,
        } => {
            let coninfo = req.connection_info();
            let ip = coninfo.realip_remote_addr().unwrap_or("<unknown IP>");
            config_clone.tell(format!(
                "{}\t{:>w_s$.w_a$}\t\t\t{}\t{}",
                "POST:200".color_ok_green(),
                {
                    let uri = req.uri().to_string();
                    if uri == *"" {
                        "/".to_string()
                    } else {
                        uri
                    }
                },
                ip.color_lightblue(),
                "extern".color_pink()
            ));
            let mut response = HttpResponse::build(actix_web::http::StatusCode::OK);
            for (k, v) in append_headers {
                response.append_header((k, v));
            }
            return response.body(response_body);
        }
        crate::externalpluginservers::EPSResponseBody::NoneOk => {}
        crate::externalpluginservers::EPSResponseBody::Disabled => {}
        _ => return HttpResponse::InternalServerError().body("Internal server error."),
    }
    HttpResponse::NoContent().finish()
}
