use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use serde_dhall::StaticType;

#[derive(Debug, PartialEq, Serialize, Deserialize, StaticType)]
// #[serde(rename_all = "camelCase")]
pub(crate) struct CynthiaConf {
    #[serde(alias = "PORT")]
    #[serde(alias = "Port")]
    #[serde(default = "c_port")]
    pub(crate) port: u16,
    #[serde(alias = "Cache")]
    #[serde(default = "c_cache")]
    pub(crate) cache: Cache,
    #[serde(alias = "Generator")]
    #[serde(alias = "generator")]
    #[serde(alias = "Site")]
    #[serde(alias = "pages")]
    #[serde(alias = "Pages")]
    #[serde(default)]
    pub(crate) site: Site,
    #[serde(alias = "Logs")]
    #[serde(default = "c_logs")]
    pub(crate) logs: Option<Logging>,
    #[serde(alias = "Runtimes")]
    #[serde(alias = "runners")]
    #[serde(default)]
    pub(crate) runtimes: Runtimes,
    #[serde(alias = "Scenes")]
    #[serde(default = "c_emptyscenelist")]
    pub(crate) scenes: SceneCollection,
    #[serde(default = "c_plugins")]
    pub(crate) plugins: Vec<Plugin>,
}

impl Default for CynthiaConf {
    fn default() -> Self {
        CynthiaConf {
            port: c_port(),
            cache: Cache::default(),
            site: Site::default(),
            logs: c_logs(),
            scenes: c_emptyscenelist(),
            runtimes: Runtimes::default(),
            plugins: c_plugins(),
        }
    }
}

fn c_logs() -> Option<Logging> {
    Some(Logging {
        file_loglevel: Some(3),
        term_loglevel: Some(2),
        logfile: Some(String::from("cynthia.log")),
    })
}

#[cfg(feature = "js_runtime")]
pub(crate) type ExternalJavascriptRuntime = String;
#[cfg(feature = "js_runtime")]
pub(crate) trait ConfigExternalJavascriptRuntime {
    fn auto() -> ExternalJavascriptRuntime;
    fn validate(&self) -> Result<(), ()>;
}
#[derive(Debug, PartialEq, Serialize, Deserialize, StaticType, Clone)]
pub(crate) struct Runtimes {
    #[cfg(feature = "js_runtime")]
    #[serde(default = "ExternalJavascriptRuntime::auto")]
    #[serde(alias = "node")]
    pub(crate) ext_js_rt: ExternalJavascriptRuntime,
}
#[cfg(feature = "js_runtime")]
impl ConfigExternalJavascriptRuntime for ExternalJavascriptRuntime {
    fn auto() -> Self {
        let available_runtimes = (|| {
            #[cfg(windows)]
            // Deno is untested on Windows, so is not yet scanned for.
            return ["bun.exe", "node.exe"];
            #[cfg(not(windows))]
            return ["bun", "deno", "node"];
        })();
        let node = match available_runtimes.iter().find(|&runtime| {
            std::process::Command::new(runtime)
                .arg("-v")
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
        }) {
            Some(a) => *a,
            None => {
                error!("Failed to find a node runtime to use and none set. Please set a valid `node` path under `[runtimes]` in the configuration.");
                "disabled"
            }
        };
        node.to_string()
    }
    fn validate(&self) -> Result<(), ()> {
        if self == "disabled" {
            info!("Node runtime is disabled. This may cause some features to not work.");
            return Ok(());
        }
        std::process::Command::new(self)
            .arg("-v")
            .output()
            .map(|output| {
                if output.status.success() {
                    Ok(())
                } else {
                    Err(())
                }
            })
            .unwrap_or(Err(()))
    }
}
#[allow(clippy::derivable_impls)]
impl Default for Runtimes {
    fn default() -> Self {
        Runtimes {
            #[cfg(feature = "js_runtime")]
            ext_js_rt: ExternalJavascriptRuntime::auto(),
        }
    }
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
#[derive(Debug, PartialEq, Serialize, Deserialize, StaticType, Clone)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "plugin_runtime")]
pub(crate) enum Plugin {
    #[serde(rename = "javascript")]
    JsPlugin {
        plugin_name: String,
        plugin_enabled: bool,
    },
}

fn c_plugins() -> Vec<Plugin> {
    vec![Plugin::JsPlugin {
        plugin_name: "test".to_string(),
        plugin_enabled: false,
    }]
}

/// A clone of the CynthiaConf struct
pub(crate) struct CynthiaConfClone {
    pub(crate) port: u16,
    pub(crate) cache: Cache,
    pub(crate) site: Site,
    pub(crate) logs: Option<Logging>,
    pub(crate) scenes: SceneCollection,
    pub(crate) runtimes: Runtimes,
    pub(crate) plugins: Vec<Plugin>,
}

impl CynthiaConfig for CynthiaConfClone {
    fn hard_clone(&self) -> CynthiaConf {
        CynthiaConf {
            port: self.port,
            cache: self.cache.clone(),
            site: self.site.clone(),
            logs: self.logs.clone(),
            scenes: self.scenes.clone(),
            runtimes: self.runtimes.clone(),
            plugins: self.plugins.clone(),
        }
    }
    fn clone(&self) -> CynthiaConfClone {
        CynthiaConfClone {
            port: self.port,
            cache: self.cache.clone(),
            site: self.site.clone(),
            logs: self.logs.clone(),
            scenes: self.scenes.clone(),
            runtimes: self.runtimes.clone(),
            plugins: self.plugins.clone(),
        }
    }
}
impl CynthiaConfig for CynthiaConf {
    fn hard_clone(&self) -> CynthiaConf {
        CynthiaConf {
            port: self.port,
            cache: self.cache.clone(),
            site: self.site.clone(),
            logs: self.logs.clone(),
            scenes: self.scenes.clone(),
            runtimes: self.runtimes.clone(),
            plugins: self.plugins.clone(),
        }
    }
    fn clone(&self) -> CynthiaConfClone {
        CynthiaConfClone {
            port: self.port,
            cache: self.cache.clone(),
            site: self.site.clone(),
            logs: self.logs.clone(),
            scenes: self.scenes.clone(),
            runtimes: self.runtimes.clone(),
            plugins: self.plugins.clone(),
        }
    }
}
pub(crate) trait CynthiaConfig {
    fn hard_clone(&self) -> CynthiaConf;
    fn clone(&self) -> CynthiaConfClone;
}

impl CynthiaConf {
    pub(crate) fn clone(&self) -> CynthiaConfClone {
        CynthiaConfClone {
            port: self.port,
            cache: self.cache.clone(),
            site: self.site.clone(),
            logs: self.logs.clone(),
            scenes: self.scenes.clone(),
            runtimes: self.runtimes.clone(),
            plugins: self.plugins.clone(),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, StaticType)]
// #[serde(rename_all = "camelCase")]
pub(crate) struct Cache {
    pub(crate) lifetimes: Lifetimes,

