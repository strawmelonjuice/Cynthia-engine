/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */

use std::path::Path;
use std::{fs, process};

use jsonc_parser::parse_to_serde_value as preparse_jsonc;
use log::{error, warn};
use serde::{Deserialize, Serialize};

use crate::config::{CynthiaConfClone, CynthiaConfig};

pub(crate) type CynthiaPublicationList = Vec<CynthiaPublication>;
pub(crate) trait PostLists {
    fn filter(&self, filter: PostListFilter) -> Vec<PostPublication>;
    #[allow(dead_code)]
    fn get_by_id(&self, id: String) -> Option<CynthiaPublication>;
}
impl PostLists for CynthiaPostList {
    fn filter(&self, filter: PostListFilter) -> Vec<PostPublication> {
        match filter {
            PostListFilter::Latest => {
                let mut p = self.clone();
                p.sort_by(|a, b| b.dates.published.cmp(&a.dates.published));
                p
            }
            PostListFilter::Oldest => {
                let mut p = self.clone();
                p.sort_by(|a, b| a.dates.published.cmp(&b.dates.published));
                p
            }
            PostListFilter::Tag(tag) => self
                .iter()
                .filter(|x| x.tags.contains(&tag))
                .cloned()
                .collect(),
            PostListFilter::Category(category) => self
                .iter()
                .filter(|x| x.category == Some(category.clone()))
                .cloned()
                .collect(),
            PostListFilter::Author(author) => self
                .iter()
                .filter(|x| {
                    x.author
                        .as_ref()
                        .map_or(false, |a| a.name == Some(author.clone()))
                })
                .cloned()
                .collect(),
            PostListFilter::Search(search) => self
                .iter()
                .filter(|x| {
                    x.title.contains(&search)
                        || x.short.as_ref().map_or(false, |s| s.contains(&search))
                    // || x.postcontent.get_inner().contains(&search)
                })
                .cloned()
                .collect(),
        }
    }
    fn get_by_id(&self, id: String) -> Option<CynthiaPublication> {
        let mut a: Option<CynthiaPublication> = None;
        for i in self {
            if i.id == id {
                a = Some(CynthiaPublication::Post {
                    id: i.id.to_string(),
                    title: i.title.to_string(),
                    short: i.short.clone(),
                    dates: i.dates.clone(),
                    thumbnail: i.thumbnail.clone(),
                    category: i.category.clone(),
                    tags: i.tags.clone(),
                    author: i.author.clone(),
                    postcontent: i.postcontent.clone(),
                    scene_override: i.scene_override.clone(),
                })
            }
        }
        a
    }
}
pub(crate) type CynthiaPostList = Vec<PostPublication>;
pub(crate) trait CynthiaPublicationListTrait {
    fn only_posts(&self) -> CynthiaPostList;
    fn get_notfound(&self, config: CynthiaConfClone) -> Option<CynthiaPublication>;
    fn get_root(&self) -> Option<CynthiaPublication>;
    fn get_by_id(&self, id: String) -> Option<CynthiaPublication>;
    fn validate(&self, config: CynthiaConfClone) -> bool;
    fn load() -> CynthiaPublicationList;
}
impl CynthiaPublicationListTrait for CynthiaPublicationList {
    fn only_posts(&self) -> CynthiaPostList {
        let mut p = Vec::new();
        for i in self {
            if let CynthiaPublication::Post {
                    id,
                    title,
                    short,
                    dates,
                    thumbnail,
                    category,
                    tags,
                    author,
                    postcontent,
                    scene_override,
                } = i {
                p.push(PostPublication {
                    id: id.to_string(),
                    title: title.to_string(),
                    short: short.clone(),
                    dates: dates.clone(),
                    thumbnail: thumbnail.clone(),
                    category: category.clone(),
                    tags: tags.clone(),
                    author: author.clone(),
                    postcontent: postcontent.clone(),
                    scene_override: scene_override.clone(),
                });
            }
        }
        p
    }
    fn get_notfound(&self, config: CynthiaConfClone) -> Option<CynthiaPublication> {
        self.iter()
            .find(|x| {
                let notfound = config.clone().site.notfound_page;
                if x.get_id() == notfound {
                    match x {
                        CynthiaPublication::Page { .. } => true,
                        _ => {
                            warn!("Page id reserved for notfound responses ({notfound}), was not a page.");
                            false
                        }
                    }
                } else {

                match x.get_id().as_str() {
                    "404" | "notfound" => match x {
                        CynthiaPublication::Page { .. } => true,
                        _ => {
                            warn!("Page id reserved for notfound responses (\"404\", \"notfound\"), was not a page.");
                            false
                        }
                    },
                    _ => false,
                }
                 }
            })
            .cloned()
    }
    fn get_root(&self) -> Option<CynthiaPublication> {
        self.iter()
            .find(|x| match x.get_id().as_str() {
                "root" | "" | "/" => match x {
                    CynthiaPublication::Page { .. } => true,
                    _ => {
                        warn!("Page using a reserved id for root pages (\"/\", \"root\" or \"\") was not a page.");
                        false
                },
                },
                _ => false,
            })
            .cloned()
    }
    fn get_by_id(&self, id: String) -> Option<CynthiaPublication> {
        self.iter().find(|x| x.get_id() == id).cloned()
    }
    fn validate(&self, config: CynthiaConfClone) -> bool {
        // Collect validation results in a vector
        let mut valid: Vec<bool> = vec![];

        // Check for duplicate ids
        let mut ids: Vec<String> = vec![];
        let duplication = self.iter().all(|x| {
            let id = x.get_id();
            if ids.contains(&id) {
                error!("Duplicate id found in published.jsonc: {}", id);
                false
            } else {
                ids.push(id);
                true
            }
        });
        valid.push(duplication);
        // Checking for required pages:
        // - 404 page
        let notfound_exists = self.get_notfound(config).is_some();
        if !notfound_exists {
            error!("404 page not found in published.jsonc: Add a page with id being either \"404\" or \"notfound\" or the id specified in the config.");
        }
        valid.push(notfound_exists);

        // - Root page
        let root_exists = self.get_root().is_some();
        if !root_exists {
            error!("Root page not found in published.jsonc: Add a page with id being either \"root\" or \"/\"");
        }
        valid.push(root_exists);

        // An empty list is not valid
        let itemsin = if self.is_empty() {
            error!("No correct publications found in publication list.");
            false
        } else {
            true
        };
        valid.push(itemsin);

        // Return true if all checks passed
        valid.iter().all(|x| *x)
    }
    fn load() -> CynthiaPublicationList {
        if Path::new("./cynthiaFiles/published.yaml").exists() {
            let file = "./cynthiaFiles/published.yaml".to_owned();
            let unparsed_yaml = fs::read_to_string(file).expect("Couldn't find or load that file.");
            serde_yaml::from_str(&unparsed_yaml).unwrap_or_else(|_e| {
                error!("Published.yaml contains invalid Cynthia-instructions.",);
                Vec::new()
            })
        } else {
            let file = "./cynthiaFiles/published.jsonc".to_owned();
            let unparsed_json = match fs::read_to_string(file) {
                Ok(t) => t,
                Err(e) => {
                    error!("Couldn't find or load published.jsonc.\n\n\t\t{e}");
                    process::exit(1);
                }
            };
            // println!("{}", unparsed_json);
            let preparsed: Option<serde_json::Value> =
                match preparse_jsonc(unparsed_json.as_str(), &Default::default()) {
                    Ok(t) => t,
                    Err(e) => {
                        error!("Couldn't parse published.jsonc.\n\n\t\t{e}");
                        process::exit(1);
                    }
                };
            serde_json::from_value(preparsed.into()).unwrap_or_else(|e| {
                let k = e.line();
                error!("Published.json contains invalid Cynthia-instructions.\n\n\t\t{e}, {k}",);
                Vec::new()
            })
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct PostPublication {
    id: String,
    title: String,
    short: Option<String>,
    dates: CynthiaPublicationDates,
    thumbnail: Option<String>,
    category: Option<String>,
    tags: Vec<String>,
    author: Option<Author>,
    postcontent: PublicationContent,
    scene_override: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) enum CynthiaPublication {
    #[serde(alias = "page")]
    Page {
        id: String,
        title: String,
        description: Option<String>,
        thumbnail: Option<String>,
        dates: CynthiaPublicationDates,
        #[serde(alias = "content")]
        pagecontent: PublicationContent,
        #[serde(alias = "scene")]
        #[serde(alias = "scene-override")]
        scene_override: Option<String>,
    },
    #[serde(alias = "post")]
    Post {
        id: String,
        title: String,
        #[serde(alias = "description")]
        short: Option<String>,
        dates: CynthiaPublicationDates,
        thumbnail: Option<String>,
        category: Option<String>,
        tags: Vec<String>,
        author: Option<Author>,
        #[serde(alias = "content")]
        postcontent: PublicationContent,
        #[serde(alias = "scene")]
        #[serde(alias = "scene-override")]
        scene_override: Option<String>,
    },
    #[serde(alias = "postlist")]
    #[serde(alias = "selection")]
    #[serde(alias = "Selection")]
    PostList {
        id: String,
        title: String,
        #[serde(alias = "description")]
        short: Option<String>,
        filter: PostListFilter,
        #[serde(alias = "scene")]
        #[serde(alias = "scene-override")]
        scene_override: Option<String>,
    },
}
impl CynthiaPublication {
    pub fn get_id(&self) -> String {
        match self {
            CynthiaPublication::Page { id, .. } => id.to_string(),
            CynthiaPublication::Post { id, .. } => id.to_string(),
            CynthiaPublication::PostList { id, .. } => id.to_string(),
        }
    }

    pub(crate) fn get_scene_name(&self) -> Option<String> {
        match self {
            CynthiaPublication::Page { scene_override, .. } => scene_override.clone(),
            CynthiaPublication::Post { scene_override, .. } => scene_override.clone(),
            CynthiaPublication::PostList { scene_override, .. } => scene_override.clone(),
        }
    }
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct CynthiaPublicationDates {
    pub(crate) altered: u64,
    pub(crate) published: u64,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) enum PostListFilter {
    #[default]
    #[serde(alias = "latest")]
    Latest,
    #[serde(alias = "oldest")]
    Oldest,
    #[serde(alias = "tag")]
    Tag(String),
    #[serde(alias = "category")]
    Category(String),
    #[serde(alias = "author")]
    Author(String),
    #[serde(alias = "search")]
    Search(String),
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) enum PublicationContent {
    #[serde(alias = "inline")]
    Inline(ContentType),
    #[serde(alias = "external")]
    External { source: ContentType },
    #[serde(alias = "local")]
    Local { source: ContentType },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "as", content = "value")]
pub(crate) enum ContentType {
    #[serde(alias = "html")]
    Html(String),
    #[serde(alias = "markdown")]
    #[serde(alias = "MarkDown")]
    #[serde(alias = "md")]
    #[serde(alias = "MD")]
    Markdown(String),
    #[serde(alias = "plaintext")]
    #[serde(alias = "text")]
    PlainText(String),
}
impl ContentType {
    pub fn get_inner(&self) -> String {
        match self {
            ContentType::Html(c) => c.to_string(),
            ContentType::Markdown(c) => c.to_string(),
            ContentType::PlainText(c) => c.to_string(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Author {
    pub(crate) name: Option<String>,
    pub(crate) thumbnail: Option<String>,
    pub(crate) link: Option<String>,
}
