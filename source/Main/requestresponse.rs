/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */
use std::sync::Arc;

use actix_web::web::Data;
use actix_web::{get, HttpRequest, HttpResponse, Responder};
use colored::Colorize;
use log::warn;
use tokio::sync::Mutex;

use crate::externalpluginservers::{contact_eps, EPSRequestBody};
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
    warn!("EPSRequest::WebRequest doesn't have headers implemented yet. Thread carefully.");

    let pluginsresponse = contact_eps(
        server_context_mutex.clone(),
        EPSRequestBody::WebRequest {
            page_id: page_id.to_string(),
            headers: vec![], // No clue how to get these tbh
            method: "get".to_string(),
        },
    )
    .await;
    match pluginsresponse {
        crate::externalpluginservers::EPSResponseBody::NoneOk => {}
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
                    server_context.store_cache(page_id, page.unwrap().as_bytes(), 15);
                    server_context.get_cache(page_id, 0).unwrap()
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
            HttpResponse::Ok().body(page.0)
        }
        renders::PGIDCheckResponse::Error => {
            HttpResponse::InternalServerError().body("Internal server error.")
        }
        renders::PGIDCheckResponse::NotFound => {
            let coninfo = req.connection_info().clone();
            let ip = coninfo.realip_remote_addr().unwrap_or("<unknown IP>");
            warn!(
                "{}\t{:>25.27}\t\t{}",
                "Request/404".bright_red(),
                req.path(),
                ip
            );

            HttpResponse::NotFound().body(
                render_from_pgid(
                    config_clone.pages.notfound_page.clone(),
                    server_context_mutex.clone(),
                )
                .await
                .unwrap(),
            )
        }
    }
}
