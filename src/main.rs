use std::{path::Path, process, sync::Mutex};

use actix_web::{
    get,
    http::header::ContentType,
    web::{self, Data},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use colored::Colorize;
use dotenv::dotenv;
use jsonc_parser::parse_to_serde_value;
use mime::Mime;

use structs::*;

use crate::logger::logger;

mod structs;
mod subcommand;

mod logger;

mod contentservers;
mod jsr;

pub(crate) const CYNTHIAPLUGINCOMPAT: &str = "2";

#[get("/p/{id:.*}")]
async fn serves_p(id: web::Path<String>, pluginsmex: Data<Mutex<Vec<PluginMeta>>>) -> HttpResponse {
    let plugins: Vec<PluginMeta> = pluginsmex.lock().unwrap().clone();
    let s = id.as_str();
    let pgid = if s.ends_with('/') {
        s.strip_suffix('/').unwrap()
    } else {
        s
    };
    contentservers::p_server(&pgid.to_string(), format!("/p/{}", id), plugins)
}

#[get("/c/{category:.*}")]
async fn serves_c(
    category: web::Path<String>,
    pluginsmex: Data<Mutex<Vec<PluginMeta>>>,
) -> HttpResponse {
    let plugins: Vec<PluginMeta> = pluginsmex.lock().unwrap().clone();
    let s = category.as_str();
    let pgid = if s.ends_with('/') {
        s.strip_suffix('/').unwrap()
    } else {
        s
    };
    contentservers::f_server(true, &pgid.to_string(), format!("/c/{}", category), plugins)
}

#[get("/s/{searchterm:.*}")]
async fn serves_s(
    searchterm: web::Path<String>,
    pluginsmex: Data<Mutex<Vec<PluginMeta>>>,
) -> HttpResponse {
    let plugins: Vec<PluginMeta> = pluginsmex.lock().unwrap().clone();
    let s = searchterm.as_str();
    let term = if s.ends_with('/') {
        s.strip_suffix('/').unwrap()
    } else {
        s
    };
    contentservers::s_server(&term.to_string(), format!("/s/{}", searchterm), plugins)
}

#[get("/t/{tag:.*}")]
async fn serves_t(
    tag: web::Path<String>,
    pluginsmex: Data<Mutex<Vec<PluginMeta>>>,
) -> HttpResponse {
    let plugins: Vec<PluginMeta> = pluginsmex.lock().unwrap().clone();
    let s = tag.as_str();
    let pgid = if s.ends_with('/') {
        s.strip_suffix('/').unwrap()
    } else {
        s
    };
    contentservers::f_server(false, &pgid.to_string(), format!("/t/{}", tag), plugins)
}

fn find_mimetype(filename_: &str) -> Mime {
    let filename = filename_.replace('"', "").to_lowercase();
    let parts: Vec<&str> = filename.split('.').collect();

    let res = match parts.last() {
        Some(v) => match *v {
            "png" => mime::IMAGE_PNG,
            "jpg" => mime::IMAGE_JPEG,
            "json" => mime::APPLICATION_JSON,
            "js" => mime::TEXT_JAVASCRIPT,
            "ico" => "image/vnd.microsoft.icon".parse().unwrap(),
            "svg" => mime::IMAGE_SVG,
            "css" => mime::TEXT_CSS,
            &_ => mime::TEXT_PLAIN,
        },
        None => mime::TEXT_PLAIN,
    };
    // println!("{filename}: {res}");
    res
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
                    let l = match s[1].ends_with('/') {
                        true => &s[1],
                        false => &z,
                    };
                    if id.starts_with(&**l) {
                        let fid = id.replace(&**l, "");
                        let fileb = format!("./plugins/{}/{}/{fid}", plugin.name, s[0]);
                        let file = Path::new(&fileb);
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

#[get("/es/{en}/{id:.*}")]
async fn serves_es(req: HttpRequest, pluginsmex: Data<Mutex<Vec<PluginMeta>>>) -> HttpResponse {
    let en: String = req.match_info().get("en").unwrap().parse().unwrap();
    let id: String = req.uri().to_string().replacen("/es", "", 1);
    let plugins: Vec<PluginMeta> = pluginsmex.lock().unwrap().clone();
    let mut body = String::new();
    for plugin in plugins {
        match &plugin.runners.proxied {
            Some(p) => {
                for s in p {
                    // println!("{} == {}?", en , s[1].to_string());
                    if en == s[1].to_string() {
                        body = contentservers::fetcher(format!("{}/{}", s[0], id));
                    };
                }
            }
            None => {}
        }
    }

    return HttpResponse::Ok().body(body);
}

async fn root(pluginsmex: Data<Mutex<Vec<PluginMeta>>>) -> impl Responder {
    let plugins: Vec<PluginMeta> = pluginsmex.lock().unwrap().clone();
    contentservers::p_server(&"root".to_string(), "/".to_string(), plugins)
}

fn read_published_jsonc() -> Vec<CynthiaContentMetaData> {
    let file = "./cynthiaFiles/published.jsonc".to_owned();
    let unparsed_json = std::fs::read_to_string(file).expect("Couldn't find or load that file.");
    // println!("{}", unparsed_json);
    let parsed_json: Option<serde_json::Value> =
        parse_to_serde_value(unparsed_json.as_str(), &Default::default())
            .expect("Could not read published.jsonc.");
    let res: Vec<CynthiaContentMetaData> = serde_json::from_value(parsed_json.into()).unwrap();
    res
}

fn load_mode(mode_name: String) -> CynthiaModeObject {
    let file = format!("./cynthiaFiles/modes/{}.jsonc", mode_name).to_owned();
    let unparsed_json = std::fs::read_to_string(file).expect("Couldn't find or load that file.");
    // println!("{}", unparsed_json);
    let parsed_json: Option<serde_json::Value> =
        parse_to_serde_value(unparsed_json.as_str(), &Default::default())
            .expect("Could not read published.jsonc.");
    serde_json::from_value(parsed_json.into()).unwrap()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!(
        "{} - version {}\n by {}{}{} {}!",
        "CynthiaEngine".bold().bright_purple(),
        env!("CARGO_PKG_VERSION").to_string().green(),
        "Straw".bright_red(),
        "melon".green(),
        "juice".bright_yellow(),
        "Mar".magenta()
    );
    if std::env::args()
        .nth(1)
        .unwrap_or(String::from(""))
        .to_lowercase()
        == *"help"
    {
        println!(
            r#"{}Help

{}
{}

As of now, Cynthia has only 4 commands:

- {}
    You are viewing this now. It just displays this and then exits!
- {}
    Creates a new CynthiaConfig in the folder you are currently in. You need to run this command before being able to host a Cynthia site from a new folder!
- {} {{{}}} <{}> ({})
    Available subcommands:
        - Add:
            Installs a new plugin as registered in the Cynthia Plugin Index. (Does not save it to the manifest file.)

            Options:
                - <{}>
                    Specifies the name of the plugin to install. Is required.
                - {{{}}}
                    (Optional) Specifies the plugin version (this will not work if a plugin has a single-version channel)
                    If not specified, latest available will be used.
        - Install:
            Installs plugins from {} using the Cynthia Plugin Index. Useful after cloning a config.
- {}
    Starts the Cynthia server!
{}"#, "\r",
            "Cynthia is a way to host stuff, but also a very extensible and structurised generator of stuff. And by stuff, I mean websites.".italic(),
            format!("This help page helps you through the Cynthia {} only. For a guide on {}, see its documentation on {}.", "cli-options".cyan(), "the CynthiaConfig", "https://cynthia-docs.strawmelonjuice.com/".underline().blue()).blue(),
            "Help".bold().yellow(),
            "Init".bold().yellow(),
            "PM".bold().yellow(),"subcommand".bright_green(),"plugin name".bright_yellow(), "plugin version".bright_purple(),
            "plugin name".bright_yellow(),
            "plugin version".bright_purple()       ,
            "cynthiapluginmanifest.json".bright_green(),
            "Start".bold().yellow(),
"\n\r"
        );
        process::exit(0);
    } else if std::env::args()
        .nth(1)
        .unwrap_or(String::from(""))
        .to_lowercase()
        == *"init"
    {
        subcommand::init();
    } else if std::env::args()
        .nth(1)
        .unwrap_or(String::from(""))
        .to_lowercase()
        == *"pm"
    {
        if std::env::args()
        .nth(2)
        .unwrap_or(String::from(""))
        .to_lowercase()
        == *"add"
    {
        subcommand::plugin_install(
            std::env::args().nth(3).unwrap_or(String::from("none")),
            std::env::args().nth(4).unwrap_or(String::from("latest")),
        );
    } else if std::env::args()
        .nth(2)
        .unwrap_or(String::from(""))
        .to_lowercase()
        == *"install"{
            subcommand::install_from_plugin_manifest()
        } else {
            logger(
            5,
            format!(
                "No subcommand specified! Use '{} {}' for help.",
                std::env::args()
                    .next()
                    .unwrap_or(String::from("cynthiaweb"))
                    .purple(),
                "help".bright_yellow()
            ),
        );
        process::exit(1);
        }
        process::exit(0);
    } else if std::env::args()
        .nth(1)
        .unwrap_or(String::from(""))
        .to_lowercase()
        == *""
    {
        logger(
            5,
            format!(
                "No command specified! Use '{} {}' for help.",
                std::env::args()
                    .next()
                    .unwrap_or(String::from("cynthiaweb"))
                    .purple(),
                "help".bright_yellow()
            ),
        );
        process::exit(1);
    } else if std::env::args()
        .nth(1)
        .unwrap_or(String::from(""))
        .to_lowercase()
        != *"start"
    {
        logger(
            5,
            format!(
                "Unknown command! Use '{} {}' for help.",
                std::env::args()
                    .next()
                    .unwrap_or(String::from("cynthiaweb"))
                    .purple(),
                "help".bright_yellow()
            ),
        );
        process::exit(1);
    }
    if !Path::new("./.env").exists() {
        logger(5, String::from("No CynthiaConfig found."));
        logger(
            10,
            format!(
                "To set up a clean Cynthia config, run {} {}.",
                std::env::args()
                    .next()
                    .unwrap_or(String::from("cynthiaweb"))
                    .purple(),
                "init".bright_yellow()
            ),
        );
        process::exit(1);
    }
    logger(
        1,
        format!(
            "🤔 Loading configuration from '{}'!",
            Path::new("./.env")
                .canonicalize()
                .unwrap()
                .display()
                .to_string()
                .replace("\\\\?\\", "")
                .bright_purple()
                .italic()
        ),
    );
    dotenv().ok();
    let portnum: u16 = std::env::var("PORT")
        .expect("PORT must be set in the '.env' file.")
        .parse::<u16>()
        .unwrap();
    match jsr::jsruntime(true) {
        "" => logger(5, String::from("No JS runtime found! Cynthia doesn't need one, but most of it's plugins do!\n\nSee: <https://github.com/strawmelonjuice/CynthiaWebsiteEngine/blob/rust/docs/jsr.md>")),
        g => {
            logger(1, format!("💪 Using JS runtime: '{}' version {}!",
                              g.bright_cyan().bold(),
                              str::replace(
                                  str::replace(
                                      str::replace(
                                          jsr::noderunner(
                                              ["-v"].to_vec(), "./".into(),
                                          )
                                              .as_str(), "v", "")
                                          .as_str(), "\n", "").as_str(),
                                  "\r", "")
                                  .cyan()
            ),
            );
            logger(10, String::from("The JS runtime is important for plugin compatibility."));
        }
    }
    let mut pluginlist: Vec<PluginMeta> = [].to_vec();
    if Path::new("./plugins").exists() {
        for entry in std::fs::read_dir("./plugins").unwrap() {
            if entry.is_ok() {
                let name = entry.unwrap().file_name().to_string_lossy().into_owned();
                let p = format!("./plugins/{}/cynthiaplugin.json", name);
                let pluginmetafile = Path::new(&p);
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
                                format!("🧩  Plugin '{}' loaded!", name.italic().bright_green()),
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
                                    if p.type_field == *"js" {
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
                                            match process::Command::new(jsr::jsruntime(false))
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
                                    } else if p.type_field == *"bin" {
                                    } else {
                                        logger(5, format!("{} is using a '{}' type modifier, which is not supported by this version of cynthia", f.name, p.type_field))
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
    let data: Data<Mutex<Vec<PluginMeta>>> = Data::new(Mutex::new(pluginlist));
    logger(
        1,
        format!(
            "🆙  Running at {} ...",
            format!(
                "http://{}:{}/",
                "localhost".green(),
                portnum.to_string().bold().green()
            )
            .yellow()
            .italic()
        ),
    );
    HttpServer::new(move || {
        App::new()
            .service(
                actix_files::Files::new("/assets", "./cynthiaFiles/assets").show_files_listing(),
            )
            .service(serves_p)
            .service(serves_c)
            .service(serves_t)
            .service(serves_s)
            .service(serves_e)
            .service(serves_es)
            .route("/", web::get().to(root))
            .app_data(web::Data::clone(&data))
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
