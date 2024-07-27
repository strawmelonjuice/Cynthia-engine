use log::{error, warn};
use serde::{Deserialize, Serialize};
#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CynthiaConf {
    #[serde(alias = "PORT")]
    #[serde(alias = "Port")]
    #[serde(default = "c_port")]
    pub port: u16,
    #[serde(alias = "Cache")]
    #[serde(default)]
    pub cache: Cache,
    #[serde(alias = "Pages")]
    #[serde(default)]
    pub pages: Pages,
    #[serde(alias = "Generator")]
    #[serde(default)]
    pub generator: Generator,
    #[serde(alias = "Logs")]
    pub logs: Option<Logging>,
    #[serde(alias = "Scenes")]
    #[serde(default = "c_emptyscenelist")]
    pub scenes: SceneCollection,
}
pub(crate) type SceneCollection = Vec<Scene>;
pub(crate) trait SceneCollectionTrait {
    fn get_by_name(&self, name: &str) -> Option<Scene>;
    fn get_default(&self) -> Scene;
    fn validate(&self) -> bool;
}
impl Scene {
    pub(crate) fn get_name(&self) -> String {
        self.name.to_string()
    }
}
impl SceneCollectionTrait for SceneCollection {
    fn get_by_name(&self, name: &str) -> Option<Scene> {
        for scene in self {
            if scene.get_name() == name {
                return Some(scene.clone());
            }
        }
        None
    }
    fn get_default(&self) -> Scene {
        for scene in self {
            if scene.get_name() == "default" {
                return scene.clone();
            }
        }
        if self.is_empty() {
            warn!("No scenes found in the configuration file, making up a default scene.");
            return Scene::default();
        }
        self[0].clone()
    }
    fn validate(&self) -> bool {
        if self.is_empty() {
            error!("No scenes found in the configuration file");
            return false;
        }
        true
    }
}

/// A clone of the CynthiaConf struct
pub struct CynthiaConfClone {
    pub port: u16,
    pub cache: Cache,
    pub pages: Pages,
    pub generator: Generator,
    pub logs: Option<Logging>,
    pub scenes: SceneCollection,
}

impl CynthiaConfig for CynthiaConfClone {
    fn hard_clone(&self) -> CynthiaConf {
        CynthiaConf {
            port: self.port,
            cache: self.cache.clone(),
            pages: self.pages.clone(),
            generator: self.generator.clone(),
            logs: self.logs.clone(),
            scenes: self.scenes.clone(),
        }
    }
    fn clone(&self) -> CynthiaConfClone {
        CynthiaConfClone {
            port: self.port,
            cache: self.cache.clone(),
            pages: self.pages.clone(),
            generator: self.generator.clone(),
            logs: self.logs.clone(),
            scenes: self.scenes.clone(),
        }
    }
}
impl CynthiaConfig for CynthiaConf {
    fn clone(&self) -> CynthiaConfClone {
        CynthiaConfClone {
            port: self.port,
            cache: self.cache.clone(),
            pages: self.pages.clone(),
            generator: self.generator.clone(),
            logs: self.logs.clone(),
            scenes: self.scenes.clone(),
        }
    }
    fn hard_clone(&self) -> CynthiaConf {
        CynthiaConf {
            port: self.port,
            cache: self.cache.clone(),
            pages: self.pages.clone(),
            generator: self.generator.clone(),
            logs: self.logs.clone(),
            scenes: self.scenes.clone(),
        }
    }
}
#[allow(unused)]
pub trait CynthiaConfig {
    fn hard_clone(&self) -> CynthiaConf;
    fn clone(&self) -> CynthiaConfClone;
}

impl CynthiaConf {
    pub fn clone(&self) -> CynthiaConfClone {
        CynthiaConfClone {
            port: self.port,
            cache: self.cache.clone(),
            pages: self.pages.clone(),
            generator: self.generator.clone(),
            logs: self.logs.clone(),
            scenes: self.scenes.clone(),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cache {
    pub lifetimes: Lifetimes,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pages {
    #[serde(alias = "404-page")]
    #[serde(alias = "notfound-page")]
    #[serde(default = "c_404")]
    pub notfound_page: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lifetimes {
    #[serde(default = "c_cache_lifetime_stylesheets")]
    pub stylesheets: u64,
    #[serde(default = "c_cache_lifetime_js")]
    pub javascript: u64,
    #[serde(default = "c_cache_lifetime_external")]
    #[serde(alias = "external")]
    pub forwarded: u64,
    #[serde(default = "c_cache_lifetime_served")]
    pub served: u64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Generator {
    #[serde(alias = "site-baseurl")]
    #[serde(default = "c_emptystring")]
    pub site_baseurl: String,

    #[serde(alias = "og-site-name")]
    #[serde(alias = "sitename")]
    #[serde(default = "c_emptystring")]
    pub og_sitename: String,

    pub meta: Meta,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    #[serde(alias = "enable-tags")]
    #[serde(default = "c_bool_false")]
    pub enable_tags: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Logging {
    #[serde(alias = "file-loglevel")]
    #[serde(alias = "file-log-level")]
    pub file_loglevel: Option<u8>,
    #[serde(alias = "term-loglevel")]
    #[serde(alias = "term-log-level")]
    #[serde(alias = "console-loglevel")]
    #[serde(alias = "console-log-level")]
    pub term_loglevel: Option<u8>,

    #[serde(alias = "file")]
    #[serde(alias = "filename")]
    pub logfile: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scene {
    pub name: String,
    pub sitename: Option<String>,
    pub stylefile: Option<String>,
    pub templates: Templates,
}
impl Default for Scene {
    fn default() -> Self {
        Scene {
            name: String::from("default"),
            sitename: Some(String::from("My Cynthia Site")),
            stylefile: None,
            templates: Templates {
                post: String::from("post"),
                page: String::from("page"),
                postlist: String::from("postlist"),
            },
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Templates {
    pub post: String,
    pub page: String,
    pub postlist: String,
}

fn c_port() -> u16 {
    3000
}
fn c_bool_false() -> bool {
    false
}
fn c_emptystring() -> String {
    String::from("")
}
fn c_cache_lifetime_stylesheets() -> u64 {
    72000
}
fn c_cache_lifetime_js() -> u64 {
    1200
}
fn c_cache_lifetime_external() -> u64 {
    1600
}
fn c_cache_lifetime_served() -> u64 {
    50
}

fn c_404() -> String {
    String::from("404")
}
fn c_emptyscenelist() -> Vec<Scene> {
    Vec::new()
}
