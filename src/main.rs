use actix_files::NamedFile;
use std::io::{Error, ErrorKind};
use std::{fs, path::Path, process, sync::Mutex};

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

use crate::dashfunctions::dashserver;
use crate::files::import_js_minified;
use structs::*;

use crate::logger::logger;

mod structs;
mod subcommand;

mod logger;

mod contentservers;
mod dashfunctions;
mod files;
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

#[get("/ej/{id:.*}")]
async fn serves_ej(
    id: web::Path<String>,
    pluginsmex: Data<Mutex<Vec<PluginMeta>>>,
) -> HttpResponse {
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
                        body = if mime == mime::TEXT_JAVASCRIPT {
                            import_js_minified(file.to_str().unwrap().to_string())
                        } else {
                            fs::read_to_string(file).unwrap_or(String::from("Couldn't serve file."))
                        };
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

async fn serves_e(
    id: web::Path<String>,
    pluginsmex: Data<Mutex<Vec<PluginMeta>>>,
) -> Result<NamedFile, Error> {
    let plugins: Vec<PluginMeta> = pluginsmex.lock().unwrap().clone();
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
                        logger(
                            10,
                            format!("Serving {}", file.canonicalize().unwrap().display()),
                        );
                        return NamedFile::open(file);
                    };
                }
            }
            None => {}
        }
    }
    Err(Error::from(ErrorKind::NotFound))
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
                    if en == s[1] {
                        body = contentservers::fetcher(format!("{}/{}", s[0], id));
                    };
                }
            }
            None => {}
        }
    }

    HttpResponse::Ok().body(body)
}

async fn root(pluginsmex: Data<Mutex<Vec<PluginMeta>>>) -> impl Responder {
    let plugins: Vec<PluginMeta> = pluginsmex.lock().unwrap().clone();
    contentservers::p_server(&"root".to_string(), "/".to_string(), plugins)
}

fn read_published_jsonc() -> Vec<CynthiaContentMetaData> {
    if Path::new("./cynthiaFiles/published.yaml").exists() {
        let file = "./cynthiaFiles/published.yaml".to_owned();
        let unparsed_yaml = fs::read_to_string(file).expect("Couldn't find or load that file.");
        serde_yaml::from_str(&unparsed_yaml).unwrap_or_else(|_e| {
            logger(
                5,
                String::from("Published.yaml contains invalid Cynthia-instructions."),
            );
            Vec::new()
        })
    } else {
        let file = "./cynthiaFiles/published.jsonc".to_owned();
        let unparsed_json = fs::read_to_string(file).expect("Couldn't find or load that file.");
        // println!("{}", unparsed_json);
        let parsed_json: Option<serde_json::Value> =
            parse_to_serde_value(unparsed_json.as_str(), &Default::default())
                .expect("Could not read published.jsonc.");
        serde_json::from_value(parsed_json.into()).unwrap_or_else(|_e| {
            logger(
                5,
                String::from("Published.json contains invalid Cynthia-instructions."),
            );
            Vec::new()
        })
    }
}

