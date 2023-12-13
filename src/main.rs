use std::sync::Mutex;

use actix_web::{
    get,
    http::header::ContentType,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use colored::Colorize;
use dotenv::dotenv;
use init::init;
use jsonc_parser::parse_to_serde_value;
use mime::Mime;
mod init;
mod structs;
use structs::*;
mod logger;
use crate::logger::logger;
mod jsr;
mod contentservers;

const CYNTHIAPLUGINCOMPAT: &str = "2";

fn empty_post_data_content_object() -> CynthiaPostDataContentObject {
    let n: CynthiaPostDataContentObject = CynthiaPostDataContentObject {
        markup_type: ("none".to_string()),
        data: ("none".to_string()),
        location: ("none".to_string()),
    };
    return n;
}

#[get("/p/{id:.*}")]
async fn serves_p(id: web::Path<String>, pluginsmex: Data<Mutex<Vec<PluginMeta>>>) -> HttpResponse {
    let plugins: Vec<PluginMeta> = pluginsmex.lock().unwrap().clone();
    let s = id.as_str();
    let pgid = if s.ends_with("/") {
        &s[..s.len() - "/".len()]
    } else {
        s
    };
    return contentservers::p_server(&pgid.to_string(), format!("/p/{}", id), plugins);
}
fn find_mimetype(filename_: &String) -> Mime {
    let filename = filename_.replace("\"", "");
    let parts: Vec<&str> = filename.split('.').collect();

    let res = match parts.last() {
        Some(v) => match *v {
            "png" => mime::IMAGE_PNG,
            "jpg" => mime::IMAGE_JPEG,
            "json" => mime::APPLICATION_JSON,
            "js" => mime::TEXT_JAVASCRIPT,
            &_ => mime::TEXT_PLAIN,
        },
        None => mime::TEXT_PLAIN,
    };
    // println!("{filename}: {res}");
    return res;
}
#[get("/e/{id:.*}")]
async fn serves_e(id: web::Path<String>, pluginsmex: Data<Mutex<Vec<PluginMeta>>>) -> HttpResponse {
    let plugins: Vec<PluginMeta> = pluginsmex.lock().unwrap().clone();
    let mut body = String::new();
    let mut mime = find_mimetype(&String::from("hello.html"));
    for plugin in plugins {
        match &plugin.runners.hostedfolders {
            Some(p) => {
                for s in p {
                    let z = format!("{}/", s[1]);
                    let l = match s[1].ends_with("/") {
                        true => &s[1],
                        false => &z,
                    };
                    if id.starts_with(&*l) {
                        let fid = id.replace(&*l, "");
                        let fileb = format!("./plugins/{}/{}/{fid}", plugin.name, s[0]);
                        let file = std::path::Path::new(&fileb);
                        mime = find_mimetype(&format!("{:?}", file.file_name().unwrap()));
                        body = std::fs::read_to_string(file)
                            .unwrap_or(String::from("Couldn't serve file."));
                    };
                }
            }
            None => {}
        }
    }

    return HttpResponse::Ok()
        .append_header(ContentType(mime))
        .body(body);
}
async fn root(pluginsmex: Data<Mutex<Vec<PluginMeta>>>) -> impl Responder {
    let plugins: Vec<PluginMeta> = pluginsmex.lock().unwrap().clone();
    return contentservers::p_server(&"root".to_string(), "/".to_string(), plugins);
}



fn read_published_jsonc() -> Vec<CynthiaPostData> {
    let file = ("./cynthiaFiles/published.jsonc").to_owned();
    let unparsed_json = std::fs::read_to_string(file).expect("Couldn't find or load that file.");
    // println!("{}", unparsed_json);
    let parsed_json: Option<serde_json::Value> =
        parse_to_serde_value(&unparsed_json.as_str(), &Default::default())
            .expect("Could not read published.jsonc.");
    let res: Vec<CynthiaPostData> = serde_json::from_value(parsed_json.into()).unwrap();
    return res;
}
fn load_mode(mode_name: String) -> CynthiaModeObject {
    let file = format!("./cynthiaFiles/modes/{}.jsonc", mode_name).to_owned();
    let unparsed_json = std::fs::read_to_string(file).expect("Couldn't find or load that file.");
    // println!("{}", unparsed_json);
    let parsed_json: Option<serde_json::Value> =
        parse_to_serde_value(&unparsed_json.as_str(), &Default::default())
            .expect("Could not read published.jsonc.");
    let res: CynthiaModeObject = serde_json::from_value(parsed_json.into()).unwrap();
    return res;
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!(
        "{} - version {}\n by {}{}{} {}!",
        "CynthiaCMS".bold().bright_purple(),
        env!("CARGO_PKG_VERSION").to_string().green(),
        "Straw".bright_red(),
        "melon".green(),
        "juice".bright_yellow(),
        "Mar".magenta()
    );
    if std::env::args().nth(1).unwrap_or(String::from("")) == *"init" {
        init();
    }
    dotenv().ok();
    let portnum: u16 = std::env::var("PORT")
        .expect("PORT must be set in the '.env' file.")
        .parse::<u16>()
        .unwrap();
    logger(
        1,
        format!(
            "Starting server on {} ...",
            format!(
                "http://{}:{}/",
                "localhost".green(),
                portnum.to_string().bold().green()
            ).yellow()
            .italic()
        ),
    );
    match jsr::jsruntime(true) {
        "" => logger(5, String::from("No JS runtime found! Cynthia doesn't need one, but most of it's plugins do!\n\nSee: <https://github.com/strawmelonjuice/CynthiaCMS/blob/rust/docs/jsr.md>")),
        g => {logger(10, format!("Using JS runtime: '{}' version {}!", 
        g.bright_cyan().bold(),
        str::replace(
        str::replace(
            str::replace(
                jsr::noderunner(
                    ["-v"].to_vec(), "./".into()
                )
                .as_str(),"v","")
            .as_str(),"\n","").as_str(),
        "\r","")
            .cyan()
            )
        );
        logger(10, String::from("The JS runtime is important for plugin compatibility."));}
    }
    let mut pluginlist: Vec<PluginMeta> = [].to_vec();
    if std::path::Path::new("./plugins").exists() {
        for entry in std::fs::read_dir("./plugins").unwrap() {
            if !entry.is_err() {
                let name = entry.unwrap().file_name().to_string_lossy().into_owned();
                let p = format!("./plugins/{}/cynthiaplugin.json", name);
                let pluginmetafile = std::path::Path::new(&p);
                match std::fs::read_to_string(pluginmetafile) {
                    Ok(e) => {
                        let mut f: PluginMeta = serde_json::from_str(&e).unwrap();
                        if f.cyntia_plugin_compat != CYNTHIAPLUGINCOMPAT {
                            logger(
                    5,
                    format!(
                        "Plugin '{}' (for CynthiaPluginLoader v{}) isn't compatible with current Cynthia version (PL v{})!",
                        name,
                        f.cyntia_plugin_compat.yellow(),
                        CYNTHIAPLUGINCOMPAT.bright_yellow()
                    ))
                        } else {
                            logger(
                                1,
                                format!("Plugin '{}' loaded!", name.italic().bright_green()),
                            );
                            f.name = name;
                            match &f.runners.plugin_children {
                                Some(p) => {
                                    let cmdjson: String = p.execute.clone();
                                    let cmds: Vec<String> = serde_json::from_str(cmdjson.as_str())
                                        .unwrap_or(
                                            ["returndirect".to_string().to_string()].to_vec(),
                                        );
                                    let mut cmd: Vec<&str> = vec![];
                                    for com in &cmds {
                                        cmd.push(com.as_str());
                                    }
                                    if p.type_field == String::from("js") {
                                        logger(
                                            1,
                                            format!(
                                                "JSR: Running child script for plugin '{}'",
                                                f.name.italic().bright_green()
                                            ),
                                        );
                                        {
                                            if cmd[0] == "returndirect" {
                                                logger(1, String::from("Directreturn called on the JSR, this usually means something inside of Cynthia's Plugin Loader went wrong."));
                                            }
                                            match std::process::Command::new(jsr::jsruntime(false))
                                                .args(cmd.clone())
                                                .current_dir(
                                                    format!("./plugins/{}/", f.name).as_str(),
                                                )
                                                .spawn()
                                            {
                                                Ok(_) => {}
                                                Err(_erro) => {
                                                    logger(
                                                        5,
                                                        String::from(
                                                            "Couldn't launch Javascript runtime.",
                                                        ),
                                                    );
                                                }
                                            };
                                        }
                                    } else if p.type_field == String::from("bin") {
                                    } else {
                                        logger(5, format!("{} is using a '{}' type modifier, which is not supported by this version of cynthia",f.name,p.type_field))
                                    }
                                }
                                None => {}
                            }
                            pluginlist.push(f);
                        }
                    }
                    Err(_) => logger(
                        5,
                        format!(
                            "Plugin '{}' doesn't have a CynthiaPlugin.json manifest!",
                            name
                        ),
                    ),
                };
            }
        }
    }
    let data: Data<std::sync::Mutex<Vec<PluginMeta>>> =
        web::Data::new(std::sync::Mutex::new(pluginlist));
    HttpServer::new(move || {
        let app = App::new()
            .service(
                actix_files::Files::new("/assets", "./cynthiaFiles/assets").show_files_listing(),
            )
            .service(
                actix_files::Files::new("/jquery", "./node_modules/jquery").show_files_listing(),
            )
            .service(serves_p)
            .service(serves_e)
            .route("/", web::get().to(root))
            .app_data(web::Data::clone(&data));
        return app;
    })
    .bind(("127.0.0.1", portnum))?
    .run()
    .await
}

fn escape_json(src: &str) -> String {
    // Thank you https://www.reddit.com/r/rust/comments/i4bg0q/comment/g0hl58g/
    use std::fmt::Write;
    let mut escaped = String::with_capacity(src.len());
    let mut utf16_buf = [0u16; 2];
    for c in src.chars() {
        match c {
            '\x08' => escaped += "\\b",
            '\x0c' => escaped += "\\f",
            '\n' => escaped += "\\n",
            '\r' => escaped += "\\r",
            '\t' => escaped += "\\t",
            '"' => escaped += "\\\"",
            '\\' => escaped += "\\",
            c if c.is_ascii_graphic() => escaped.push(c),
            c => {
                let encoded = c.encode_utf16(&mut utf16_buf);
                for utf16 in encoded {
                    write!(&mut escaped, "\\u{:04X}", utf16).unwrap();
                }
            }
        }
    }
    escaped
}