    /// Maximum cache size in bytes
    /// Default: 536870912 (512MB)
    #[serde(alias = "max-cache-size")]
    #[serde(default = "c_max_cache_size")]
    pub(crate) max_cache_size: usize,
}
fn c_cache() -> Cache {
    Cache {
        max_cache_size: c_max_cache_size(),
        lifetimes: Lifetimes::default(),
    }
}
fn c_max_cache_size() -> usize {
    536870912
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, StaticType)]
// #[serde(rename_all = "camelCase")]
pub(crate) struct Lifetimes {
    #[serde(default = "c_cache_lifetime_stylesheets")]
    pub(crate) stylesheets: u64,
    #[serde(default = "c_cache_lifetime_js")]
    pub(crate) javascript: u64,
    #[serde(default = "c_cache_lifetime_external")]
    #[serde(alias = "external")]
    pub(crate) forwarded: u64,
    #[serde(default = "c_cache_lifetime_external")]
    pub(crate) assets: u64,
    #[serde(default = "c_cache_lifetime_served")]
    pub(crate) served: u64,
}
impl Default for Lifetimes {
    fn default() -> Self {
        Lifetimes {
            stylesheets: c_cache_lifetime_stylesheets(),
            javascript: c_cache_lifetime_js(),
            forwarded: c_cache_lifetime_external(),
            served: c_cache_lifetime_served(),
            assets: c_cache_lifetime_external(),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, StaticType)]
// #[serde(rename_all = "camelCase")]
pub(crate) struct Site {
    #[serde(alias = "404-page")]
    #[serde(alias = "notfound-page")]
    #[serde(default = "c_404")]
    pub(crate) notfound_page: String,

    #[serde(alias = "site-baseurl")]
    #[serde(default = "c_emptystring")]
    pub(crate) site_baseurl: String,

    #[serde(alias = "og-site-name")]
    #[serde(alias = "sitename")]
    #[serde(default = "c_emptystring")]
    pub(crate) og_sitename: String,

    pub(crate) meta: Meta,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, StaticType)]
pub(crate) struct Meta {
    #[serde(alias = "enable-tags")]
    #[serde(alias = "enableTags")]
    #[serde(default = "c_bool_false")]
    pub(crate) enable_tags: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, StaticType)]
// #[serde(rename_all = "camelCase")]
pub(crate) struct Logging {
    #[serde(alias = "file-loglevel")]
    #[serde(alias = "file-log-level")]
    pub(crate) file_loglevel: Option<u16>,
    #[serde(alias = "term-loglevel")]
    #[serde(alias = "term-log-level")]
    #[serde(alias = "console-loglevel")]
    #[serde(alias = "console-log-level")]
    pub(crate) term_loglevel: Option<u16>,

    #[serde(alias = "file")]
    #[serde(alias = "filename")]
    pub(crate) logfile: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, StaticType)]
// #[serde(rename_all = "camelCase")]
pub(crate) struct Scene {
    pub(crate) name: String,
    pub(crate) sitename: Option<String>,
    pub(crate) stylefile: Option<String>,
    pub(crate) script: Option<String>,
    pub(crate) templates: Templates,
}
impl Default for Scene {
    fn default() -> Self {
        Scene {
            name: String::from("default"),
            sitename: Some(String::from("My Cynthia Site")),
            stylefile: Some(String::from("/styles/default.css")),
            script: Some(String::from("/scripts/client.js")),
            templates: Templates {
                post: String::from("../default"),
                page: String::from("../default"),
                postlist: String::from("default"),
            },
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, StaticType)]
// #[serde(rename_all = "camelCase")]
pub(crate) struct Templates {
    pub(crate) post: String,
    pub(crate) page: String,
    pub(crate) postlist: String,
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
    vec![Scene::default()]
}
pub(crate) mod actions;
