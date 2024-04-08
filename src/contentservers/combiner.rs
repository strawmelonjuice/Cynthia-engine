use std::string::String;

use handlebars::Handlebars;

use crate::{config, jsr, logger, structs::*};
use crate::files::{import_css_minified, import_js_minified};

pub(crate) fn combine_content(
    pgid: String,
    content: String,
    menus: Menulist,
    plugins: Vec<PluginMeta>,
) -> String {
    match content.as_str() {
        "contentlocationerror" | "404error" | "contenttypeerror" => return content,
        &_ => {}
    }
    let mut contents = escape_known_problematic_chars(content);
    contents = runner_content(plugins.clone(), contents);
    let mut published_jsonc = crate::read_published_jsonc();
    for p_met in &mut published_jsonc {
        if p_met.id == pgid {
            let mode_to_load = p_met
                .mode
                .get_or_insert_with(|| String::from("default"))
                .to_string();
            let p_m_met: CynthiaContentMetaDataMinimal =
                serde_json::from_str(serde_json::to_string(&p_met).unwrap().as_str()).unwrap();
            let pagemetainfojson = serde_json::to_string(&p_m_met).unwrap();
            let currentmode = crate::load_mode(mode_to_load).1;
            let stylesheet: String =
                import_css_minified(format!("./cynthiaFiles/styles/{}", currentmode.stylefile));
            let handlebarfile = format!(
                "./cynthiaFiles/templates/{}.handlebars",
                if p_met.kind == "post" {
                    currentmode.handlebar.post
                } else {
                    currentmode.handlebar.page
                }
            )
            .to_owned();
            let source = std::fs::read_to_string(handlebarfile)
                .expect("Couldn't find or load handlebars file.");
            let handlebars = Handlebars::new();
            let favicondec = match currentmode.favicon {
                Some(d) => {
                    format!(
                        r#"<link rel="shortcut icon" href="/assets/{}" type="image/x-icon"/>"#,
                        d
                    )
                }
                None => String::from(""),
            };
            let postthumbnail = match &p_met.thumbnail {
                Some(d) => format!(r#"<meta name="og:image" content="{}">"#, d),
                None => String::from(""),
            };
            let authorname = match &p_met.author {
                Some(a) => a.name.as_str(),
                None => "Unknown author.",
            };
            let mut metatags: String = match p_met.kind.as_str() {
                "post" => {
                    format!(
                        r#"
    <meta charset="UTF-8">
    <meta name="og:title" content="{}">
    <meta name="description" content="{}">
    <meta name="og:description" content="{}">
    {}
    <meta name="author" content="{}">
    <meta name="og:author" content="{}">
    <meta name="robots" content="index, follow">
    <meta name="generator" content="Cynthia {}">
                        "#,
                        &p_met.title,
                        &p_met
                            .short
                            .clone()
                            .unwrap_or(String::from("No description available.")),
                        &p_met
                            .short
                            .clone()
                            .unwrap_or(String::from("No description available.")),
                        postthumbnail,
                        authorname,
                        authorname,
                        env!("CARGO_PKG_VERSION")
                    )
                }
                _ => String::from(""),
            };
            if config::main().generator.site_baseurl != *"" {
                metatags.push_str(
                    format!(
                        r#"{1}<base href="{0}">"#,
                        config::main().generator.site_baseurl,
                        "\n"
                    )
                    .as_str(),
                )
            }
            if config::main().generator.og_sitename != *"" {
                metatags.push_str(
                    format!(
                        r#"{1}<meta property="og:site_name" content="{0}"/>"#,
                        config::main().generator.og_sitename,
                        "\n"
                    )
                    .as_str(),
                )
            }

            let mut head = format!(
                r#"
            <style>
	{}
	</style>
	{}
	{}
	<script src="https://cdn.jsdelivr.net/npm/jquery@latest/dist/jquery.min.js"></script>
    
	<title>{}&ensp;&ndash;&ensp;{}</title>
	"#,
                stylesheet, favicondec, metatags, currentmode.sitename, p_met.title
            );
            head = runnners_head(plugins.clone(), head);
            head.push_str(
                format!(
                    r#"<script>
		const pagemetainfo = JSON.parse(`{0}`);
	</script>
    "#,
                    pagemetainfojson,
                )
                .as_str(),
            );

            let pageinfosidebarthing = if (p_met.kind == *"post")
                || p_met
                    .pageinfooverride
                    .unwrap_or(currentmode.pageinfooverride.unwrap_or(false))
            {
                r#"<span class="pageinfosidebar" id="pageinfosidebartoggle" style="transition: all 1s ease-out 0s; width: 0px; font-size: 3em; bottom: 215px; display: none; text-align: right; padding: 0px; cursor: pointer;" onclick="pageinfosidebar_rollout()">&#x27A7;</span>
	<div class="pageinfosidebar" id="cynthiapageinfoshowdummyelem"></div>"#
            } else {
                ""
            };
            let data = CynthiaPageVars {
                head,
                content: contents,
                menu1: menus.menu1,
                menu2: menus.menu2,
                infoshow: String::from(pageinfosidebarthing),
            };
            let mut k = format!(
                "\n{}\n</html>",
                handlebars
                    .render_template(&source.to_string(), &data)
                    .unwrap()
            );
            k = runners_output(plugins.clone(), k);
            let j = k.replace(
                "</body>",
                format!(
                    r#"
                <script defer>
                {0}
                </script>
                </body>
                "#,
                    import_js_minified("./cynthiaFiles/assets/scripts/client.js".to_string())
                )
                .as_str(),
            );
            return format!("<!DOCTYPE html>\n<html>\n<!--\n\nGenerated and hosted through Cynthia v{}, by Strawmelonjuice.\nAlso see:\t<https://github.com/strawmelonjuice/CynthiaWebsiteEngine/blob/main/README.MD>\n\n-->\n\n\n\n\r{j}", env!("CARGO_PKG_VERSION"));
        }
    }
    // logger(3, String::from("Can't find that page."));
    contents
}

#[allow(dead_code)]

pub(crate) fn postlistcombine(
    postlist: Vec<PostlistPost>,
    menus: Menulist,
    plugins: Vec<PluginMeta>,
) -> String {
    let pgid = String::from("root");
     if postlist.is_empty() { return combine_content(pgid, str::to_string("No posts found for these criteria."), menus, plugins); };
    let mut published_jsonc = crate::read_published_jsonc();
    for p_met in &mut published_jsonc {
        if p_met.id == pgid {
            let mode_to_load = p_met
                .mode
                .get_or_insert_with(|| String::from("default"))
                .to_string();
            let p_m_met: CynthiaContentMetaDataMinimal =
                serde_json::from_str(serde_json::to_string(&p_met).unwrap().as_str()).unwrap();
            let pagemetainfojson = serde_json::to_string(&p_m_met).unwrap();
            let currentmode = crate::load_mode(mode_to_load).1;
            let stylesheet: String =
                import_css_minified(format!("./cynthiaFiles/styles/{}", currentmode.stylefile));
            let handlebarfile = format!(
                "./cynthiaFiles/templates/{}.handlebars",
                currentmode.handlebar.postlist.unwrap_or(currentmode.handlebar.page)
            )
                .to_owned();
            let source = std::fs::read_to_string(handlebarfile)
                .expect("Couldn't find or load handlebars file.");
            let handlebars = Handlebars::new();
            let favicondec = match currentmode.favicon {
                Some(d) => {
                    format!(
                        r#"<link rel="shortcut icon" href="/assets/{}" type="image/x-icon"/>"#,
                        d
                    )
                }
                None => String::from(""),
            };
            let postthumbnail = match &p_met.thumbnail {
                Some(d) => format!(r#"<meta name="og:image" content="{}">"#, d),
                None => String::from(""),
            };
            let authorname = match &p_met.author {
                Some(a) => a.name.as_str(),
                None => "Unknown author.",
            };
            let mut metatags: String = match p_met.kind.as_str() {
                "post" => {
                    format!(
                        r#"
    <meta charset="UTF-8">
    <meta name="og:title" content="{}">
    <meta name="description" content="{}">
    <meta name="og:description" content="{}">
    {}
    <meta name="author" content="{}">
    <meta name="og:author" content="{}">
    <meta name="robots" content="index, follow">
    <meta name="generator" content="Cynthia {}">
                        "#,
                        &p_met.title,
                        &p_met
                            .short
                            .clone()
                            .unwrap_or(String::from("No description available.")),
                        &p_met
                            .short
                            .clone()
                            .unwrap_or(String::from("No description available.")),
                        postthumbnail,
                        authorname,
                        authorname,
                        env!("CARGO_PKG_VERSION")
                    )
                }
                _ => String::from(""),
            };
            if config::main().generator.site_baseurl != *"" {
                metatags.push_str(
                    format!(
                        r#"{1}<base href="{0}">"#,
                        config::main().generator.site_baseurl,
                        "\n"
                    )
                        .as_str(),
                )
            }
            if config::main().generator.og_sitename != *"" {
                metatags.push_str(
                    format!(
                        r#"{1}<meta property="og:site_name" content="{0}"/>"#,
                        config::main().generator.og_sitename,
                        "\n"
                    )
                        .as_str(),
                )
            }

            let mut head = format!(
                r#"
            <style>
	{}
	</style>
	{}
	{}
	<script src="https://cdn.jsdelivr.net/npm/jquery@latest/dist/jquery.min.js"></script>

	<title>{}&ensp;&ndash;&ensp;{}</title>
	"#,
                stylesheet, favicondec, metatags, currentmode.sitename, p_met.title
            );
            head = runnners_head(plugins.clone(), head);
            head.push_str(
                format!(
                    r#"<script>
		const pagemetainfo = JSON.parse(`{0}`);
	</script>
    "#,
                    pagemetainfojson,
                )
                    .as_str(),
            );

            let pageinfosidebarthing = if (p_met.kind == *"post")
                || p_met
                .pageinfooverride
                .unwrap_or(currentmode.pageinfooverride.unwrap_or(false))
            {
                r#"<span class="pageinfosidebar" id="pageinfosidebartoggle" style="transition: all 1s ease-out 0s; width: 0px; font-size: 3em; bottom: 215px; display: none; text-align: right; padding: 0px; cursor: pointer;" onclick="pageinfosidebar_rollout()">&#x27A7;</span>
	<div class="pageinfosidebar" id="cynthiapageinfoshowdummyelem"></div>"#
            } else {
                ""
            };
            #[derive(serde::Serialize)]
            struct PageData {
                head: String,
                postlist: Vec<PostlistPost>,
                menu1: String,
                menu2: String,
                infoshow: String,
            }
            let PageData { head: head0, postlist: postlist0, menu1, menu2, infoshow } = PageData { head, postlist, menu1: menus.menu1, menu2: menus.menu2, infoshow:  String::from(pageinfosidebarthing)};
            let mut k = format!(
                "\n{}\n</html>",
                handlebars
                    .render_template(&source.to_string(), &PageData { head: head0, postlist: postlist0, menu1, menu2, infoshow })
                    .unwrap()
            );
            k = runners_output(plugins.clone(), k);
            let j = k.replace(
                "</body>",
                format!(
                    r#"
                <script defer>
                {0}
                </script>
                </body>
                "#,
                    import_js_minified("./cynthiaFiles/assets/scripts/client.js".to_string())
                )
                    .as_str(),
            );
            return format!("<!DOCTYPE html>\n<html>\n<!--\n\nGenerated and hosted through Cynthia v{}, by Strawmelonjuice.\nAlso see:\t<https://github.com/strawmelonjuice/CynthiaWebsiteEngine/blob/main/README.MD>\n\n-->\n\n\n\n\r{j}", env!("CARGO_PKG_VERSION"));
        }
    }
    // logger(3, String::from("Can't find that page."));
    combine_content(pgid, str::to_string("No posts found for these criteria."), menus, plugins)
}

fn runner_content(plugins: Vec<PluginMeta>, mut contents: String) -> String {
    for plugin in plugins {
        match &plugin.runners.modify_body_html {
            Some(p) => {
                let cont_1 = serde_json::to_string(&contents).unwrap();
                let mut cont_2 = cont_1.as_str().chars();
                cont_2.next();
                cont_2.next_back();
                let con_s = cont_2.as_str();
                let cmdjson: String = p.execute.replace(r#"{{{input}}}"#, con_s);
                let cmds: Vec<String> = match serde_json::from_str(cmdjson.as_str()) {
                    Ok(cmds) => cmds,
                    Err(e) => {
                        logger::jsr_error(format!(
                            "Could not parse JSON for plugin {}: {}",
                            plugin.name, e
                        ));
                        [].to_vec()
                    }
                };
                // .unwrap_or(["returndirect", contents.as_str()].to_vec());
                let mut cmd: Vec<&str> = vec![];
                for com in &cmds {
                    cmd.push(match com.as_str() {
                        "kamkdxcvjgCVJGVvdbvcgcvgdvd" => contents.as_str(),
                        a => a,
                    });
                }
                if p.type_field == *"js" {
                    contents = jsr::noderunner(cmd, format!("./plugins/{}/", plugin.name).into());
                } else {
                    logger::general_error( format!("{} is using a '{}' type allternator, which is not supported by this version of cynthia", plugin.name, p.type_field))
                }
            }
            None => {}
        }
    }
    contents
}

fn runners_output (plugins: Vec<PluginMeta>, mut k: String) -> String {
    for plugin in plugins {
        match &plugin.runners.modify_output_html {
            Some(p) => {
                let cont_1 = serde_json::to_string(&k).unwrap();
                let mut cont_2 = cont_1.as_str().chars();
                cont_2.next();
                cont_2.next_back();
                let con_s = cont_2.as_str();
                let cmdjson: String = p.execute.replace(r#"{{{input}}}"#, con_s);
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
                if p.type_field == *"js" {
                    k = jsr::noderunner(cmd, format!("./plugins/{}/", plugin.name).into());
                } else {
                    logger::general_error( format!("{} is using a '{}' type modifier, which is not supported by this version of cynthia", plugin.name, p.type_field))
                }
            }
            None => {}
        }
    }
    k
}

fn runnners_head(plugins: Vec<PluginMeta>, mut head: String) -> String {
    for plugin in plugins {
        match &plugin.runners.modify_head_html {
            Some(p) => {
                let cont_1 = serde_json::to_string(&head).unwrap();
                let mut cont_2 = cont_1.as_str().chars();
                cont_2.next();
                cont_2.next_back();
                let con_s = cont_2.as_str();
                let cmdjson: String = p.execute.replace(r#"{{{input}}}"#, con_s);
                let cmds: Vec<String> = serde_json::from_str(cmdjson.as_str()).unwrap_or(
                    [
                        "returndirect".to_string(),
                        crate::escape_json(&head).to_string(),
                    ]
                        .to_vec(),
                );
                let mut cmd: Vec<&str> = vec![];
                for com in &cmds {
                    cmd.push(com.as_str());
                }
                if p.type_field == *"js" {
                    head = jsr::noderunner(cmd, format!("./plugins/{}/", plugin.name).into());
                } else {
                    logger::general_error( format!("{} is using a '{}' type modifier, which is not supported by this version of cynthia", plugin.name, p.type_field))
                }
            }
            None => {}
        }
    }
    head
}

fn escape_known_problematic_chars(s: String) -> String {
    s.replace('—', "&#8212;") // Em-dash
        .replace('–', "&#8211;") // En-dash
        .replace('€', "&#8364;") // Euro sign
                                 // etc...
}
