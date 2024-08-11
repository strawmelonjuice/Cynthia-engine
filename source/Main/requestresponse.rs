/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */
use actix_files::NamedFile;
use actix_web::web::Data;
use actix_web::{get, HttpRequest, HttpResponse, Responder};
use colored::Colorize;
use log::{debug, trace, warn};
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::CynthiaConfig;
use crate::externalpluginservers::{contact_eps, EPSRequestBody};
use crate::files::CynthiaCacheExtraction;
use crate::renders::render_from_pgid;
use crate::LockCallback;
use crate::{renders, ServerContext};

#[get("/{a:.*}")]
#[doc = r"Serves pages included in CynthiaConfig, or a default page if not found."]
pub(crate) async fn serve(
    server_context_mutex: Data<Arc<Mutex<ServerContext>>>,
    req: HttpRequest,
) -> impl Responder {
    // We can't lock the mutex here because it wouldn't be usable by EPS, so we need to use a callback.
    // let mut server_context: MutexGuard<ServerContext> = server_context_mutex.lock().await;
    let config_clone = server_context_mutex
        .lock_callback(|a| {
            a.request_count += 1;
            a.config.clone()
        })
        .await;

    let page_id = req.match_info().get("a").unwrap_or("root");
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
            page_id: page_id.to_string(),
            headers,
            method: "get".to_string(),
        },
    )
    .await;
    match pluginsresponse {
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
                "{}\t{:>25.27}\t\t{}\t{}",
                "Request/200".bright_green(),
                req.path(),
                ip,
                {
                    if from_cache {
                        "from cache".green()
                    } else {
                        "generated".yellow()
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
                "{}\t{:>25.27}\t\t{}\t{}",
                "Request/404".bright_red(),
                req.path(),
                ip,
                "not found".red()
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
            if filepath.exists() {
                let contents: Vec<u8> = std::fs::read(filepath).unwrap();
                let mut server_context = server_context_mutex.lock().await;
                server_context
                    .store_cache(path, &contents, config_clone.cache.lifetimes.assets)
                    .unwrap();
                let coninfo = req.connection_info();
                let ip = coninfo.realip_remote_addr().unwrap_or("<unknown IP>");
                server_context.tell(format!(
                    "{}\t{:>25.27}\t\t{}\t{}",
                    "Request/200".bright_green(),
                    req.path().blue(),
                    ip,
                    "filesystem".blue()
                ));
                HttpResponse::Ok()
                    .append_header(("Content-Type", "text/html; charset=utf-8"))
                    .body(contents)
            } else {
                let coninfo = req.connection_info();
                let ip = coninfo.realip_remote_addr().unwrap_or("<unknown IP>");
                config_clone.tell(format!(
                    "{}\t{:>25.27}\t\t{}\t{}",
                    "Request/404".bright_red(),
                    req.path().blue(),
                    ip,
                    "not found".red()
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
                "{}\t{:>25.27}\t\t{}\t{}",
                "Request/200".bright_green(),
                req.path().blue(),
                ip,
                "from cache".green()
            ));
            HttpResponse::Ok()
                .append_header(("Content-Type", "text/html; charset=utf-8"))
                .body(c.0)
        }
    }
}
