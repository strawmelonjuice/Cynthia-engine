use std::sync::Mutex;

use actix_web::{
    get,
    http::header::ContentType,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use colored::Colorize;
use curl::easy::Easy;
use dotenv::dotenv;
use handlebars::Handlebars;
use init::init;
use jsonc_parser::parse_to_serde_value;
use markdown::{to_html_with_options, CompileOptions, Options};
use mime::Mime;
mod init;
mod structs;
use structs::*;

const CYNTHIAPLUGINCOMPAT: &str = "2";

// Javascript runtimes:
//     NodeJS:
#[cfg(windows)]
pub const NODEJSR: &str = "node.exe";
#[cfg(not(windows))]
pub const NODEJSR: &'static str = "node";
//     Bun:
#[cfg(windows)]
pub const BUNJSR: &str = "bash.exe bun";
#[cfg(not(windows))]
pub const BUNJSR: &'static str = "bun";

// Javascript package managers:
//     NPM:
#[cfg(windows)]
pub const NODE_NPM: &str = "node";
#[cfg(not(windows))]
pub const NODE_NPM: &str = "node";
//     PNPM:
#[cfg(windows)]
pub const PNPM: &str = "pnpm.exe";
#[cfg(not(windows))]
pub const PNPM: &str = "pnpm";
//     Bun:
#[cfg(windows)]
pub const BUN_NPM: &str = "bash.exe bun";
#[cfg(not(windows))]
pub const BUN_NPM: &'static str = "bun";

fn noderunner(args: Vec<&str>, cwd: std::path::PathBuf) -> String {
    if args[0] == "returndirect" {
        logger(1, String::from("Directreturn called on the JSR, this usually means something inside of Cynthia's Plugin Loader went wrong."));
        return args[1].to_string();
    }
    let output = match std::process::Command::new(jsr(false))
        .args(args.clone())
        .current_dir(cwd)
        .output()
    {
        Ok(result) => result,
        Err(_erro) => {
            logger(5, String::from("Couldn't launch Javascript runtime."));
            std::process::exit(1);
        }
    };
    if output.status.success() {
        return String::from_utf8_lossy(&output.stdout)
            .into_owned()
            .to_string();
    } else {
        println!("Script failed.");
        logger(12, String::from_utf8_lossy(&output.stderr).to_string());
    }
    String::from("")
}

fn jsr(pop: bool) -> &'static str {
    match std::process::Command::new(BUNJSR).arg("-v").output() {
        Ok(_t) => {
            return BUNJSR;
        }
        Err(_err) => {
            match std::process::Command::new(NODEJSR).arg("-v").output() {
                Ok(_t) => {
                    return NODEJSR;
                }
                Err(_err) => {
                    if !pop {
                        logger(
                            5,
                            String::from(
                                "No supported (Node.JS or Bun) Javascript runtimes found on path!",
                            ),
                        );
                        std::process::exit(1);
                    }
                    return "";
                }
            };
        }
    };
}

