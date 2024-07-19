/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */

use crate::logger;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::{fs, process};

pub fn main() -> CynthiaConf {
    match fs::read_to_string(Path::new("./cynthia.toml")) {
        Ok(g) => match toml::from_str(&g) {
            Ok(p) => p,
            Err(_e) => {
                logger::fatal_error(
                    "Could not interpret cynthia-configuration at `./cynthia.toml`!".to_string(),
                );
                process::exit(1);
            }
        },
        Err(_) => {
            logger::fatal_error(
                "Could not interpret cynthia-configuration at `./cynthia.toml`!".to_string(),
            );
            process::exit(1);
        }
    }
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    #[serde(alias = "Logging")]
    #[serde(default = "d_logging")]
    pub logging: Logging,
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
    #[serde(default = "d_file_logging")]
    pub(crate) file: FileLogging,

    #[serde(default = "d_console_logging")]
    pub(crate) console: ConsoleLogging,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileLogging {
    #[serde(alias = "filename")]
    #[serde(default = "c_l_filep")]
    pub filepath: String,
    #[serde(default = "c_bool_true")]
    pub enabled: bool,
    #[serde(default = "c_bool_true")]
    pub cache: bool,
    #[serde(default = "c_bool_true")]
    pub error: bool,
    #[serde(default = "c_bool_true")]
    pub warn: bool,
    #[serde(default = "c_bool_false")]
    pub info: bool,
    #[serde(default = "c_bool_true")]
    pub requests: bool,
    #[serde(alias = "proxy-requests")]
    #[serde(default = "c_bool_false")]
    pub proxy_requests: bool,
    #[serde(alias = "plugin-asset-requests")]
    #[serde(default = "c_bool_false")]
    pub plugin_asset_requests: bool,
    #[serde(alias = "jsr-errors")]
    #[serde(default = "c_bool_true")]
    pub jsr_errors: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleLogging {
    #[serde(default = "c_bool_true")]
    pub enabled: bool,
    #[serde(default = "c_bool_false")]
    pub cache: bool,
    #[serde(default = "c_bool_true")]
    pub error: bool,
    #[serde(default = "c_bool_true")]
    pub warn: bool,
    #[serde(default = "c_bool_true")]
    pub info: bool,
    #[serde(default = "c_bool_true")]
    pub requests: bool,
    #[serde(alias = "proxy-requests")]
    #[serde(default = "c_bool_false")]
    pub proxy_requests: bool,
    #[serde(alias = "plugin-asset-requests")]
    #[serde(default = "c_bool_false")]
    pub plugin_asset_requests: bool,
    #[serde(alias = "jsr-errors")]
    #[serde(default = "c_bool_true")]
    pub jsr_errors: bool,
}
fn c_port() -> u16 {
    3000
}
fn c_bool_false() -> bool {
    false
}
fn c_bool_true() -> bool {
    true
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

fn c_l_filep() -> String {
    String::from("./cynthia.log")
}

fn d_logging() -> Logging {
    Logging {
        file: d_file_logging(),
        console: d_console_logging(),
    }
}
fn d_file_logging() -> FileLogging {
    FileLogging {
        filepath: "./cynthia.log".to_string(),
        enabled: true,
        cache: true,
        error: true,
        warn: true,
        info: false,
        requests: true,
        proxy_requests: false,
        plugin_asset_requests: false,
        jsr_errors: true,
    }
}
fn d_console_logging() -> ConsoleLogging {
    ConsoleLogging {
        enabled: true,
        cache: false,
        error: true,
        warn: true,
        info: true,
        requests: true,
        proxy_requests: false,
        plugin_asset_requests: false,
        jsr_errors: true,
    }
}
