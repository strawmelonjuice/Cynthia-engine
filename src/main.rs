use actix_files as fs;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use colored::Colorize;
use curl::easy::Easy;
use dotenv::dotenv;
use handlebars::Handlebars;
use init::init;
use jsonc_parser::parse_to_serde_value;
use serde_json;
use markdown::{to_html_with_options, CompileOptions, Options};
mod init;
mod structs;
use structs::*;

#[cfg(windows)]
pub const NODEJSR: &'static str = "node.exe";
#[cfg(not(windows))]
pub const NODEJSR: &'static str = "node";
#[cfg(windows)]
pub const BUNJSR: &'static str = "bash.exe bun";
#[cfg(not(windows))]
pub const BUNJSR: &'static str = "bun";

fn noderunner(args: Vec<&str>) -> String {
    let output = match std::process::Command::new(jsr(false)).args(args).output() {
        Ok(result) => result,
        Err(_erro) => {
            logger(5, String::from("Couldn't launch Javascript runtime."));
            std::process::exit(1);
        }
    };
    if output.status.success() {
        return String::from_utf8_lossy(&output.stdout)
            .to_owned()
            .to_string();
    } else {
        println!("Script failed.");
    }
    return String::from("");
}

fn jsr(pop: bool) -> &'static str {
    match std::process::Command::new(BUNJSR).arg("version").output() {
        Ok(_t) => {
            return BUNJSR;
        }
        Err(_err) => {
            match std::process::Command::new(NODEJSR).arg("version").output() {
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
async fn serves_p(id: web::Path<String>) -> HttpResponse {
    return p_server(&id.to_string(), format!("/p/{}", id));
}

async fn root() -> impl Responder {
    return p_server(&"root".to_string(), "/".to_string());
}

fn p_server(pgid: &String, probableurl: String) -> HttpResponse {
    let cynres = combine_content(
        pgid.to_string(),
        return_content_p(pgid.to_string()),
        generate_menus(pgid.to_string(), &probableurl),
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
    // println!("{:#?}", parsed_json);
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
    // println!("{:#?}", parsed_json);
    let res: CynthiaModeObject = serde_json::from_value(parsed_json.into()).unwrap();
    return res;
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cynthia_version: &str = env!("CARGO_PKG_VERSION");
    println!(
        "{} - version {}\n by {}{}{} {}!",
        "CynthiaCMS".bold().bright_purple(),
        cynthia_version.to_string().green(),
        "Straw".bright_red(),
        "melon".green(),
        "juice".bright_yellow(),
        "Mar".magenta()
    );
    if std::env::args().nth(1).unwrap_or(String::from("")) == String::from("init") {
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
        g => logger(10, format!("Using JS runtime: '{}'!", g.bright_cyan().bold()))
    }
    
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
                        transfer
                            .write_function(|new_data| {
                                data.extend_from_slice(new_data);
                                Ok(new_data.len())
                            })
                            .unwrap();
                        transfer.perform().unwrap();
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
                "raw" => {
                    return rawcontent;
                }
                "text" => {
                    return rawcontent;
                }
                "markdown" => {
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
                "md" => {
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



fn combine_content(pgid: String, content: String, menus: Menulist) -> String {
    if content == "contentlocationerror".to_string() || content == "contenttypeerror".to_string() {
        return content;
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
        let clientjs: String = std::fs::read_to_string(
                std::path::Path::new("./src/client.js")
            )
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
            let data = CynthiaPageVars {
                head: format!(
                    r#"
            <style>
	{0}
	</style>
	<script src="/jquery/jquery.min.js"></script>
	<title>{1} &ndash; {2}</title>
	<script>
		const pagemetainfo = JSON.parse(\`{3}\`);
	</script>
	"#,
                    stylesheet, post.title, currentmode.sitename, pagemetainfojson
                ),
                content,
                menu1: menus.menu1,
                menu2: menus.menu2,
                infoshow: String::from(""),
            };
            let k = format!("<html>\n{}\n\n\n\n<script>{}</script>\n\n</html>", handlebars
                .render_template(&source.to_string(), &data)
                .unwrap(),
                clientjs);
            return k;
        }
    }
    // logger(3, String::from("Can't find that page."));
    return content;
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
