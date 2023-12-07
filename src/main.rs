use actix_files as fs;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use colored::Colorize;
use dotenv::dotenv;
use handlebars::Handlebars;
use jsonc_parser::parse_to_serde_value;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{fs as stdfs, path::Path};

#[derive(Deserialize, Debug, Serialize)]
struct CynthiaUrlDataF {
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
fn empty_menulist () -> Vec<Menulink> {
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
struct CynthiaPostData {
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

fn logger(act: i8, msg: String) {
    /*

    Acts:
    0: Debug log, only act if logging is set to verbose
    1: General log item -- '[log]'
    2: Request that on Cynthia's part succeeded -- '[CYNGET/OK]'

    5: Error!



     */
    let spaces: usize = 15;
    let tabs: String = "\t\t".to_string();
    if act == 1 {
        let name = "[Log]".blue();
        let spaceleft = if name.chars().count() < spaces {
            spaces - name.chars().count()
        } else {
            0
        };
        let title = format!("{}", name.bold().blue());
        let preq = format!("{0}{2}{1}", title, " ".repeat(spaceleft), tabs);
        println!("{0}{1}", preq, msg);
    }
    if act == 2 {
        let name = "[CYNGET/OK] ✅";
        let spaceleft = if name.chars().count() < spaces {
            spaces - name.chars().count()
        } else {
            0
        };
        let title = format!("{}", name.bold().yellow());
        let preq = format!("{0}{2}{1}", title, " ".repeat(spaceleft), tabs);
        println!("{0}{1}", preq, msg);
    }
    if act == 5 {
        let name = "[ERROR] ✅";
        let spaceleft = if name.chars().count() < spaces {
            spaces - name.chars().count()
        } else {
            0
        };
        let title = format!("{}", name.bold().black().on_bright_yellow());
        let preq = format!("{0}{2}{1}", title, " ".repeat(spaceleft), tabs);
        println!("{0}{1}", preq, msg.red());
    }
}

fn empty_post_data_content_object() -> CynthiaPostDataContentObject {
    let n: CynthiaPostDataContentObject = CynthiaPostDataContentObject {
        markup_type: ("none".to_string()),
        data: ("none".to_string()),
        location: ("none".to_string()),
    };
    return n;
}

#[get("/p/{id:.*}")]
async fn serves_p(id: web::Path<String>) -> HttpResponse {
    logger(2, format!("--> ID: '{}' was requested.", id));
    HttpResponse::Ok().body(returns_p(&id.to_string(), format!("/p/{}",id)))
}

async fn root() -> impl Responder {
    HttpResponse::Ok().body(returns_p(&"root".to_string(), "/".to_string()))
}

fn returns_p (pgid: &String, probableurl: String) -> std::string::String {
    logger(2, format!("--> {0} ({1})", pgid, probableurl));
    return combine_content(
        pgid.to_string(),
        return_content_p(pgid.to_string()),
        generate_menus(pgid.to_string(), probableurl),
    );
}

fn read_published_jsonc() -> Vec<CynthiaPostData> {
    let file = ("./cynthiaFiles/published.jsonc").to_owned();
    let unparsed_json = stdfs::read_to_string(file).expect("Couldn't find or load that file.");
    // println!("{}", unparsed_json);
    let parsed_json: Option<serde_json::Value> =
        parse_to_serde_value(&unparsed_json.as_str(), &Default::default())
            .expect("Could not read published.jsonc.");
    // println!("{:#?}", parsed_json);
    let res: Vec<CynthiaPostData> = serde_json::from_value(parsed_json.into()).unwrap();
    return res;
}
fn load_mode(mode_name: String) -> CynthiaModeObject {
    let file = format!("./cynthiaFiles/modes/{}.jsonc", mode_name).to_owned();
    let unparsed_json = stdfs::read_to_string(file).expect("Couldn't find or load that file.");
    // println!("{}", unparsed_json);
    let parsed_json: Option<serde_json::Value> =
        parse_to_serde_value(&unparsed_json.as_str(), &Default::default())
            .expect("Could not read published.jsonc.");
    // println!("{:#?}", parsed_json);
    let res: CynthiaModeObject = serde_json::from_value(parsed_json.into()).unwrap();
    return res;
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let portnum: u16 = std::env::var("PORT")
        .expect("PORT must be set in the '.env' file.")
        .parse::<u16>()
        .unwrap();
    logger(
        1,
        format!(
            "{}",
            format!(
                "Starting server on {0}{1}",
                "http://localhost:".green(),
                portnum.to_string().bold().green()
            )
            .italic()
        ),
    );
    HttpServer::new(|| {
        App::new()
            .service(fs::Files::new("/assets", "./assets").show_files_listing())
            .service(fs::Files::new("/jquery", "./node_modules/jquery").show_files_listing())
            .service(serves_p)
            .route("/", web::get().to(root))
    })
    .bind(("127.0.0.1", portnum))?
    .run()
    .await
}

fn return_content_p(pgid: String) -> String {
    let published_jsonc = read_published_jsonc();
    for i in &published_jsonc {
        if i.id == pgid {
            let post: &CynthiaPostData = i;
            let postcontent_html: String = r#"
    <div>
        <p>No content on this page</p>
    </div>
    "#
            .to_string();
            if post.kind == "postlist".to_string() {
                return "Cynthia cannot handle post lists just yet!"
                    .to_owned()
                    .to_string();
            };
            if &post.content.location == &String::from("external") {
                return "Cynthia cannot handle external content yet!"
                    .to_owned()
                    .to_string();
            };
            return postcontent_html;
        }
    }
    return String::from("");
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct CynthiaPageVars {
    head: String,
    content: String,
    menu1: String,
    menu2: String,
    infoshow: String,
}

fn combine_content(pgid: String, content: String, menus: Menulist) -> String {
    let mut published_jsonc = read_published_jsonc();
    for post in &mut published_jsonc {
        if post.id == pgid {
            let mode_to_load = post
                .mode
                .get_or_insert_with(|| String::from("default"))
                .to_string();
            let pagemetainfojson = serde_json::to_string(&post).unwrap();
            let currentmode = load_mode(mode_to_load).1;
            let stylesheet: String = stdfs::read_to_string( Path::new("./cynthiaFiles/styles/").join(currentmode.stylefile) )
                .expect("Couldn't find or load CSS file for this pages' mode.");
            let handlebarfile = format!(
                "./cynthiaFiles/templates/{}.handlebars",
                (if post.kind == "post" {
                    currentmode.handlebar.post
                } else {
                    currentmode.handlebar.page
                })
            )
            .to_owned();
            let source = stdfs::read_to_string(handlebarfile)
                .expect("Couldn't find or load handlebars file.");
            let handlebars = Handlebars::new();
            let data = CynthiaPageVars {
                head: format!(r#"
            <style>
	{0}
	</style>
	<script src="/jquery/jquery.min.js"></script>
	<title>{1} ﹘ {2}</title>
	<script>
		const pagemetainfo = JSON.parse(\`{3}\`);
	</script>
	"#,
stylesheet,
post.title,
currentmode.sitename,
pagemetainfojson
),
                content,
                menu1: menus.menu1,
                menu2: menus.menu2,
                infoshow: String::from(""),
            };
            let k = handlebars
                .render_template(&source.to_string(), &data)
                .unwrap();
            return k;
        }
    }
    return content;
}

struct Menulist {
    menu1: String,
    menu2: String,
}

fn generate_menus(pgid: String, probableurl: String) -> Menulist {
    let mut published_jsonc = read_published_jsonc();
    for post in &mut published_jsonc {
        if post.id == pgid {
            let mode_to_load = post
                .mode
                .get_or_insert_with(|| String::from("default"))
                .to_string();
            let mode = load_mode(mode_to_load).1;
            let mut mlist1 = String::from("");
            match !mode.menulinks.is_empty() {
                true => {
                    for ele in mode.menulinks {
                        let link: String = if ele.href == probableurl {
                            format!(
                                r#"<a href="{0}" class="active">{1}</a>"#,
                                ele.href, ele.name
                            )
                        } else {
                            format!(r#"<a href="{0}" class="">{1}</a>"#, ele.href, ele.name)
                        };
                        mlist1.push_str(link.as_str());
                    }
                }
                false => (),
            }
            let mut mlist2 = String::from("");
            match !mode.menu2links.is_empty() {
                true => {
                    for ele in mode.menu2links {
                        let link: String = if ele.href == probableurl {
                            format!(
                                r#"<a href="{0}" class="active">{1}</a>"#,
                                ele.href, ele.name
                            )
                        } else {
                            format!(r#"<a href="{0}" class="">{1}</a>"#, ele.href, ele.name)
                        };
                        mlist2.push_str(link.as_str());
                    }
                }
                false => (),
            }
            let menus: Menulist = Menulist {
                menu1: String::from(mlist1),
                menu2: String::from(mlist2),
            };

            return menus;
        }
    }
    let menus: Menulist = Menulist {
        menu1: String::from(""),
        menu2: String::from(""),
    };
    return menus;
}