fn load_mode(mode_name: String) -> CynthiaModeObject {
    let file = format!("./cynthiaFiles/modes/{}.jsonc", mode_name).to_owned();
    let unparsed_json = match fs::read_to_string(file) {
        Ok(s) => s,
        Err(f) => {
            if f.kind() == ErrorKind::NotFound {
                if mode_name != *"default" {
                    logger(15, format!("Cynthia is missing the `{}´ mode for a page to be served. It will retry using the `default´ mode.", mode_name));
                    return load_mode(String::from("default"));
                } else {
                    logger(
                        5,
                        String::from("Cynthia is missing the right mode for some pages to serve."),
                    );
                    process::exit(1);
                }
            } else {
                logger(
                    5,
                    String::from(
                        "Cynthia is having trouble loading the mode for some pages to serve.",
                    ),
                );
                process::exit(1);
            }
        }
    };
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
            == *"install"
        {
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
    if !Path::new("./.env").exists() || !Path::new("./cynthiaFiles").exists() {
        logger(5, String::from("No CynthiaConfig found."));
        logger(
            10,
            format!(
                "To set up a clean Cynthia config, run '{} {}'.",
                std::env::args()
                    .next()
                    .unwrap_or(String::from("cynthiaweb"))
                    .purple(),
                "init".bright_yellow()
            ),
        );
        process::exit(1);
    }
    logger(1, "🤔\tLoading configuration from:".to_string());
    logger(
        1,
        format!(
            "`{}´",
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
    let _ = fs::remove_dir_all("./.cynthiaTemp");
    match fs::create_dir_all("./.cynthiaTemp") {
        Ok(_) => {}
        Err(e) => {
            logger(
                5,
                format!(
                    "Could not create the Cynthia temp folder! Error: {}",
                    e.to_string().bright_red()
                ),
            );
            process::exit(1);
        }
    }
    let portnum: u16 = match std::env::var("PORT") {
        Ok(g) => g.parse::<u16>().unwrap(),
        Err(_) => 3000,
    };
    match jsr::jsruntime(true) {
        "" => logger(5, String::from("No JS runtime found! Cynthia doesn't need one, but most of it's plugins do!\n\nSee: <https://github.com/strawmelonjuice/CynthiaWebsiteEngine/blob/rust/docs/jsr.md>")),
        g => {
            logger(1, format!("💪\tUsing JS runtime: '{}' version {}!",
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
    let mut cynthiadashactive: bool = false;
    if Path::new("./plugins").exists() {
        for entry in fs::read_dir("./plugins").unwrap() {
            if entry.is_ok() {
                let name = entry.unwrap().file_name().to_string_lossy().into_owned();
                let p = format!("./plugins/{}/cynthiaplugin.json", name);
                let pluginmetafile = Path::new(&p);
                if name != ".gitignore" {
                    match fs::read_to_string(pluginmetafile) {
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
                                    format!(
                                        "🧩\tPlugin '{}' loaded!",
                                        name.italic().bright_green()
                                    ),
                                );
                                f.name = name;
                                match &f.runners.plugin_children {
                                    Some(p) => {
                                        let cmdjson: String = p.execute.clone();
                                        let mut cmds: Vec<String> =
                                            serde_json::from_str(cmdjson.as_str()).unwrap_or(
                                                ["returndirect".to_string().to_string()].to_vec(),
                                            );
                                        if f.name == "cynthia-dash" {
                                            cmds.push(
                                                dashfunctions::passkey()
                                                    .unwrap_or(String::from("")),
                                            );
                                            cynthiadashactive = true;
                                        }
                                        let mut cmd: Vec<&str> = vec![];
                                        for com in &cmds {
                                            cmd.push(com.as_str());
                                        }
                                        if p.type_field == *"js" {
                                            logger(
                                                1,
                                                format!(
                                                    "🏃\tRunning child script for plugin '{}'",
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
                            15,
                            format!(
                                "Plugin `{}´ doesn't have a CynthiaPlugin.json manifest!",
                                name
                            ),
                        ),
                    }
                };
            }
        }
    }
    let data: Data<Mutex<Vec<PluginMeta>>> = Data::new(Mutex::new(pluginlist));
    logger(
        1,
        format!(
            "🆙\tRunning at {} ...",
            format!(
                "http://{}:{}/",
                "localhost".green(),
                portnum.to_string().bold().green()
            )
            .yellow()
            .italic()
        ),
    );
    if cynthiadashactive {
        logger(15, String::from("Cynthia dashboard plugin found! The Cynthia Dashboard has additional permissions, so uninstall it if left unused, also check the source  of this plugin."));

        HttpServer::new(move || {
            App::new()
                .service(
                    actix_files::Files::new("/assets", "./cynthiaFiles/assets")
                        .show_files_listing(),
                )
                .service(serves_p)
                .service(serves_c)
                .service(serves_t)
                .service(serves_s)
                .route("/e/{id:.*}", web::get().to(serves_e))
                .service(serves_ej)
                .service(serves_es)
                .route("/", web::get().to(root))
                .app_data(web::Data::clone(&data))
                .service(dashserver)
        })
        .bind(("127.0.0.1", portnum))?
        .run()
        .await
    } else {
        HttpServer::new(move || {
            App::new()
                .service(
                    actix_files::Files::new("/assets", "./cynthiaFiles/assets")
                        .show_files_listing(),
                )
                .service(serves_p)
                .service(serves_c)
                .service(serves_t)
                .service(serves_s)
                .route("/e/{id:.*}", web::get().to(serves_e))
                .service(serves_ej)
                .service(serves_es)
                .route("/", web::get().to(root))
                .app_data(web::Data::clone(&data))
        })
        .bind(("127.0.0.1", portnum))?
        .run()
        .await
    }
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
