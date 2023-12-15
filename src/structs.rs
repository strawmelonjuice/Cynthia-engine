use serde::{Deserialize, Serialize};


#[derive(Deserialize, Debug, Serialize)]
pub(crate) struct CynthiaPluginRepoItem {
    pub(crate) id: String,
    pub(crate) host: String,
    pub(crate) referrer: String,
}

#[derive(Deserialize, Debug, Serialize)]
pub(crate) struct CynthiaPluginManifestItem {
    pub(crate) id: String,
    pub(crate) version: String,
}


#[derive(Deserialize, Debug, Serialize)]
pub(crate) struct CynthiaUrlDataF {
    pub fullurl: String,
}

pub(crate) type CynthiaModeObject = (String, Config);

pub(crate) fn empty_post_data_content_object() -> CynthiaPostDataContentObject {
    CynthiaPostDataContentObject {
        markup_type: "none".to_string(),
        data: "none".to_string(),
        location: "none".to_string(),
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Config {
    pub sitename: String,
    pub stylefile: String,
    pub handlebar: Handlebar,
    #[serde(default = "empty_menulist")]
    pub menulinks: Vec<Menulink>,
    #[serde(default = "empty_menulist")]
    pub menu2links: Vec<Menulink>,
}

fn empty_menulist() -> Vec<Menulink> {
    Vec::new()
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Handlebar {
    pub post: String,
    pub page: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Menulink {
    pub name: String,
    pub href: String,
}

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

#[derive(Deserialize, Debug, Serialize)]
pub(crate) struct CynthiaPostData {
    pub id: String,
    pub title: String,
    pub short: Option<String>,
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
    filters: Option<PostListFilter>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostListFilter {
    pub category: Option<String>,
    pub tag: Option<String>,
    pub searchline: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PluginMeta {
    #[serde(rename = "CyntiaPluginCompat")]
    pub cyntia_plugin_compat: String,
    pub runners: PluginRunners,
    #[serde(default = "nonestring")]
    pub name: String,
}

fn nonestring() -> String {
    String::from("none")
}

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
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ModifyBodyHtml {
    #[serde(rename = "type")]
    pub type_field: String,
    pub execute: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PluginChildExecute {
    #[serde(rename = "type")]
    pub type_field: String,
    pub execute: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ModifyHeadHtml {
    #[serde(rename = "type")]
    pub type_field: String,
    pub execute: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ModifyOutputHtml {
    #[serde(rename = "type")]
    pub type_field: String,
    pub execute: String,
}
