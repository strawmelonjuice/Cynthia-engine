use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize)]
pub(crate) struct CynthiaUrlDataF {
    fullurl: String,
}

pub type CynthiaModeObject = (String, Config);

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub sitename: String,
    pub stylefile: String,
    pub handlebar: Handlebar,
    #[serde(default = "empty_menulist")]
    pub menulinks: Vec<Menulink>,
    #[serde(default = "empty_menulist")]
    pub menu2links: Vec<Menulink>,
}
fn empty_menulist() -> Vec<Menulink> {
    let hi: Vec<Menulink> = Vec::new();
    return hi;
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Handlebar {
    pub post: String,
    pub page: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Menulink {
    pub name: String,
    pub href: String,
}

#[derive(Deserialize, Debug, Serialize)]
pub(crate) struct CynthiaPostData {
    pub id: String,
    pub title: String,
    pub short: Option<String>,
    pub author: Option<Author>,
    #[serde(default = "crate::empty_post_data_content_object")]
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
pub struct Author {
    pub name: String,
    pub thumbnail: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CynthiaPostDataContentObject {
    pub markup_type: String,
    pub location: String,
    pub data: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dates {
    pub published: i64,
    pub altered: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Postlist {}