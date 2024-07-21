/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */

use std::{fs, process};
use std::path::Path;

use jsonc_parser::parse_to_serde_value;
use log::error;
use serde::{Deserialize, Serialize};

pub(crate) type CynthiaPublicationList = Vec<CynthiaPublication>;
pub(crate) trait CynthiaPublicationListTrait {
    fn get_by_id(&self, id: String) -> Option<CynthiaPublication>;
}
impl CynthiaPublicationListTrait for CynthiaPublicationList {
    fn get_by_id(&self, id: String) -> Option<CynthiaPublication> {
        self.iter().find(|x| x.get_id() == id).cloned()
    }
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) enum CynthiaPublication {
    #[serde(alias = "page")]
    Page{
        id: String,
        title: String,
        short: Option<String>,
        thumbnail: Option<String>,
    },
    #[serde(alias = "post")]
    Post{
        id: String,
        title: String,
        short: Option<String>,
        thumbnail: Option<String>,
    },
    #[serde(alias = "postlist")]
    #[serde(alias = "selection")]
    #[serde(alias = "Selection")]
    PostList{
        id: String,
        title: String,
        short: Option<String>,
        thumbnail: Option<String>,
        // todo! make this an enum!
        filter: String,
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
    pub fn get_title(&self) -> String {
        match self {
            CynthiaPublication::Page { title, .. } => title.to_string(),
            CynthiaPublication::Post { title, .. } => title.to_string(),
            CynthiaPublication::PostList { title, .. } => title.to_string(),
        }
    }
    pub fn get_short(&self) -> Option<String> {
        match self {
            CynthiaPublication::Page { short, .. } => short.clone(),
            CynthiaPublication::Post { short, .. } => short.clone(),
            CynthiaPublication::PostList { short, .. } => short.clone(),
        }
    }
    pub fn get_thumbnail(&self) -> Option<String> {
        match self {
            CynthiaPublication::Page { thumbnail, .. } => thumbnail.clone(),
            CynthiaPublication::Post { thumbnail, .. } => thumbnail.clone(),
            CynthiaPublication::PostList { thumbnail, .. } => thumbnail.clone(),
        }
    }
}
pub(crate) fn read_published_jsonc() -> CynthiaPublicationList {
    if Path::new("./cynthiaFiles/published.yaml").exists() {
        let file = "./cynthiaFiles/published.yaml".to_owned();
        let unparsed_yaml = fs::read_to_string(file).expect("Couldn't find or load that file.");
        serde_yaml::from_str(&unparsed_yaml).unwrap_or_else(|_e| {
            error!("Published.yaml contains invalid Cynthia-instructions.",
            );
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
        let parsed_json: Option<serde_json::Value> =
            match parse_to_serde_value(unparsed_json.as_str(), &Default::default())
            {
                Ok(t) => t,
                Err(e) => {
                    error!("Couldn't parse published.jsonc.\n\n\t\t{e}");
                    process::exit(1);
                }

            };
        serde_json::from_value(parsed_json.into()).unwrap_or_else(|e| {
            error!(
                "Published.json contains invalid Cynthia-instructions.\n\n\t\t{e}",
            );
            Vec::new()
        })
    }
}