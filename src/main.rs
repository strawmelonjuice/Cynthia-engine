use std::fs as stdfs;
use actix_files as fs;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use colored::Colorize;
use dotenv::dotenv;
use jsonc_parser::parse_to_serde_value;
use serde::{Serialize, Deserialize};
use serde_json;
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

#[derive(Deserialize, Debug, Serialize)]
struct CynthiaUrlDataF {
    fullurl: String,
}

#[derive(Deserialize, Debug, Serialize)]
struct CynthiaPostData {
    id: String,
    title: String,
    #[serde(rename = "type")]
    kind: String,
    #[serde(default = "empty_post_data_content_object")]
    content: CynthiaPostDataContentObject
}

fn empty_post_data_content_object() -> CynthiaPostDataContentObject {
    let n: CynthiaPostDataContentObject= CynthiaPostDataContentObject { markup_type: ("none".to_string()), data: ("none".to_string()), location: ("none".to_string()) };
    return n;
}


#[derive(Deserialize, Debug, Serialize)]
struct CynthiaPostDataContentObject {
    #[serde(rename = "markupType")]
    markup_type: String,
    data: String,
    location: String
}

#[get("/p/{id:.*}")]
async fn serves_p(id: web::Path<String>) -> HttpResponse {
    logger(2, format!("--> ID: '{}' was requested.", id));
    HttpResponse::Ok().body(format!("User detail: {}", id))
}

async fn root() -> impl Responder {
    logger(2, "--> Home".to_string());
    HttpResponse::Ok().body(return_content_p("root".to_string(), "/".to_string()))
}

fn read_published_jsonc() -> Vec<CynthiaPostData> {
    let file = ("./cynthiaFiles/published.jsonc").to_owned();
    let unparsed_json = stdfs::read_to_string(file).expect("Couldn't find or load that file.");
    // println!("{}", unparsed_json);
    let parsed_json: Option<serde_json::Value> = parse_to_serde_value(&unparsed_json.as_str(), &Default::default()).expect("Could not read published.jsonc.");
    // println!("{:#?}", parsed_json);
    let res: Vec<CynthiaPostData> = serde_json::from_value(parsed_json.into()).unwrap();
    return res;
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let published: Vec<CynthiaPostData> = read_published_jsonc();
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
            .service(serves_p)
            .route("/", web::get().to(root))
    })
    .bind(("127.0.0.1", portnum))?
    .run()
    .await
}

fn return_content_p(id: String, probableurl: String) -> String {
    let mut post: &CynthiaPostData = &CynthiaPostData { id: "unknown".to_string(), title: "unknown".to_string(), kind: "unknown".to_string(), content: empty_post_data_content_object() };
    let published_jsonc = read_published_jsonc();
    for i in &published_jsonc {
        if i.id == id {
            post = i;
            break;
        } else {continue;}
    }
    let postcontent_html: String;
    if post.kind == "postlist".to_string() { 
        return "Cynthia cannot handle post lists just yet!".to_owned().to_string() 
    };
    if post.content.location == "external".to_string() {
        return "Cynthia cannot handle external content yet!".to_owned().to_string() 
    };
    return "Wait for this".to_string();
}
fn wrap_content(post: CynthiaPostData, content: String) {}
