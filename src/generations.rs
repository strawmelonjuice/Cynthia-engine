/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */
use log::error;
use tokio::sync::{MutexGuard};
use crate::publications::{CynthiaPublicationListTrait, read_published_jsonc};
use crate::ServerContext;

pub(crate) enum EmptyGenerationResponse {
    Ok,
    Error,
    NotFound,
}
pub(crate) enum GenerationResponse {
    Ok(String),
    Error,
    NotFound,
}

pub(crate) fn check_pgid(pgid: String, server_context: &MutexGuard<ServerContext>) -> EmptyGenerationResponse {
    let page_id = if pgid == String::from("") { String::from("root") } else { pgid };
    let published = read_published_jsonc();
    if published.is_empty() {
        error!("No correct publications found in publications.jsonc.");
        return EmptyGenerationResponse::Error;
    }
    let publication = published.get_by_id(page_id);
    if publication.is_none() {
        let publication = published.get_by_id(server_context.config.pages.notfound_page.clone());
        if publication.is_none() {
            error!("No 404 page found in publications.jsonc, or incorrectly defined in CynthiaConfig.");
            EmptyGenerationResponse::Error
        } else {
            EmptyGenerationResponse::NotFound
        }
    } else {
        EmptyGenerationResponse::Ok
    }
}
pub(crate) fn generate_from_pgid(pgid: String) -> String {
    let page_id = if pgid == String::from("") { String::from("root") } else { pgid };
    let published = read_published_jsonc();
    let publication = published.get_by_id(page_id);
    if publication.is_none() {
        panic!("No 404 page found in publications.jsonc, or incorrectly defined in CynthiaConfig.");
    } else {
        format!("{:#?}", publication.unwrap())
    }
}