use handlebars::Handlebars;

use crate::{jsr, logger::logger, structs::*};

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
                let cmds: Vec<String> = serde_json::from_str(cmdjson.as_str()).unwrap();
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
                    logger(5, format!("{} is using a '{}' type allternator, which is not supported by this version of cynthia", plugin.name, p.type_field))
                }
            }
            None => {}
        }
    }
    let mut published_jsonc = crate::read_published_jsonc();
    for post in &mut published_jsonc {
        if post.id == pgid {
            let mode_to_load = post
                .mode
                .get_or_insert_with(|| String::from("default"))
                .to_string();
            let pagemetainfojson = serde_json::to_string(&post).unwrap();
            let currentmode = crate::load_mode(mode_to_load).1;
            let stylesheet: String = std::fs::read_to_string(
                std::path::Path::new("./cynthiaFiles/styles/").join(currentmode.stylefile),
            )
            .unwrap_or(String::from(""));
            let handlebarfile = format!(
                "./cynthiaFiles/templates/{}.handlebars",
                if post.kind == "post" {
                    currentmode.handlebar.post
                } else {
                    currentmode.handlebar.page
                }
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
	<script src="https://cdn.jsdelivr.net/npm/jquery@latest/dist/jquery.min.js"></script>
	<title>{1} &ndash; {2}</title>
	"#,
                stylesheet, post.title, currentmode.sitename
            );
            for plugin in plugins.clone() {
                match &plugin.runners.modify_head_html {
                    Some(p) => {
                        let handlebars = Handlebars::new();
                        let mut data = std::collections::BTreeMap::new();
                        data.insert("input".to_string(), crate::escape_json(&head));
                        let cmdjson: String = handlebars
                            .render_template(&p.execute, &data)
                            .unwrap_or(format!("[ \"returndirect\", \"f{}\" ]", head));
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
                            head =
                                jsr::noderunner(cmd, format!("./plugins/{}/", plugin.name).into());
                        } else {
                            logger(5, format!("{} is using a '{}' type modifier, which is not supported by this version of cynthia", plugin.name, p.type_field))
                        }
                    }
                    None => {}
                }
            }
            head.push_str(
                format!(
                    r#"<script>
		const pagemetainfo = JSON.parse(`{0}`);
	</script>"#,
                    pagemetainfojson
                )
                .as_str(),
            );
            let pageinfosidebarthing = if post.kind == *"post" {
                r#"<span class="pageinfosidebar" id="pageinfosidebartoggle" style="transition: all 1s ease-out 0s; width: 0px; font-size: 3em; bottom: 215px; display: none; text-align: right; padding: 0px; cursor: pointer;" onclick="pageinfosidebar_rollout()">➧</span>
	<div class="pageinfosidebar" id="cynthiapageinfoshowdummyelem"></div>"#
            } else {
                ""
            };
            contents.push_str(pageinfosidebarthing);
            let data = CynthiaPageVars {
                head,
                content: contents,
                menu1: menus.menu1,
                menu2: menus.menu2,
                infoshow: String::from(""),
            };
            let mut k = format!(
                "\n{}\n\n\n\n<script src=\"/assets/scripts/client.js\"></script>\n\n</html>",
                handlebars
                    .render_template(&source.to_string(), &data)
                    .unwrap(),
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
                            logger(5, format!("{} is using a '{}' type modifier, which is not supported by this version of cynthia", plugin.name, p.type_field))
                        }
                    }
                    None => {}
                }
            }
            return format!("<!DOCTYPE html>\n<html>\n<!--\n\nGenerated and hosted through Cynthia v{}, by Strawmelonjuice.\nAlso see:\t<https://github.com/strawmelonjuice/CynthiaCMS-JS/blob/main/README.MD>\n\n-->\n\n\n\n\r{k}", env!("CARGO_PKG_VERSION"));
        }
    }
    // logger(3, String::from("Can't find that page."));
    contents
}
