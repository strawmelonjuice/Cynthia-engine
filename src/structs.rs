/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */

use crate::config::CynthiaConf;
use serde::{Deserialize, Serialize};

// Serde allows using the output of a function to replace incoherent data. This is why there are
// some private functions in this file, that just create empty objects or structs.

// Cache index, used in cache-indexes saved on disk as JSON.
#[derive(Deserialize, Debug, Serialize, Clone)]
pub(crate) struct CynthiaCacheIndexObject {
    pub(crate) fileid: String,
    pub(crate) cachepath: std::path::PathBuf,
    pub(crate) timestamp: u64,
}
// Plugin information as stored in the Cynthia Plugin Repo
#[derive(Deserialize, Debug, Serialize)]
pub(crate) struct CynthiaPluginRepoItem {
    pub(crate) id: String,
    pub(crate) host: String,
    pub(crate) referrer: String,
}

// Plugin information on plugins Cynthia should install from the cynthiapluginmanifest file.
#[derive(Deserialize, Debug, Serialize)]
pub(crate) struct CynthiaPluginManifestItem {
    pub(crate) id: String,
    pub(crate) version: String,
}

// Reserved for usage by F-type servers to keep track of URL data.
#[derive(Deserialize, Debug, Serialize)]
pub(crate) struct CynthiaUrlDataF {
    pub fullurl: String,
}

// Cynthia Mode Objects resemble the structure of a Cynthia mode.jsonc file: An array with the name of the mode in 0, and data for it in 1.
pub(crate) type CynthiaModeObject = (String, ModeConfig);

// Function to initialise empty post data objects
pub(crate) fn empty_post_data_content_object() -> CynthiaPostDataContentObject {
    CynthiaPostDataContentObject {
        markup_type: "none".to_string(),
        data: "none".to_string(),
        location: "none".to_string(),
    }
}

// Containing data belonging to a Cynthia page-"mode".
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ModeConfig {
    pub sitename: String,
    pub favicon: Option<String>,
    pub stylefile: String,
    pub handlebar: Handlebar,
    #[serde(default = "empty_menulist")]
    pub menulinks: Vec<Menulink>,
    #[serde(default = "empty_menulist")]
    pub menu2links: Vec<Menulink>,
    pub pageinfooverride: Option<bool>,
}

// Creates an empty vec and sends it out as Vec<Menulink>
fn empty_menulist() -> Vec<Menulink> {
    Vec::new()
}

// Allows setting Handlebar file per Cynthia-page-"mode" per pagetype
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Handlebar {
    pub post: String,
    pub page: String,
}

// Menulinks that'll be send to the client in an Array, allowing it to generate navigation menus.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Menulink {
    pub name: String,
    pub href: String,
}

// Variables as sent to the web page.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct CynthiaPageVars {
    pub head: String,
    pub content: String,
    pub menu1: String,
    pub menu2: String,
    pub infoshow: String,
}

pub(crate) struct Menulist {
    pub menu1: String,
    pub menu2: String,
}

// A list of metadata necessary for the serving of publishments
#[derive(Deserialize, Debug, Serialize)]
pub(crate) struct CynthiaContentMetaData {
    pub id: String,
    pub title: String,
    pub short: Option<String>,
    pub thumbnail: Option<String>,
    pub author: Option<Author>,
    #[serde(default = "empty_post_data_content_object")]
    pub content: CynthiaPostDataContentObject,
    pub dates: Option<Dates>,
    #[serde(rename = "type")]
    pub kind: String,
    pub mode: Option<String>,
    pub category: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub postlist: Option<Postlist>,
    pub pageinfooverride: Option<bool>,
}

// CynthiaContentMetaData but without stuff that doesn't need to be served again, or that would cause troubles otherwise.
#[derive(Deserialize, Debug, Serialize)]
pub(crate) struct CynthiaContentMetaDataMinimal {
    pub id: String,
    pub title: String,
    pub short: Option<String>,
    pub thumbnail: Option<String>,
    pub author: Option<Author>,
    pub dates: Option<Dates>,
    #[serde(rename = "type")]
    pub kind: String,
    pub mode: Option<String>,
    pub category: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub pageinfooverride: Option<bool>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Author {
    pub name: String,
    pub thumbnail: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CynthiaPostDataContentObject {
    pub markup_type: String,
    pub location: String,
    pub data: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Dates {
    pub published: i64,
    pub altered: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Postlist {
    pub filters: Option<PostListFilter>,
}

// Metadata allowing to filter out publications based on certain variables.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostListFilter {
    pub category: Option<String>,
    pub tag: Option<String>,
    pub searchline: Option<String>,
}

// Metadata necessary to run plugins.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PluginMeta {
    #[serde(rename = "CyntiaPluginCompat")]
    pub cyntia_plugin_compat: String,
    pub runners: PluginRunners,
    #[serde(default = "nonestring")]
    pub name: String,
}

// Creates empty string
fn nonestring() -> String {
    String::from("none")
}

// Plugin runners, tell the plugin executor (JSR, PYR or binairy mode) what to execute and when.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PluginRunners {
    #[serde(rename = "modifyBodyHTML")]
    pub modify_body_html: Option<ModifyBodyHtml>,
    #[serde(rename = "modifyHeadHTML")]
    pub modify_head_html: Option<ModifyHeadHtml>,
    #[serde(rename = "modifyOutputHTML")]
    pub modify_output_html: Option<ModifyOutputHtml>,
    #[serde(rename = "pluginChildExecute")]
    pub plugin_children: Option<PluginChildExecute>,
    pub hostedfolders: Option<Vec<Vec<String>>>,
    pub proxied: Option<Vec<Vec<String>>>,
}

// What should run over the HTML body after being created by Cynthia?
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ModifyBodyHtml {
    #[serde(rename = "type")]
    pub type_field: String,
    pub execute: String,
}

// What child processes should Cynthia have running
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PluginChildExecute {
    #[serde(rename = "type")]
    pub type_field: String,
    pub execute: String,
}

// What should run over the HTML head after being created by Cynthia?
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ModifyHeadHtml {
    #[serde(rename = "type")]
    pub type_field: String,
    pub execute: String,
}

// What should run over HTML after being compiled by Cynthia?
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ModifyOutputHtml {
    #[serde(rename = "type")]
    pub type_field: String,
    pub execute: String,
}

pub(crate) struct LoadedData {
    pub plugins: Vec<PluginMeta>,
    pub config: CynthiaConf,
}
