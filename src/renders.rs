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

    use crate::{
        config::{CynthiaConfig, Scene, SceneCollectionTrait},
        publications::{ContentType, CynthiaPublication, PublicationContent},
    };

    use super::*;
    pub(super) fn render_controller(
        publication: CynthiaPublication,
        config: CynthiaConfClone,
    ) -> RenderrerResponse {
        let scene = fetch_scene(publication.clone(), config.clone());
        if scene.is_none() {
            error!("No scene found for publication.");
            return RenderrerResponse::Error;
        };
        // Extract the content from the publication, if it's a pagish publication, we can just
        // unwrap the result if we know it's a page or post. If it's not, we'll ignore this
        // (then `None`) variable later.
        let content = match publication {
            CynthiaPublication::Page { pagecontent, .. } => Some(fetch_content(pagecontent)),
            CynthiaPublication::Post { pagecontent, .. } => Some(fetch_content(pagecontent)),
            _ => None,
        }
        // Normally, we'd check if the publication is pagish, but we're merely testing to build
        // pages and posts here, so we'll just unwrap the result.
        .unwrap();
        let innerhtml = match content {
            FetchedContent::Ok(c) => match c {
                ContentType::Html(h) => h,
                _ => {
                    error!("Seems like an error occurred while rendering the content.");
                    return RenderrerResponse::Error;
                }
            },
            FetchedContent::Error => {
                error!("An error occurred while fetching the content.");
                return RenderrerResponse::Error;
            }
        };

        // content.unwrap().unwrap_html();
        RenderrerResponse::Ok(innerhtml)
    }
    fn fetch_scene(publication: CynthiaPublication, config: CynthiaConfClone) -> Option<Scene> {
        let scene = publication.get_scene_name();
        match scene {
            Some(s) => {
                let fetched_scene = config.scenes.get_by_name(s.as_str());
                if fetched_scene.is_none() {
                    error!("Scene \"{}\" not found in the configuration file.", s);
                    None
                } else {
                    fetched_scene
                }
            }
            None => {
                let fetched_scene = config.scenes.get_default();
                Some(fetched_scene)
            }
        }
    }

    #[derive(Debug)]
    enum FetchedContent {
        Error,
        Ok(ContentType),
    }
    struct ContentSource {
        inner: String,
        target_type: crate::publications::ContentType,
    }
    fn fetch_content(content: PublicationContent) -> FetchedContent {
        let content_output = match content {
            PublicationContent::Inline(c) => ContentSource {
                inner: c.get_inner(),
                target_type: c,
            },
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
                ContentSource {
                    inner: output,
                    target_type: source,
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
                ContentSource {
                    inner: output,
                    target_type: source,
                }
            }
        };
        let contenttype = match content_output.target_type {
            crate::publications::ContentType::Html(_) => ContentType::Html(content_output.inner),
            crate::publications::ContentType::Markdown(_) => {
                let html = match markdown::to_html_with_options(
                    content_output.inner.as_str(),
                    &markdown::Options::gfm(),
                ) {
                    Ok(html) => html,
                    Err(_) => {
                        error!("An error occurred while rendering the markdown.");
                        return FetchedContent::Error;
                    }
                };
                ContentType::Html(html)
            }
            crate::publications::ContentType::PlainText(_) => {
                ContentType::Html(format!("<pre>{}</pre>", content_output.inner))
            }
        };

        FetchedContent::Ok(contenttype)
    }
}