fn logger(act: i32, msg: String) {
    /*

    Acts:
    0: Debug log, only act if logging is set to verbose
    1: General log item -- '[log]'
    2/200: Request that on Cynthia's part succeeded (and is so responded to) -- '[CYNGET/OK]'
    3/404: Request for an item that does not exist Cynthia published.jsonc

    5: Error!


    10: Note

    12: Error in JSR

     */
    let spaces: usize = 10;
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
    if act == 200 || act == 2 {
        let name = "✅ [CYNGET/OK]";
        let spaceleft = if name.chars().count() < spaces {
            spaces - name.chars().count()
        } else {
            0
        };
        let title = format!("{}", name.bold().yellow());
        let preq = format!("{0}{2}{1}", title, " ".repeat(spaceleft), tabs);
        println!("{0}{1}", preq, msg);
    }
    if act == 3 || act == 404 {
        let name = "❎ [CYNGET/404]";
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
        let name = "[ERROR]";
        let spaceleft = if name.chars().count() < spaces {
            spaces - name.chars().count()
        } else {
            0
        };
        let title = format!("{}", name.bold().black().on_bright_yellow());
        let preq = format!("{0}{2}{1}", title, " ".repeat(spaceleft), tabs);
        println!("{0}{1}", preq, msg.bright_red());
    }
    if act == 12 {
        let name = "[JS/ERROR]";
        let spaceleft = if name.chars().count() < spaces {
            spaces - name.chars().count()
        } else {
            0
        };
        let title = format!("{}", name.bold().black().on_bright_yellow());
        let preq = format!("{0}{2}{1}", title, " ".repeat(spaceleft), tabs);
        println!("{0}{1}", preq, msg.bright_red().on_bright_yellow());
    }
    if act == 10 {
        let name = "[Note]";
        let spaceleft = if name.chars().count() < spaces {
            spaces - name.chars().count()
        } else {
            0
        };
        let title = format!("{}", name.bold().bright_magenta());
        let preq = format!("{0}{2}{1}", title, " ".repeat(spaceleft), tabs);
        println!("{0}{1}", preq, msg.bright_purple());
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
async fn serves_p(id: web::Path<String>, pluginsmex: Data<Mutex<Vec<PluginMeta>>>) -> HttpResponse {
    let plugins: Vec<PluginMeta> = pluginsmex.lock().unwrap().clone();
    return p_server(&id.to_string(), format!("/p/{}", id), plugins);
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
    return p_server(&"root".to_string(), "/".to_string(), plugins);
}

fn p_server(pgid: &String, probableurl: String, plugins: Vec<PluginMeta>) -> HttpResponse {
    let cynres = combine_content(
        pgid.to_string(),
        return_content_p(pgid.to_string()),
        generate_menus(pgid.to_string(), &probableurl),
        plugins.clone(),
    );
    if cynres == String::from("404error") {
        logger(404, format!("--> {0} ({1})", pgid, probableurl));
        return HttpResponse::NotFound().into();
    }
    if cynres == String::from("unknownexeception") {
        logger(5, format!("--> {0} ({1})", pgid, probableurl));
        return HttpResponse::ExpectationFailed().into();
    }
    if cynres == String::from("contentlocationerror") {
        logger(
            5,
            format!("--> {0} ({1}) : Post location error", pgid, probableurl),
        );
        return HttpResponse::ExpectationFailed().into();
    }
    logger(200, format!("--> {0} ({1})", pgid, probableurl));
    return HttpResponse::Ok().body(cynres).into();
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
            "{}",
            format!(
                "Starting server on {0}{1}...",
                "http://localhost:".green(),
                portnum.to_string().bold().green()
            )
            .italic()
        ),
    );
    match jsr(true) {
        "" => logger(5, String::from("No JS runtime found! Cynthia doesn't need one, but most of it's plugins do!\n\nSee: <https://github.com/strawmelonjuice/CynthiaCMS/blob/rust/docs/jsr.md>")),
        g => {logger(10, format!("Using JS runtime: '{}' version {}!", 
        g.bright_cyan().bold(),
        str::replace(
        str::replace(
            str::replace(
                noderunner(
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
                                            match std::process::Command::new(jsr(false))
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

fn return_content_p(pgid: String) -> String {
    let published_jsonc = read_published_jsonc();
    for i in &published_jsonc {
        if i.id == pgid {
            let post: &CynthiaPostData = i;
            if post.kind == "postlist" {
                return "Cynthia cannot handle post lists just yet!"
                    .to_owned()
                    .to_string();
            };
            let rawcontent: String;
            match (post.content.location).to_owned().as_str() {
                "external" => {
                    let mut data = Vec::new();
                    let mut c = Easy::new();
                    c.url(&(post.content.data)).unwrap();
                    {
                        let mut transfer = c.transfer();
                        match transfer
                            .write_function(|new_data| {
                                data.extend_from_slice(new_data);
                                Ok(new_data.len())
                            }) {
                        Ok(v) => v,
                        Err(_e) => {
                            logger(5, String::from("Could not download external content!"));

                            return "contentlocationerror".to_owned();
                        }};
                        match transfer.perform() {
                        Ok(v) => v,
                        Err(_e) => {
                            logger(5, String::from("Could not download external content!"));

                            return "contentlocationerror".to_owned();
                        }
                    };
                    }
                    let resp = match std::str::from_utf8(&data) {
                        Ok(v) => v,
                        Err(_e) => {
                            logger(5, String::from("Could not download external content!"));

                            return "contentlocationerror".to_owned();
                        }
                    };
                    rawcontent = resp.to_owned();
                }
                "local" => {
                    let contentpath_ = std::path::Path::new("./cynthiaFiles/pages/");
                    let contentpath = &contentpath_.join(post.content.data.to_owned().as_str());
                    rawcontent =
                        std::fs::read_to_string(contentpath).unwrap_or("contenterror".to_string());
                }
                "inline" => {
                    rawcontent = post.content.data.to_owned();
                }
                &_ => {
                    return "contentlocationerror".to_owned();
                }
            }
            match (post.content.markup_type)
                .to_owned()
                .to_lowercase()
                .as_str()
            {
                "html" | "webfile" => {
                    return format!(
                        "<div><pre>{}</pre></div>",
                        rawcontent
                            .replace("&", "&amp;")
                            .replace("<", "&lt;")
                            .replace(">", "&gt;")
                            .replace('"', "&quot;")
                            .replace("'", "&#039;")
                    );
                }
                "text" | "raw" => {
                    return format!("<div>{rawcontent}</div>");
                }
                "markdown" | "md" => {
                    return format!(
                        "<div>{}</div>",
                        to_html_with_options(
                            &rawcontent,
                            &Options {
                                compile: CompileOptions {
                                    allow_dangerous_html: true,
                                    ..CompileOptions::default()
                                },
                                ..Options::default()
                            },
                        )
                        .unwrap()
                    );
                }
                "" => {
                    return to_html_with_options(
                        &rawcontent,
                        &Options {
                            compile: CompileOptions {
                                allow_dangerous_html: true,
                                ..CompileOptions::default()
                            },
                            ..Options::default()
                        },
                    )
                    .unwrap();
                }
                &_ => {
                    return "contenttypeerror".to_owned();
                }
            }
        }
    }
    String::from("404error")
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
fn combine_content(
    pgid: String,
    content: String,
    menus: Menulist,
    plugins: Vec<PluginMeta>,
) -> String {
    match content.as_str() {
        "contentlocationerror" | "404error" | "contenttypeerror" => return content,
        &_ => {}
    }
    let mut contents = content;
    for plugin in plugins.clone() {
        match &plugin.runners.modify_body_html {
            Some(p) => {
                let handlebars = Handlebars::new();
                let mut data = std::collections::BTreeMap::new();
                data.insert("input".to_string(), "kamkdxcvjgCVJGVvdbvcgcvgdvd");
                let cmdjson: String = handlebars
                    .render_template(&p.execute, &data)
                    .unwrap_or(format!("[ \"returndirect\", \"f{}\" ]", contents));
                let path = "cmdjson.json";
                if false {
                    use std::io::Write;
                    let mut output = std::fs::File::create(path).unwrap();
                    write!(output, "{}", cmdjson.as_str()).unwrap();
                }
                let cmds: Vec<String> = serde_json::from_str(cmdjson.as_str()).unwrap();
                // .unwrap_or(["returndirect", contents.as_str()].to_vec());
                let mut cmd: Vec<&str> = vec![];
                for com in &cmds {
                    cmd.push(match com.as_str() {
                        "kamkdxcvjgCVJGVvdbvcgcvgdvd" => contents.as_str(),
                        a => a,
                    });
                }
                if p.type_field == String::from("js") {
                    contents = noderunner(cmd, format!("./plugins/{}/", plugin.name).into());
                } else {
                    logger(5, format!("{} is using a '{}' type allternator, which is not supported by this version of cynthia",plugin.name,p.type_field))
                }
            }
            None => {}
        }
    }
    let mut published_jsonc = read_published_jsonc();
    for post in &mut published_jsonc {
        if post.id == pgid {
            let mode_to_load = post
                .mode
                .get_or_insert_with(|| String::from("default"))
                .to_string();
            let pagemetainfojson = serde_json::to_string(&post).unwrap();
            let currentmode = load_mode(mode_to_load).1;
            let stylesheet: String = std::fs::read_to_string(
                std::path::Path::new("./cynthiaFiles/styles/").join(currentmode.stylefile),
            )
            .unwrap_or(String::from(""));
            let clientjs: String = std::fs::read_to_string(std::path::Path::new("./src/client.js"))
                .expect("Could not load src/client.js");
            let handlebarfile = format!(
                "./cynthiaFiles/templates/{}.handlebars",
                (if post.kind == "post" {
                    currentmode.handlebar.post
                } else {
                    currentmode.handlebar.page
                })
            )
            .to_owned();
            let source = std::fs::read_to_string(handlebarfile)
                .expect("Couldn't find or load handlebars file.");
            let handlebars = Handlebars::new();
            let mut head = format!(
                r#"
            <style>
	{0}
	</style>
	<script src="/jquery/jquery.min.js"></script>
	<title>{1} &ndash; {2}</title>
	"#,
                stylesheet, post.title, currentmode.sitename
            );
            for plugin in plugins.clone() {
                match &plugin.runners.modify_head_html {
                    Some(p) => {
                        let handlebars = Handlebars::new();
                        let mut data = std::collections::BTreeMap::new();
                        data.insert("input".to_string(), escape_json(&head));
                        let cmdjson: String = handlebars
                            .render_template(&p.execute, &data)
                            .unwrap_or(format!("[ \"returndirect\", \"f{}\" ]", head));
                        let path = "cmdjson.json";
                        if false {
                            use std::io::Write;
                            let mut output = std::fs::File::create(path).unwrap();
                            write!(output, "{}", cmdjson.as_str()).unwrap();
                        }
                        let cmds: Vec<String> = serde_json::from_str(cmdjson.as_str()).unwrap_or(
                            ["returndirect".to_string(), escape_json(&head).to_string()].to_vec(),
                        );
                        let mut cmd: Vec<&str> = vec![];
                        for com in &cmds {
                            cmd.push(com.as_str());
                        }
                        if p.type_field == String::from("js") {
                            head = noderunner(cmd, format!("./plugins/{}/", plugin.name).into());
                        } else {
                            logger(5, format!("{} is using a '{}' type modifier, which is not supported by this version of cynthia",plugin.name,p.type_field))
                        }
                    }
                    None => {}
                }
            }
            head.push_str(
                format!(
                    r#"<script>
		const pagemetainfo = JSON.parse(\`{0}\`);
	</script>"#,
                    pagemetainfojson
                )
                .as_str(),
            );
            let data = CynthiaPageVars {
                head,
                content: contents,
                menu1: menus.menu1,
                menu2: menus.menu2,
                infoshow: String::from(""),
            };
            let mut k = format!(
                "<html>\n{}\n\n\n\n<script>{}</script>\n\n</html>",
                handlebars
                    .render_template(&source.to_string(), &data)
                    .unwrap(),
                clientjs
            );
            for plugin in plugins.clone() {
                match &plugin.runners.modify_output_html {
                    Some(p) => {
                        let handlebars = Handlebars::new();
                        let mut data = std::collections::BTreeMap::new();
                        data.insert("input".to_string(), "kamdlnjnjnsjkanj");
                        let cmdjson: String = handlebars
                            .render_template(&p.execute, &data)
                            .unwrap_or(format!("[ \"returndirect\", \"f{}\" ]", k));
                        let path = "cmdjson.json";
                        if false {
                            use std::io::Write;
                            let mut output = std::fs::File::create(path).unwrap();
                            write!(output, "{}", cmdjson.as_str()).unwrap();
                        }
                        let cmds: Vec<String> = serde_json::from_str(cmdjson.as_str()).unwrap();
                        // .unwrap_or(["returndirect".to_string(), escape_json(&k).to_string()].to_vec());
                        let mut cmd: Vec<&str> = vec![];
                        for com in &cmds {
                            cmd.push(match com.as_str() {
                                // See? We support templating :')
                                "kamdlnjnjnsjkanj" => k.as_str(),
                                a => a,
                            });
                        }
                        // let cmd = ["append.js", "output", k.as_str()].to_vec();
                        if p.type_field == String::from("js") {
                            k = noderunner(cmd, format!("./plugins/{}/", plugin.name).into());
                        } else {
                            logger(5, format!("{} is using a '{}' type modifier, which is not supported by this version of cynthia",plugin.name,p.type_field))
                        }
                    }
                    None => {}
                }
            }
            return format!("<!--\n\nGenerated and hosted through Cynthia v{}, by Strawmelonjuice.\nAlso see:\t<https://github.com/strawmelonjuice/CynthiaCMS-JS/blob/main/README.MD>\n\n-->\n\n\n\n\r{k}", env!("CARGO_PKG_VERSION"));
        }
    }
    // logger(3, String::from("Can't find that page."));
    return contents;
}

fn generate_menus(pgid: String, probableurl: &String) -> Menulist {
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
                        let link: String = if ele.href == String::from(probableurl) {
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
            if !mode.menu2links.is_empty() {
                for ele in mode.menu2links {
                    let link: String = if ele.href == String::from(probableurl) {
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
