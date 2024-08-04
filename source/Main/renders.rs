/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */
use std::sync::Arc;
use actix_web::web::Data;
use log::error;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, MutexGuard};

use crate::config::CynthiaConfClone;
use crate::publications::{
    Author, CynthiaPublicationDates, CynthiaPublicationList, CynthiaPublicationListTrait,
};
use crate::{LockCallback, ServerContext};
use colored::Colorize;

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
    let published = CynthiaPublicationList::load();

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
pub(crate) async fn render_from_pgid(pgid: String, server_context_mutex: Data<Arc<Mutex<ServerContext>>>) -> RenderrerResponse {
    let config = server_context_mutex.lock_callback(|a| a.config.clone()).await;
    let published = CynthiaPublicationList::load();
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
        in_renderer::render_controller(pb, server_context_mutex).await
    } else {
        RenderrerResponse::Error
    }
}

/// This struct is a stripped down version of the Scene struct in the config module.
/// It stores only the necessary data for rendering a single publication.
struct PublicationScene {
    template: String,
    // Not used yet. Will be used in the future when implementing stylesheets.
    //  I know these lints are there so I won't forget, but I'm not forgetting.
    #[allow(unused)]
    stylesheet: Option<String>,
    // Not used yet. Will be used in the future when implementing custom scripts.
    #[allow(unused)]
    script: Option<String>,
    kind: String,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct PageLikePublicationTemplateData {
    meta: PageLikePublicationTemplateDataMeta,
    content: String,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct PageLikePublicationTemplateDataMeta {
    id: String,
    title: String,
    desc: Option<String>,
    category: Option<String>,
    author: Option<Author>,
    dates: CynthiaPublicationDates,
    thumbnail: Option<String>,
}


mod in_renderer {
    use std::{fs, path::Path};
    use std::path::PathBuf;
    use handlebars::{handlebars_helper, Handlebars};

    use crate::{
        config::{CynthiaConfig, Scene, SceneCollectionTrait},
        publications::{ContentType, CynthiaPublication, PublicationContent},
    };
    use crate::externalpluginservers::EPSRequestBody;
    use super::*;

    pub(super) async fn render_controller(
        publication: CynthiaPublication,
        server_context_mutex: Data<Arc<Mutex<ServerContext>>>,
    ) -> RenderrerResponse {
        let config = server_context_mutex.lock_callback(|a| a.config.clone()).await;
        let scene = fetch_scene(publication.clone(), config.clone());

        if scene.is_none() {
            error!("No scene found for publication.");
            return RenderrerResponse::Error;
        };
        let scene = scene.unwrap();
        let scène = match publication {
            CynthiaPublication::Page { .. } => PublicationScene {
                template: scene.templates.page.clone(),
                stylesheet: scene.stylefile.clone(),
                script: scene.script.clone(),
                kind: "page".to_string(),
            },
            CynthiaPublication::Post { .. } => PublicationScene {
                template: scene.templates.post.clone(),
                stylesheet: scene.stylefile.clone(),
                script: scene.script.clone(),
                kind: "post".to_string(),
            },
            CynthiaPublication::PostList { .. } => PublicationScene {
                template: scene.templates.postlist.clone(),
                stylesheet: scene.stylefile.clone(),
                script: scene.script.clone(),
                kind: "postlist".to_string(),
            },
        };

        let pageish_template_data: PageLikePublicationTemplateData = match publication {
            CynthiaPublication::Page {
                pagecontent,
                id,
                title,
                thumbnail,
                description,
                dates,
                ..
            } => PageLikePublicationTemplateData {
                meta: PageLikePublicationTemplateDataMeta {
                    id: id.clone(),
                    title: title.clone(),
                    desc: description.clone(),
                    category: None,
                    author: None,
                    dates: dates.clone(),
                    thumbnail: thumbnail.clone(),
                },
                content: match fetch_page_ish_content(pagecontent).await.unwrap_html() {
                    RenderrerResponse::Ok(s) => s,
                    _ => return RenderrerResponse::Error,
                },
            },
            CynthiaPublication::Post {
                id,
                title,
                short,
                dates,
                thumbnail,
                category,
                author,
                postcontent,
                ..
            } => PageLikePublicationTemplateData {
                meta: PageLikePublicationTemplateDataMeta {
                    id: id.clone(),
                    title: title.clone(),
                    desc: short.clone(),
                    category: category.clone(),
                    author: author.clone(),
                    dates: dates.clone(),
                    thumbnail: thumbnail.clone(),
                },
                content: match fetch_page_ish_content(postcontent).await.unwrap_html() {
                    RenderrerResponse::Ok(s) => s,
                    _ => return RenderrerResponse::Error,
                },
            },
            _ => todo!("Implement fetching content for postlists."),
        };

        let outerhtml: String = {
            let cwd: PathBuf = std::env::current_dir().unwrap();
            let template_path = cwd.join("cynthiaFiles/templates/".to_owned() + &*scène.kind.clone() +"/"+ &*scène.template.clone() + ".hbs");
            if !template_path.exists() {
                error!(
                    "Template file '{}' not found.",
                    template_path.display()
                );
                return RenderrerResponse::Error;
            }
            // A fallback function that uses the builtin handlebars renderer.
            let builtin_handlebars = || {
                let mut template = Handlebars::new();
                // Num = equal helper
                // This helper checks if two numbers are equal.
                // Usage: {{#if (numequal posttype 2)}} ... {{/if}}
                handlebars_helper!(num_is_equal: |x: usize, y: usize| x == y);
                template.register_helper("numequal", Box::new(num_is_equal));
                match template.register_template_file("base", template_path.clone()) {
                    Ok(_) => {}
                    Err(e) => {
                        error!(
                            "Error reading template file '{}':\n\n{}",
                            template_path.display(),
                            format!("{}", e).bright_red()
                        );
                        return RenderrerResponse::Error;
                    }
                };
                RenderrerResponse::Ok(template.render("base", &pageish_template_data.meta).unwrap())
            };
            let htmlbody: String = if !cfg!(feature = "js_runtime") {
                // Fall back to builtin handlebars if the js_runtime feature is not enabled.
                if let RenderrerResponse::Ok(a) = builtin_handlebars() {
                    a
                } else {
                    return RenderrerResponse::Error;
                }
            } else {
                if let crate::externalpluginservers::EPSResponseBody::OkString { value } =
                crate::externalpluginservers::contact_eps(
                    server_context_mutex,
                    EPSRequestBody::ContentRenderRequest {
                        template_path: template_path.to_string_lossy().parse().unwrap(),
                        template_data: pageish_template_data.clone(),
                    }
                ).await {
                    value
                } else {
                    // Fall back to builtin handlebars if the external plugin server fails.
                    if let RenderrerResponse::Ok(a) = builtin_handlebars() {
                        a
                    } else {
                        return RenderrerResponse::Error;
                    }
                }
            };
            format!(
                "<html><head><title>{}</title></head><body>{}</body></html>",
                pageish_template_data.meta.title,
                htmlbody
            )
        };

        // content.unwrap().unwrap_html();
        RenderrerResponse::Ok(outerhtml)
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

    impl FetchedContent {
        fn unwrap_html(self) -> RenderrerResponse {
            match self {
                FetchedContent::Ok(c) => match c {
                    ContentType::Html(h) => RenderrerResponse::Ok(h),
                    _ => {
                        error!("An error occurred while unwrapping the content.");
                        RenderrerResponse::Error
                    }
                },
                FetchedContent::Error => {
                    error!("An error occurred while unwrapping the content.");
                    RenderrerResponse::Error
                }
            }
        }
    }
    struct ContentSource {
        inner: String,
        target_type: ContentType,
    }
    #[doc = "Fetches the content of a pageish (a post or a page) publication."]
    async fn fetch_page_ish_content(content: PublicationContent) -> FetchedContent {
        let content_output = match content {
            PublicationContent::Inline(c) => ContentSource {
                inner: c.get_inner(),
                target_type: c,
            },
            PublicationContent::External { source } => {
                let a = reqwest::get(source.get_inner()).await;
                let output = match a {
                    Ok(w) => match w.text().await {
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
