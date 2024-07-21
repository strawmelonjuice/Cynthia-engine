/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */

use crate::{generations, ServerContext};
use actix_web::web::Data;
use actix_web::{get, HttpRequest, HttpResponse, Responder};
use colored::Colorize;
use log::{warn};
use tokio::sync::{Mutex, MutexGuard};
use crate::generations::generate_from_pgid;
#[get("/{a:.*}")]
#[doc = r"Serves pages included in CynthiaConfig, or a default page if not found."]
pub(crate) async fn serve(server_context_mutex: Data<Mutex<ServerContext>>, req: HttpRequest)  -> impl Responder {
    let mut server_context: MutexGuard<ServerContext> = server_context_mutex.lock().await;
    let page_id = "test";
    match generations::check_pgid(page_id.to_string(), &server_context) {
        generations::EmptyGenerationResponse::Ok => {
            let from_cache: bool;
            let page = match server_context.get_cache(page_id, 0) {
                Some(c) => {
                    from_cache = true;
                    c
                }
                None => {
                    from_cache = false;
                    let page = generate_from_pgid(page_id.parse().unwrap());
                    server_context.store_cache(page_id, page.as_bytes(), 15);
                    server_context.get_cache(page_id, 0).unwrap()
                }
            };
            let coninfo = req.connection_info();
            let ip = coninfo.realip_remote_addr().unwrap_or("<unknown IP>");
            server_context.tell(format!(
                "Request/200\t{:>45.47}\t\t{}\t{}",
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
        generations::EmptyGenerationResponse::Error => {
            return HttpResponse::InternalServerError().body("Internal server error.");
        }
        generations::EmptyGenerationResponse::NotFound => {
            return HttpResponse::NotFound().body(generate_from_pgid(server_context.config.pages.notfound_page.clone()));
        }
    }
}

