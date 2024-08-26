/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */
use actix_web::web::Data;
use log::error;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::CynthiaConfClone;
use crate::publications::{CynthiaPostList, CynthiaPublicationList, CynthiaPublicationListTrait};
use crate::{LockCallback, ServerContext};

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

pub(crate) async fn check_pgid(
    pgid: String,
    server_context_mutex: Data<Arc<Mutex<ServerContext>>>,
) -> PGIDCheckResponse {
    let page_id = if pgid == *"" {
        String::from("root")
    } else {
        pgid
    };
    let published = CynthiaPublicationList::load(server_context_mutex.clone()).await;
    let server_context = server_context_mutex.lock().await;
    if !published.validate(server_context.config.clone()) {
        error!("Incorrect publications found in publications.jsonc.");
        return PGIDCheckResponse::Error;
    }
    let publication = published.get_by_id(page_id);
    if publication.is_none() {
        let publication = published.get_by_id(server_context.config.site.notfound_page.clone());
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
pub(crate) async fn render_from_pgid(
    pgid: String,
    server_context_mutex: Data<Arc<Mutex<ServerContext>>>,
) -> RenderrerResponse {
    let config = server_context_mutex
        .lock_callback(|a| a.config.clone())
        .await;
    let published = CynthiaPublicationList::load(server_context_mutex.clone()).await;
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
        in_renderer::render_controller(pb, server_context_mutex.clone()).await
    } else {
        RenderrerResponse::Error
    }
}

/// This struct is a stripped down version of the Scene struct in the config module.
/// It stores only the necessary data for rendering a single publication.
struct PublicationScene {
    template: String,
    stylesheet: Option<String>,
    script: Option<String>,
    kind: String,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct PageLikePublicationTemplateData {
    meta: PageLikePublicationTemplateDataMeta,
    content: String,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct PostListPublicationTemplateData {
    meta: PageLikePublicationTemplateDataMeta,
    posts: CynthiaPostList,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct PageLikePublicationTemplateDataMeta {
    id: String,
    title: String,
    desc: Option<String>,
    category: Option<String>,
    tags: Vec<String>,
    author: Option<crate::publications::Author>,
    dates: crate::publications::CynthiaPublicationDates,
    thumbnail: Option<String>,
}

mod in_renderer {
    use super::*;
    use crate::externalpluginservers::EPSRequestBody;
    use crate::publications::{CynthiaPostList, CynthiaPublicationListTrait, PostLists};
    use crate::tell::CynthiaColors;
    use crate::{
        config::{CynthiaConfig, Scene, SceneCollectionTrait},
        publications::{ContentType, CynthiaPublication, PublicationContent},
    };
    use handlebars::{handlebars_helper, Handlebars};
    use log::warn;
    use std::path::PathBuf;
    use std::{fs, path::Path};
    use ContentType::Html;

    pub(super) async fn render_controller(
        publication: CynthiaPublication,
        server_context_mutex: Data<Arc<Mutex<ServerContext>>>,
    ) -> RenderrerResponse {
        let config = server_context_mutex
            .lock_callback(|a| a.config.clone())
            .await;
        let scene = fetch_scene(publication.clone(), config.clone());

        if scene.is_none() {
            error!("No scene found for publication.");
            return RenderrerResponse::Error;
        };
        let scene = scene.unwrap();
        let localscene = match publication {
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

        let mut pageish_template_data: PageLikePublicationTemplateData =
            PageLikePublicationTemplateData::default();
        let mut postlist_template_data: PostListPublicationTemplateData =
            PostListPublicationTemplateData::default();
        match publication {
            CynthiaPublication::Page {
                pagecontent,
                id,
                title,
                thumbnail,
                description,
                dates,
                ..
            } => {
                pageish_template_data = PageLikePublicationTemplateData {
                    meta: PageLikePublicationTemplateDataMeta {
                        id: id.clone(),
                        title: title.clone(),
                        desc: description.clone(),
                        category: None,
                        author: None,
                        tags: vec![],
                        dates: dates.clone(),
                        thumbnail: thumbnail.clone(),
                    },
                    content: match fetch_page_ish_content(pagecontent).await.unwrap_html() {
                        RenderrerResponse::Ok(s) => s,
                        _ => return RenderrerResponse::Error,
                    },
                }
            }
            CynthiaPublication::Post {
                id,
                title,
                short,
                dates,
                thumbnail,
                category,
                author,
                postcontent,
                tags,
                ..
            } => {
                pageish_template_data = PageLikePublicationTemplateData {
                    meta: PageLikePublicationTemplateDataMeta {
                        id: id.clone(),
                        title: title.clone(),
                        desc: short.clone(),
                        category: category.clone(),
                        author: author.clone(),
                        dates: dates.clone(),
                        thumbnail: thumbnail.clone(),
                        tags: tags.clone(),
                    },
                    content: match fetch_page_ish_content(postcontent).await.unwrap_html() {
                        RenderrerResponse::Ok(s) => s,
                        _ => return RenderrerResponse::Error,
                    },
                }
            }
            CynthiaPublication::PostList {
                id,
                title,
                short,
                filter,
                ..
            } => {
                let publicationlist: CynthiaPublicationList =
                    CynthiaPublicationList::load(server_context_mutex.clone()).await;
                let postlist: CynthiaPostList = publicationlist.only_posts();
                let filtered_postlist = postlist.filter(filter);
                postlist_template_data = PostListPublicationTemplateData {
                    meta: PageLikePublicationTemplateDataMeta {
                        id: id.clone(),
                        title: title.clone(),
                        desc: short.clone(),
                        category: None,
                        tags: vec![],
                        author: None,
                        dates: crate::publications::CynthiaPublicationDates {
                            altered: 0,
                            published: 0,
                        },
                        thumbnail: None,
                    },
                    posts: filtered_postlist,
                };
                pageish_template_data.meta = postlist_template_data.meta.clone();
                // println!("{}", serde_json::to_string(&postlist_template_data).unwrap());
            }
        };

        let outerhtml: String = {
            let cwd: PathBuf = std::env::current_dir().unwrap();
            let template_path = cwd.join(
                "cynthiaFiles/templates/".to_owned()
                    + &*localscene.kind.clone()
                    + "/"
                    + &*localscene.template.clone()
                    + ".hbs",
            );
            if !template_path.exists() {
                error!("Template file '{}' not found.", template_path.display());
                return RenderrerResponse::Error;
            }

            // A fallback function that uses the builtin handlebars renderer.
            let builtin_handlebars = |data| {
                let mut template = Handlebars::new();
                // streq helper
                // This helper checks if two strings are equal.
                // Usage: {{#if (streq postid "sasfs")}} ... {{/if}}
                handlebars_helper!(streq: |x: str, y: str| x == y);
                template.register_helper("streq", Box::new(streq));
                match template.register_template_file("base", template_path.clone()) {
                    Ok(g) => g,
                    Err(e) => {
                        error!(
                            "Error reading template file '{}':\n\n{}",
                            template_path.display(),
                            e.to_string().color_bright_red()
                        );
                        return RenderrerResponse::Error;
                    }
                };
                match template.render("base", &data) {
                    Ok(a) => RenderrerResponse::Ok(a),
                    Err(e) => {
                        error!(
                            "Error rendering template file '{}':\n\n{}",
                            template_path.display(),
                            e.to_string().color_bright_red()
                        );
                        RenderrerResponse::Error
                    }
                }
            };
            let mut htmlbody: String = if !cfg!(feature = "js_runtime") {
                // Fall back to builtin handlebars if the js_runtime feature is not enabled.
                if let RenderrerResponse::Ok(a) = builtin_handlebars(pageish_template_data.clone())
                {
                    a
                } else {
                    return RenderrerResponse::Error;
                }
            } else if let crate::externalpluginservers::EPSResponseBody::OkString { value } = {
                if localscene.kind != *"postlist" {
                    crate::externalpluginservers::contact_eps(
                        server_context_mutex.clone(),
                        EPSRequestBody::ContentRenderRequest {
                            template_path: template_path.to_string_lossy().parse().unwrap(),
                            template_data: pageish_template_data.clone(),
                        },
                    )
                    .await
                } else {
                    let req = EPSRequestBody::PostlistRenderRequest {
                        template_path: template_path.to_string_lossy().parse().unwrap(),
                        template_data: postlist_template_data.clone(),
                    };
                    // println!("{}", serde_json::to_string(&req).unwrap());
                    crate::externalpluginservers::contact_eps(server_context_mutex.clone(), req)
                        .await
                }
            } {
                value
            } else {
                warn!("External Javascript Runtime failed to render the content. Retrying with basic builtin rendering.");
                // Fall back to builtin handlebars if the external plugin server fails.
                if let RenderrerResponse::Ok(a) = builtin_handlebars(pageish_template_data.clone())
                {
                    a
                } else {
                    return RenderrerResponse::Error;
                }
            };
            let version = env!("CARGO_PKG_VERSION");
            let mut head = String::new();
            head.push_str("\n\t<head>");
            head.push_str("\n\t\t<meta charset=\"utf-8\" />");
            head.push_str(
                format!(
                    "\n\t\t<title>{}{}</title>",
                    pageish_template_data.meta.title.clone(),
                    match scene.sitename {
                        Some(s) => format!(" - {}", s),
                        None => String::new(),
                    }
                )
                .as_str(),
            );
            head.push_str("\n\t\t<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\" />");
            head.push_str("\n\t\t<meta name=\"generator\" content=\"strawmelonjuice-Cynthia\" />");
            head.push_str("\n\t\t<meta name=\"robots\" content=\"index, follow\" />");
            if let Some(stylefile) = localscene.stylesheet {
                let path: PathBuf = std::env::current_dir()
                    .unwrap()
                    .canonicalize()
                    .unwrap()
                    .join("./cynthiaFiles/assets/".to_string() + stylefile.as_str());
                if path.exists() {
                    let css = inlines::inline_css(path, server_context_mutex.clone()).await;
                    head.push_str(&css);
                } else {
                    error!("Stylesheet file '{}' not found.", path.display());
                    return RenderrerResponse::Error;
                }
            }
            head.push_str(
&format!("<script>const cynthia = {{version: '{}', publicationdata: JSON.parse(`{}`), kind: '{}'}};</script>",
                version,
    serde_json::to_string(&pageish_template_data.meta.clone()).unwrap(),
                localscene.kind)


            );
            if let Some(script) = localscene.script {
                let path: PathBuf = std::env::current_dir()
                    .unwrap()
                    .canonicalize()
                    .unwrap()
                    .join("./cynthiaFiles/assets/".to_string() + script.as_str());
                if path.exists() {
                    let d = inlines::inline_js(path, server_context_mutex.clone()).await;
                    htmlbody.push_str(&d);
                } else {
                    error!("Script file '{}' not found.", path.display());
                    return RenderrerResponse::Error;
                }
            }
            if let Some(author) = pageish_template_data.meta.author {
                if let Some(author_name) = author.name {
                    head.push_str(&format!(
                        "\n\t\t<meta name=\"author\" content=\"{}\" />",
                        author_name
                    ));
                }
            }
            if let Some(category) = pageish_template_data.meta.category {
                head.push_str(&format!(
                    "\n\t\t<meta name=\"category\" content=\"{}\" />",
                    category
                ));
            }
            if let Some(desc) = pageish_template_data.meta.desc {
                head.push_str(&format!(
                    "\n\t\t<meta name=\"description\" content=\"{}\" />",
                    desc
                ));
            }
            if let Some(thumbnail) = pageish_template_data.meta.thumbnail {
                head.push_str(&format!(
                    "\n\t\t<meta property=\"og:image\" content=\"{}\" />",
                    thumbnail
                ));
            }
            head.push_str("\n\t</head>");
            let docurl = "https://github.com/strawmelonjuice/CynthiaWebsiteEngine";
            format!(
                "<!DOCTYPE html>\n<html>\n<!--\n\nGenerated and hosted through Cynthia v{version}, by Strawmelonjuice.\nAlso see:	<{docurl}>\n-->\n{head}\n<body>{htmlbody}</body></html>",
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
                    Html(h) => RenderrerResponse::Ok(h),
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
            Html(_) => Html(content_output.inner),
            ContentType::Markdown(_) => {
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
                Html(html)
            }
            ContentType::PlainText(_) => {
                Html("<pre>".to_owned() + content_output.inner.as_str() + "</pre>")
            }
        };

        FetchedContent::Ok(contenttype)
    }
}
#[cfg(feature = "js_runtime")]
mod inlines {
    use crate::tell::CynthiaColors;
    use crate::{LockCallback, ServerContext};
    use actix_web::web::Data;
    use log::{debug, error, info, warn};
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    pub(crate) async fn inline_js(
        scriptfile: PathBuf,
        server_context_mutex: Data<Arc<Mutex<ServerContext>>>,
    ) -> String {
        let config_clone = server_context_mutex
            .lock_callback(|a| {
                a.request_count += 1;
                a.config.clone()
            })
            .await;
        let embed_id = format!(
            "script:{}",
            scriptfile
                .clone()
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
        );
        let jscachelifetime: u64 = config_clone.cache.lifetimes.javascript;
        let cache_result = server_context_mutex
            .lock_callback(|servercontext| servercontext.get_cache(&embed_id, jscachelifetime))
            .await;
        match cache_result {
            Some(o) => {
                let d = std::str::from_utf8(&o.0).unwrap().to_string();
                return format!(
                    "<script>\n\r// Minified internally by Cynthia using Terser\n\n{d}\n\n\r// Cached after minifying, so might be somewhat behind.\n\r</script>");
            }
            None => {
                info!("Minifying JS file '{}'...", scriptfile.display());
                let xargs: Vec<&str>;
                let scri = scriptfile.clone();
                let scr = scri.to_str().unwrap();
                let runner = {
                    if config_clone.runtimes.ext_js_rt.as_str().contains("bun") {
                        xargs = [
                            "terser",
                            scr,
                            "--compress",
                            "--keep-fnames",
                            "--keep-classnames",
                        ]
                        .to_vec();

                        "bunx"
                    } else {
                        xargs = [
                            "--yes",
                            "terser",
                            scr,
                            "--compress",
                            "--keep-fnames",
                            "--keep-classnames",
                        ]
                        .to_vec();

                        "npx"
                    }
                };

                debug!("Running Terser in {}", runner.color_purple());
                match std::process::Command::new(runner)
                    .args(xargs.clone())
                    .output()
                {
                    Ok(output) => {
                        if output.status.success() {
                            let d = format!("{}", String::from_utf8_lossy(&output.stdout));
                            {
                                let mut server_context = server_context_mutex.lock().await;
                                server_context
                                    .store_cache_async(&embed_id, d.as_bytes(), jscachelifetime)
                                    .await
                                    .unwrap();
                            };
                            return format!(
                                "<script>\n\r// Minified internally by Cynthia using Terser\n\n{d}\n\n\r// Cached after minifying, so might be somewhat behind.\n\r</script>");
                        } else {
                            warn!(
                                "Failed running Terser in {}, couldn't minify to embed JS.",
                                config_clone.runtimes.ext_js_rt.as_str().color_purple()
                            );
                            println!("Ran command \"{} {}\"", runner.color_purple(), {
                                let mut s = String::new();
                                for a in &xargs {
                                    s.push_str(a);
                                    s.push(' ');
                                }
                                s
                            })
                        }
                    }
                    Err(why) => {
                        error!(
                            "Failed running CleanCSS in {}, couldn't minify to embed JS: {}",
                            config_clone.runtimes.ext_js_rt.as_str().color_purple(),
                            why
                        );
                    }
                }
            }
        };
        warn!("Scriptfile could not be minified, so was instead inlined 1:1.");
        //     If we got here, we couldn't minify the JS.
        let file_content = fs::read_to_string(scriptfile).unwrap_or_default();
        format!("<script>\n// Scriptfile could not be minified, so was instead inlined 1:1. \n\n{}</script>", file_content)
    }

    pub(crate) async fn inline_css(
        stylefile: PathBuf,
        server_context_mutex: Data<Arc<Mutex<ServerContext>>>,
    ) -> String {
        let config_clone = server_context_mutex
            .lock_callback(|a| {
                a.request_count += 1;
                a.config.clone()
            })
            .await;
        let embed_id = format!(
            "css:{}",
            stylefile
                .clone()
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
        );

        let csscachelifetime: u64 = config_clone.cache.lifetimes.stylesheets;
        let cache_result = server_context_mutex
            .lock_callback(|servercontext| servercontext.get_cache(&embed_id, csscachelifetime))
            .await;
        match cache_result {
            Some(o) => {
                let d = std::str::from_utf8(&o.0).unwrap().to_string();
                return format!(
                    "\n\t\t<style>\n\n\t\t\t/* Minified internally by Cynthia using clean-css */\n\n\t\t\t{d}\n\n\t\t\t/* Cached after minifying, so might be somewhat behind. */\n\t\t</style>");
            }
            None => {
                info!("Minifying CSS file '{}'...", stylefile.display());
                let xargs: Vec<&str>;
                let styf = stylefile.clone();
                let stf = styf.to_str().unwrap();
                let runner = {
                    if config_clone.runtimes.ext_js_rt.as_str().contains("bun") {
                        xargs = ["clean-css-cli@4", "-O2", "--inline", "none", stf].to_vec();

                        "bunx"
                    } else {
                        xargs =
                            ["--yes", "clean-css-cli@4", "-O2", "--inline", "none", stf].to_vec();

                        "npx"
                    }
                };
                debug!("Running CleanCSS in {}", runner.color_purple());
                match std::process::Command::new(runner)
                    .args(xargs.clone())
                    .output()
                {
                    Ok(output) => {
                        if output.status.success() {
                            let d = format!("{}", String::from_utf8_lossy(&output.stdout));
                            {
                                let mut server_context = server_context_mutex.lock().await;
                                server_context
                                    .store_cache_async(&embed_id, d.as_bytes(), csscachelifetime)
                                    .await
                                    .unwrap();
                            }
                            return format!(
                                    "\n\t\t<style>\n\n\t\t\t/* Minified internally by Cynthia using clean-css */\n\n\t\t\t{d}\n\n\t\t\t/* Cached after minifying, so might be somewhat behind. */\n\t\t</style>");
                        }
                    }
                    Err(why) => {
                        error!(
                            "Failed running CleanCSS in {}, couldn't minify to embed CSS: {}",
                            config_clone.runtimes.ext_js_rt.as_str().color_purple(),
                            why
                        );
                        debug!("Ran command \"{} {}\"", runner.color_purple(), {
                            let mut s = String::new();
                            for a in &xargs {
                                s.push_str(a);
                                s.push(' ');
                            }
                            s
                        });
                    }
                }
            }
        };
        warn!("Stylefile could not be minified, so was instead inlined 1:1.");
        //     If we got here, we couldn't minify the CSS.
        let file_content = fs::read_to_string(stylefile).unwrap_or_default();
        format!("<style>\n/* Stylefile could not be minified, so was instead inlined 1:1. */\n\n{}</style>", file_content)
    }
}

#[cfg(not(feature = "js_runtime"))]
mod inlines {
    pub(crate) async fn inline_js(
        scriptfile: PathBuf,
        _server_context_mutex: Data<Arc<Mutex<ServerContext>>>,
    ) -> String {
        let file_content = fs::read_to_string(scriptfile).unwrap_or(String::new());
        format!("<script>{}</script>", file_content)
    }
    pub(crate) async fn inline_css(
        stylefile: PathBuf,
        _server_context_mutex: Data<Arc<Mutex<ServerContext>>>,
    ) -> String {
        let file_content = fs::read_to_string(stylefile).unwrap_or(String::new());
        format!("<style>{}</style>", file_content)
    }
}

pub(crate) mod json_html {

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn stringify_inner() {
            let inner = vec![
                ContentBlock::Paragraph {
                    inner: "Hello, World!".to_string(),
                },
                ContentBlock::UnorderedList {
                    inner: vec![
                        ContentBlock::ListItem {
                            inner: Inner::Elements(vec![ContentBlock::Image {
                                src: "http://hi.jpg".to_string(),
                                alt: Some("hi".to_string()),
                            }]),
                        },
                        ContentBlock::ListItem {
                            inner: Inner::Text("Hello, World!".to_string()),
                        },
                    ],
                },
            ];
            let result = serde_json::to_string(&inner).unwrap();
            let expectation = r#"[{"type":"paragraph","inner":"Hello, World!"},{"type":"unordered-list","inner":[{"type":"list-item","inner":{"elements":[{"type":"image","src":"http://hi.jpg","alt":"hi"}]}},{"type":"list-item","inner":"Hello, World!"}]}]"#.to_string();
            assert_eq!(result, expectation);
            let content_block_json = result.from_string().unwrap();
            println!("{}", content_block_json.as_str());
            let result = content_block_json.to_html().unwrap();
            println!("{}", result);
            let expectation = "<p>Hello, World!</p><ul><li><img src=\"http://hi.jpg\" alt=\"hi\"></li><li>Hello, World!</li></ul>";
            assert_eq!(result, expectation);
        }

        #[test]
        fn paragraphs() {
            let result = (r#"[{"type":"text","inner":"Hello, World!"}]"#.from_string())
                .unwrap()
                .to_html()
                .unwrap();
            let expectation = "<p>Hello, World!</p>".to_string();
            assert_eq!(result, expectation);
        }
        #[test]
        fn lists() {
            let json_str = r#"
[
    {
        "type": "unordered-list",
        "inner": [
            {
                "type": "li",
                "inner": {"elements": [
                    {
                        "type": "paragraph",
                        "inner": "**Hello, World!**"
                    }
                ]}
            },
            {
                "type": "li",
                "inner": {"elements": [
                    {
                        "type": "image",
                        "src": "http://hi.jpg"
                    }
                ]}
            }
        ]
    }
]
            "#;
            let orpoerpoe = serde_json::from_str::<Vec<ContentBlock>>(json_str);
            println!("{:?}", orpoerpoe);
            let content_block_json = json_str.from_string().unwrap();
            println!("{}", content_block_json.as_str());
            let result = content_block_json.to_html().unwrap();
            println!("{}", result);
            let expectation =
                "<ul><li><p>**Hello, World!**</p></li><li><img src=\"http://hi.jpg\"></li></ul>";
            assert_eq!(result, expectation);
        }
    }

    use log::error;
    use serde::{Deserialize, Serialize};

    fn inhouse_rewrap<T, E: std::fmt::Display>(value: Result<T, E>) -> Result<T, String> {
        match value {
            Ok(value) => Ok(value),
            Err(err) => Err(format!("{err}")),
        }
    }

    pub struct ContentBlocksJson {
        json: String,
    }

    impl ContentBlocksJson {
        #[allow(unused)]
        pub fn as_str(&self) -> &str {
            &self.json
        }
    }
    trait ToHtml {
        fn to_html(&self) -> String;
    }
    trait ToHtmlFallible {
        fn to_html(&self) -> Result<String, String>;
    }
    trait FromString {
        fn from_string(self) -> Result<ContentBlocksJson, String>;
    }
    impl FromString for &str {
        fn from_string(self) -> Result<ContentBlocksJson, String> {
            let blocks =
                inhouse_rewrap::<Vec<ContentBlock>, serde_json::Error>(serde_json::from_str(self));
            if blocks.is_ok() {
                Ok(ContentBlocksJson {
                    json: self.to_string(),
                })
            } else {
                Err("Could not parse JSON.".to_string())
            }
        }
    }
    impl FromString for String {
        fn from_string(self) -> Result<ContentBlocksJson, String> {
            let blocks =
                inhouse_rewrap::<Vec<ContentBlock>, serde_json::Error>(serde_json::from_str(&self));
            if blocks.is_ok() {
                Ok(ContentBlocksJson { json: self })
            } else {
                Err("Could not parse JSON.".to_string())
            }
        }
    }
    #[derive(Debug, Deserialize, Serialize)]
    #[serde(tag = "type")]
    #[serde(rename_all = "kebab-case")]
    pub enum ContentBlock {
        // Basic HTML blocks.
        #[serde(alias = "paragraph")]
        #[serde(alias = "p")]
        #[serde(alias = "text")]
        Paragraph { inner: String },
        #[serde(alias = "header-1")]
        #[serde(alias = "h1")]
        Header1 { inner: String },
        #[serde(alias = "header-2")]
        #[serde(alias = "h2")]
        Header2 { inner: String },
        #[serde(alias = "header-3")]
        #[serde(alias = "h3")]
        Header3 { inner: String },
        #[serde(alias = "header-4")]
        #[serde(alias = "h4")]
        Header4 { inner: String },
        #[serde(alias = "header-5")]
        #[serde(alias = "h5")]
        Header5 { inner: String },
        #[serde(alias = "header-6")]
        #[serde(alias = "h6")]
        Header6 { inner: String },
        #[serde(alias = "list-item")]
        #[serde(alias = "li")]
        #[serde(alias = "listitem")]
        ListItem { inner: Inner },
        #[serde(alias = "unordered-list")]
        #[serde(alias = "ul")]
        UnorderedList { inner: Vec<ContentBlock> },
        #[serde(alias = "ordered-list")]
        #[serde(alias = "ol")]
        OrderedList { inner: Vec<ContentBlock> },
        #[serde(alias = "blockquote")]
        Blockquote { inner: Inner },
        #[serde(alias = "code")]
        Code { inner: Inner },
        #[serde(alias = "code-block")]
        CodeBlock { inner: Inner },
        #[serde(alias = "image")]
        Image { src: String, alt: Option<String> },
        #[serde(alias = "link")]
        Link { href: String, inner: Inner },
        #[serde(alias = "horizontal-rule")]
        #[serde(alias = "hr")]
        HorizontalRule,
        #[serde(alias = "div")]
        #[serde(alias = "divblock")]
        #[serde(alias = "div-block")]
        DivBlock { inner: Vec<ContentBlock> },
        #[serde(alias = "span")]
        #[serde(alias = "spanblock")]
        #[serde(alias = "span-block")]
        SpanBlock { inner: Inner },
        #[serde(alias = "bttn")]
        #[serde(alias = "button")]
        Button { inner: Inner },
        // Embed formats blocks.
        #[serde(alias = "html")]
        #[serde(alias = "raw-html")]
        Html { content: String },
        #[serde(alias = "md")]
        #[serde(alias = "markdown")]
        Markdown { content: String },
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub enum Inner {
        #[serde(rename = "elements")]
        Elements(Vec<ContentBlock>),
        #[serde(untagged)]
        Text(String),
    }

    impl ToHtmlFallible for ContentBlocksJson {
        fn to_html(&self) -> Result<String, String> {
            let blocks: Vec<ContentBlock> = inhouse_rewrap::<Vec<ContentBlock>, serde_json::Error>(
                serde_json::from_str(&self.json),
            )?;
            Ok(blocks.to_html())
        }
    }

    impl ToHtml for Vec<ContentBlock> {
        fn to_html(&self) -> String {
            let mut html = String::new();
            for block in self {
                html.push_str(&block.to_html());
            }
            html
        }
    }

    impl ToHtml for Inner {
        fn to_html(&self) -> String {
            match self {
                Inner::Text(text) => text.clone(),
                Inner::Elements(elements) => {
                    let mut html = String::new();
                    for element in elements {
                        html.push_str(&element.to_html());
                    }
                    html
                }
            }
        }
    }

    impl ToHtml for ContentBlock {
        fn to_html(&self) -> String {
            match self {
                ContentBlock::Paragraph { inner } => format!("<p>{}</p>", inner),
                ContentBlock::Header1 { inner } => format!("<h1>{}</h1>", inner),
                ContentBlock::Header2 { inner } => format!("<h2>{}</h2>", inner),
                ContentBlock::Header3 { inner } => format!("<h3>{}</h3>", inner),
                ContentBlock::Header4 { inner } => format!("<h4>{}</h4>", inner),
                ContentBlock::Header5 { inner } => format!("<h5>{}</h5>", inner),
                ContentBlock::Header6 { inner } => format!("<h6>{}</h6>", inner),
                ContentBlock::ListItem { inner } => format!("<li>{}</li>", inner.to_html()),
                ContentBlock::UnorderedList { inner } => format!("<ul>{}</ul>", inner.to_html()),
                ContentBlock::OrderedList { inner } => format!("<ol>{}</ol>", inner.to_html()),
                ContentBlock::Blockquote { inner } => {
                    format!("<blockquote>{}</blockquote>", inner.to_html())
                }
                ContentBlock::Code { inner } => format!("<code>{}</code>", inner.to_html()),
                ContentBlock::CodeBlock { inner } => format!("<pre>{}</pre>", inner.to_html()),
                ContentBlock::Image { src, alt } => match alt {
                    Some(alt) => format!("<img src=\"{}\" alt=\"{}\">", src, alt),
                    None => format!("<img src=\"{}\">", src),
                },
                ContentBlock::Link { href, inner } => {
                    format!("<a href=\"{}\">{}</a>", href, inner.to_html())
                }
                ContentBlock::HorizontalRule => "<hr>".to_string(),
                ContentBlock::Html { content } => content.clone(),
                ContentBlock::Markdown { content } => {
                    match { markdown::to_html_with_options(content, &markdown::Options::gfm()) } {
                        Ok(html) => html,
                        Err(_) => {
                            error!("An error occurred while rendering markdown embedded in JSON.");
                            return String::from(
                                "An error occurred while rendering this markdown.",
                            );
                        }
                    }
                }
                ContentBlock::DivBlock { inner } => todo!(),
                ContentBlock::SpanBlock { inner } => todo!(),
                ContentBlock::Button { inner } => todo!(),
            }
        }
    }
}
