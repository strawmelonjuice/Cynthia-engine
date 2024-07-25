use crate::config::CynthiaConfClone;
/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */
use crate::publications::{read_published_jsonc, CynthiaPublicationListTrait};
use crate::ServerContext;
use log::error;
use tokio::sync::MutexGuard;

pub(crate) enum PGIDCheckResponse {
    Ok,
    Error,
    NotFound,
}

#[derive(Clone)]
pub(crate) enum RenderrerResponse {
    Error,
    NotFound,
    Ok(String),
}
#[allow(unused)]
impl RenderrerResponse {
    /// Returns true if the GenerationResponse is ok.
    pub fn is_ok(&self) -> bool {
        matches!(self, RenderrerResponse::Ok(_))
    }
    /// Returns true if the GenerationResponse is not found.
    pub fn is_not_found(&self) -> bool {
        matches!(self, RenderrerResponse::NotFound)
    }
    /// Returns true if the GenerationResponse is an error.
    pub fn is_error(&self) -> bool {
        matches!(self, RenderrerResponse::Error)
    }
    /// Unwraps the GenerationResponse into a String.
    pub fn unwrap(self) -> String {
        match self {
            RenderrerResponse::Ok(s) => s,
            _ => String::new(),
        }
    }
    fn within(&mut self, then: fn(inner: String) -> String) -> &Self {
        let ob = self.clone();
        if matches!(self, RenderrerResponse::Ok(_)) {
            let inner = ob.unwrap();
            let new_inner = then(inner);
            *self = RenderrerResponse::Ok(new_inner);
        }
        self
    }
}

pub(crate) fn check_pgid(
    pgid: String,
    server_context: &MutexGuard<ServerContext>,
) -> PGIDCheckResponse {
    let page_id = if pgid == *"" {
        String::from("root")
    } else {
        pgid
    };
    let published = read_published_jsonc();

    if !published.validate(server_context.config.clone()) {
        error!("Incorrect publications found in publications.jsonc.");
        return PGIDCheckResponse::Error;
    }
    let publication = published.get_by_id(page_id);
    if publication.is_none() {
        let publication = published.get_by_id(server_context.config.pages.notfound_page.clone());
        if publication.is_none() {
            error!(
                "No 404 page found in publications.jsonc, or incorrectly defined in CynthiaConfig."
            );
            PGIDCheckResponse::Error
        } else {
            PGIDCheckResponse::NotFound
        }
    } else {
        PGIDCheckResponse::Ok
    }
}
pub(crate) fn render_from_pgid(pgid: String, config: CynthiaConfClone) -> RenderrerResponse {
    let published = read_published_jsonc();
    let publication = if pgid == *"" {
        published.get_root()
    } else {
        published.get_by_id(pgid)
    };
    if publication.is_none() {
        if published.get_notfound(config).is_none() {
            RenderrerResponse::Error
        } else {
            RenderrerResponse::NotFound
        }
    } else if let Some(pb) = publication {
        in_renderer::render_controller(pb, config)
    } else {
        RenderrerResponse::Error
    }
}

mod in_renderer {
    use std::{fs, path::Path};

    use crate::publications::{ContentType, CynthiaPublication, PublicationContent};

    use super::*;
    pub(super) fn render_controller(
        publication: CynthiaPublication,
        _config: CynthiaConfClone,
    ) -> RenderrerResponse {
        // Extract the content from the publication, if it's a pagish publication, we can just
        // unwrap the result if we know it's a page or post. If it's not, we'll ignore this
        // variable later.
        let content = match publication {
            CynthiaPublication::Page { pagecontent, .. } => Some(fetch_content(pagecontent)),
            CynthiaPublication::Post { pagecontent, .. } => Some(fetch_content(pagecontent)),
            _ => None,
        };
        RenderrerResponse::Ok(format!("{:#?}", content.unwrap().unwrap()))
    }
    #[derive(Debug)]
    enum FetchedContent {
        Error,
        Ok(ContentType),
    }

    impl FetchedContent {
        fn unwrap(self) -> ContentType {
            match self {
                FetchedContent::Ok(c) => c,
                FetchedContent::Error => ContentType::PlainText(String::from("An error occurred.")),
            }
        }
    }

    fn fetch_content(content: PublicationContent) -> FetchedContent {
        let contenttype = match content {
            PublicationContent::Inline(c) => c,
            PublicationContent::External { source } => {
                let output = match reqwest::blocking::get(source.get_inner()) {
                    Ok(w) => match w.text() {
                        Ok(o) => o,
                        Err(e) => {
                            error!(
                                "Could not fetch external content from {}\n\n{e}",
                                source.get_inner()
                            );
                            return FetchedContent::Error;
                        }
                    },
                    Err(e) => {
                        error!(
                            "Could not fetch external content from {}\n\n{e}",
                            source.get_inner()
                        );
                        return FetchedContent::Error;
                    }
                };
                match source {
                    crate::publications::ContentType::Html(_) => ContentType::Html(output),
                    crate::publications::ContentType::Markdown(_) => ContentType::Markdown(output),
                    crate::publications::ContentType::PlainText(_) => {
                        ContentType::PlainText(output)
                    }
                }
            }
            PublicationContent::Local { source } => {
                let output = {
                    let mut v = String::from("./cynthiaFiles/publications/");
                    v.push_str(&source.get_inner());
                    if Path::new(v.as_str()).exists() {
                        match fs::read_to_string(v.clone()) {
                            Ok(t) => t,
                            Err(e) => {
                                error!("Could not read local content at {}\n\n{e}", v);
                                return FetchedContent::Error;
                            }
                        }
                    } else {
                        error!("Could not find local content at {}", v);
                        return FetchedContent::Error;
                    }
                };

                match source {
                    crate::publications::ContentType::Html(_) => ContentType::Html(output),
                    crate::publications::ContentType::Markdown(_) => ContentType::Markdown(output),
                    crate::publications::ContentType::PlainText(_) => {
                        ContentType::PlainText(output)
                    }
                }
            }
        };
        FetchedContent::Ok(contenttype)
    }
}
