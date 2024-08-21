/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */

use super::{CynthiaConf, CynthiaConfig};
use crate::jsrun;
use crate::jsrun::RunJSAndDeserializeResult;
use crate::tell::CynthiaColors;
use std::path::PathBuf;
use std::{fs, process};

const CONFIG_LOCATIONS: [&str; 4] = [
    "CynthiaConfig.js",
    "Cynthia.dhall",
    "Cynthia.toml",
    "Cynthia.jsonc",
];
pub(crate) enum ConfigLocations {
    Js(PathBuf),
    Dhall(PathBuf),
    Toml(PathBuf),
    JsonC(PathBuf),
}

impl ConfigLocations {
    fn clone(&self) -> ConfigLocations {
        match self {
            ConfigLocations::Js(p) => ConfigLocations::Js(p.clone()),
            ConfigLocations::Dhall(p) => ConfigLocations::Dhall(p.clone()),
            ConfigLocations::Toml(p) => ConfigLocations::Toml(p.clone()),
            ConfigLocations::JsonC(p) => ConfigLocations::JsonC(p.clone()),
        }
    }
    fn exists(&self) -> bool {
        match self {
            ConfigLocations::Js(p) => p.exists(),
            ConfigLocations::Dhall(p) => p.exists(),
            ConfigLocations::Toml(p) => p.exists(),
            ConfigLocations::JsonC(p) => p.exists(),
        }
    }
}

fn choose_config_location() -> ConfigLocations {
    let unfound = || {
        eprintln!("Could not find cynthia-configuration at `{}`! Have you initialised a Cynthia setup here? To do so, run `{}`.",
                  std::env::current_dir().unwrap().clone().to_string_lossy().replace("\\\\?\\", "").color_bright_cyan(),
                  "cynthiaweb init".color_lime());
        process::exit(1);
    };
    let cd = std::env::current_dir().unwrap();
    // In order of preference for Cynthia. I personally prefer TOML, but Cynthia would prefer Dhall. Besides, Dhall is far more powerful.
    // JS, Dhall, TOML, jsonc
    let config_locations: [ConfigLocations; 4] = [
        ConfigLocations::Js(cd.join("CynthiaConfig.js")),
        ConfigLocations::Dhall(cd.join("Cynthia.dhall")),
        ConfigLocations::Toml(cd.join("Cynthia.toml")),
        ConfigLocations::JsonC(cd.join("Cynthia.jsonc")),
    ];
    // let chosen_config_location = _chonfig_locations.iter().position(|p| p.exists());
    // Whichever config file is found first, is the one that is chosen. Put it in the enum.
    let chosen_config_location: ConfigLocations = {
        let mut a: Option<usize> = None;
        for (i, p) in config_locations.iter().enumerate() {
            if p.exists() {
                a = Some(i);
                break;
            }
        }
        match a {
            Some(p) => config_locations[p].clone(),
            None => {
                unfound();
                unreachable!()
            }
        }
    };
    chosen_config_location
}

pub(crate) fn load_config() -> CynthiaConf {
    use jsonc_parser::parse_to_serde_value as preparse_jsonc;
    let chosen_config_location = choose_config_location();
    return match chosen_config_location {
        ConfigLocations::JsonC(cynthiaconfpath) => {
            println!(
                "{} Loading: {}",
                "[Config]".color_lime(),
                cynthiaconfpath
                    .clone()
                    .to_string_lossy()
                    .replace("\\\\?\\", "")
                    .color_bright_cyan()
            );
            let unparsed_json = match fs::read_to_string(cynthiaconfpath.clone()) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!(
                        "{}\n\nReason:\n{}",
                        format!(
                            "Could not interpret cynthia-configuration at `{}`!",
                            cynthiaconfpath
                                .clone()
                                .to_string_lossy()
                                .replace("\\\\?\\", "")
                        )
                        .color_bright_red(),
                        e
                    );
                    process::exit(1);
                }
            };

            let preparsed: Option<serde_json::Value> =
                match preparse_jsonc(unparsed_json.as_str(), &Default::default()) {
                    Ok(t) => t,
                    Err(e) => {
                        eprintln!(
                            "{}\n\nReason:\n{}",
                            format!(
                                "Could not interpret cynthia-configuration at `{}`!",
                                cynthiaconfpath
                                    .clone()
                                    .to_string_lossy()
                                    .replace("\\\\?\\", "")
                            )
                            .color_bright_red(),
                            e
                        );
                        process::exit(1);
                    }
                };
            match preparsed {
                Some(g) => match serde_json::from_value(g) {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!(
                            "{}\n\nReason:\n{}",
                            format!(
                                "Could not interpret cynthia-configuration at `{}`!",
                                cynthiaconfpath
                                    .clone()
                                    .to_string_lossy()
                                    .replace("\\\\?\\", "")
                            )
                            .color_bright_red(),
                            e
                        );
                        process::exit(1);
                    }
                },
                None => {
                    eprintln!(
                        "{}\n\nReason:\n{}",
                        format!(
                            "Could not interpret cynthia-configuration at `{}`!",
                            cynthiaconfpath
                                .clone()
                                .to_string_lossy()
                                .replace("\\\\?\\", "")
                        )
                        .color_error_red(),
                        "ERROR: ".color_bright_red()
                    );
                    process::exit(1);
                }
            }
        }
        ConfigLocations::Toml(cynthiaconfpath) => {
            println!(
                "{} Loading: {}",
                "[Config]".color_lime(),
                cynthiaconfpath
                    .clone()
                    .to_string_lossy()
                    .replace("\\\\?\\", "")
                    .color_bright_cyan()
            );
            match fs::read_to_string(cynthiaconfpath.clone()) {
                Ok(g) => match toml::from_str(&g) {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!(
                            "{}\n\nReason:\n{}",
                            format!(
                                "Could not interpret cynthia-configuration at `{}`!",
                                cynthiaconfpath
                                    .clone()
                                    .to_string_lossy()
                                    .replace("\\\\?\\", "")
                            )
                            .color_bright_red(),
                            e
                        );
                        process::exit(1);
                    }
                },
                Err(e) => {
                    eprintln!(
                        "{}\n\nReason:\n{}",
                        format!(
                            "Could not interpret cynthia-configuration at `{}`!",
                            cynthiaconfpath
                                .clone()
                                .to_string_lossy()
                                .replace("\\\\?\\", "")
                        )
                        .color_bright_red(),
                        format!("{}", e).color_error_red()
                    );
                    process::exit(1);
                }
            }
        }
        ConfigLocations::Dhall(cynthiaconfpath) => {
            println!(
                "{} Loading: {}",
                "[Config]".color_lime(),
                cynthiaconfpath
                    .clone()
                    .to_string_lossy()
                    .replace("\\\\?\\", "")
                    .color_bright_cyan()
            );
            match fs::read_to_string(cynthiaconfpath.clone()) {
                Ok(g) => match serde_dhall::from_str(&g).parse() {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!(
                            "{}\n\nReason:\n{}",
                            format!(
                                "Could not interpret cynthia-configuration at `{}`!",
                                cynthiaconfpath
                                    .clone()
                                    .to_string_lossy()
                                    .replace("\\\\?\\", "")
                            )
                            .color_bright_red(),
                            e
                        );
                        process::exit(1);
                    }
                },
                Err(e) => {
                    eprintln!(
                        "{}\n\nReason:\n{}",
                        format!(
                            "Could not interpret cynthia-configuration at `{}`!",
                            cynthiaconfpath
                                .clone()
                                .to_string_lossy()
                                .replace("\\\\?\\", "")
                        )
                        .color_bright_red(),
                        format!("{}", e).color_error_red()
                    );
                    process::exit(1);
                }
            }
        }
        ConfigLocations::Js(cynthiaconfpath) => {
            println!(
                "{} Loading: {}",
                "[Config]".color_lime(),
                cynthiaconfpath
                    .clone()
                    .to_string_lossy()
                    .replace("\\\\?\\", "")
                    .color_bright_cyan()
            );
            let unparsed_js = match fs::read_to_string(cynthiaconfpath.clone()) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!(
                        "{}\n\nReason:\n{}",
                        format!(
                            "Could not interpret cynthia-configuration at `{}`!",
                            cynthiaconfpath
                                .clone()
                                .to_string_lossy()
                                .replace("\\\\?\\", "")
                        )
                        .color_bright_red(),
                        e
                    );
                    process::exit(1);
                }
            };
            match jsrun::run_js_and_deserialize::<CynthiaConf>(unparsed_js.as_str()) {
                RunJSAndDeserializeResult::Ok(p) => p,
                RunJSAndDeserializeResult::JsError(e) => {
                    eprintln!(
                        "{}\n\nReason:\n{}",
                        format!(
                            "Could not interpret cynthia-configuration at `{}`!",
                            cynthiaconfpath
                                .clone()
                                .to_string_lossy()
                                .replace("\\\\?\\", "")
                        )
                        .color_bright_red(),
                        e
                    );
                    process::exit(1);
                }
                RunJSAndDeserializeResult::SerdeError(e) => {
                    eprintln!(
                        "{}\n\nReason:\n{}",
                        format!(
                            "Could not interpret cynthia-configuration at `{}`!",
                            cynthiaconfpath
                                .clone()
                                .to_string_lossy()
                                .replace("\\\\?\\", "")
                        )
                        .color_bright_red(),
                        e
                    );
                    process::exit(1);
                }
            }
        }
    };
}

pub(crate) fn save_config(to_ex: &str, config: CynthiaConf) -> PathBuf {
    let to_ =
        if to_ex.to_lowercase().as_str() == "js" || to_ex.to_lowercase().as_str() == "javascript" {
            String::from("js")
        } else {
            to_ex.to_lowercase()
        };
    let to = to_.as_str();
    {
        let chosen_config_location = choose_config_location_option();
        match chosen_config_location {
            Some(ConfigLocations::Js(_)) => {
                if to == "js" {
                    eprintln!(
                        "{} You are trying to convert a JavaScript configuration to JavaScript. This is not possible.",
                        "error:".color_red()
                    );
                    process::exit(1);
                }
            }
            Some(ConfigLocations::Dhall(_)) => {
                if to == "dhall" {
                    eprintln!(
                        "{} You are trying to convert a Dhall configuration to Dhall. This is not possible.",
                        "error:".color_red()
                    );
                    process::exit(1);
                }
            }
            Some(ConfigLocations::Toml(_)) => {
                if to == "toml" {
                    eprintln!(
                        "{} You are trying to convert a TOML configuration to TOML. This is not possible.",
                        "error:".color_red()
                    );
                    process::exit(1);
                }
            }
            Some(ConfigLocations::JsonC(_)) => {
                if to == "jsonc" {
                    eprintln!(
                        "{} You are trying to convert a JSONC configuration to JSONC. This is not possible.",
                        "error:".color_red()
                    );
                    process::exit(1);
                }
            }
            None => {}
        }
    }
    let cynthiaconfdoclink = r#"https://strawmelonjuice.github.io/CynthiaWebsiteEngine/Admins/configuration/CynthiaConf.html"#;
    let args: Vec<String> = std::env::args().collect();
    let cd = std::env::current_dir().unwrap();
    // as a tuple, the first element is the key, the second is the comment, the third is the key in the config.
    let comments: [(&str, &str, &str); 32] = [
        ("port", "The port on which Cynthia hosts, since Cynthia was designed to be reverse-proxied, this port is usually higher than 1000.", "port"),
        ("cache", "The cache configuration for Cynthia.", "cache"),
            ("lifetimes", "These rules are set for a reason: The higher they are set, the less requests we have to do to Node, external servers, etc.\nHigher caching might consume a lot of memory or storage and crash the system.\nCaching can speed up Cynthia a whole lot, so think wisely before you change any of these numbers!", "cache.lifetimes"),
                ("stylesheets", "How long (in seconds) to cache a CSS file after having minified and served it.", "cache.lifetimes.stylesheets"),
                ("javascript", "How long (in seconds) to cache a JS file after having minified and served it.", "cache.lifetimes.javascript"),
                ("forwarded", "How long (in seconds) to cache an external output after having used it.", "cache.lifetimes.forwarded"),
                ("served", "How long should a fully-ready-to-be-served page be cached?", "cache.lifetimes.served"),
        ("runtimes", "These are the runtimes that Cynthia uses to run its scripts.\nTo run Cynthia with selected runtimes, point them to the correct binaries.", "runtimes"),
            ("ext_js_rt", "The path to the external JS runtime binary, used for running JavaScript code. Recommended runtime to use is Bun. Also see <https://bun.sh/>.", "runtimes.ext_js_rt"),
        ("site", "The site configuration for Cynthia. This is used to generate the site itself. And set things like metatags, etc.", "site"),
            ("notfound_page", "The id of a 404 page, which is then served when a page is not found.", "site.notfound_page"),
            ("meta", "Meta settings for generation, not setting 'how', but 'what' to generate.", "site.meta"),
                ("enable_tags", "Enables or disables pagetags in HTML metatags,\nthese are officially supposed to be good for\nfinding a website, but have been known to\nget nerfed by Google, considering them spam.", "site.meta.enable_tags"),
                ("enable_search", "Whether to enable search or not. If enabled, search will be used to generate pages.", "site.meta.enable_search"),
                ("enable_sitemap", "Whether to enable sitemap or not. If enabled, sitemap will be used to generate pages.", "site.meta.enable_sitemap"),
                ("enable_rss", "Whether to enable RSS or not. If enabled, RSS will be used to generate pages.", "site.meta.enable_rss"),
                ("enable_atom", "Whether to enable Atom or not. If enabled, Atom will be used to generate pages.", "site.meta.enable_atom"),
            ("site_baseurl", "The base URL of the site, used for generating links.", "site.site_baseurl"),
            ("og_sitename", "Site name for the site, this is different than the site name set in scenes, as it is mostly used for embeds, and so get's cached on url.", "site.og_sitename"),
        ("logs", "The log configuration for Cynthia.", "logs"),
            ("term_loglevel", "The minimum level of importance (1-5) before Cynthia logs to the terminal.", "logs.term_loglevel"),
            ("file_loglevel", "The minimum level of importance (1-5) before Cynthia logs to a file.", "logs.file_loglevel"),
            ("log_file", "The file Cynthia logs to.", "logs.log_file"),
        ("scenes", "Scenes allow Cynthia to switch it's behaviour and themes completely for certain pages.", "scenes"),
                ("name", "The id of the scene, used for linking. Set to `default` for the default scene.", "scenes.name"),
                ("sitename", "The name Cynthia uses for presenting the site when using this scene.", "scenes.sitename"),
                ("script", "(Optional) A script that is served on pages using this scene.", "scenes.script"),
                ("stylefile", "(Optional) A CSS file that is served on pages using this scene.", "scenes.stylefile"),
                ("templates", "The template of the scene, used for display.", "scenes.templates"),
                    ("page", "The handlebars template for serving pages using this sceme", "scenes.templates.page"),
                    ("post", "The handlebars template for serving posts using this sceme", "scenes.templates.post"),
                    ("postlist", "The handlebars template for serving postlist pages using this sceme", "scenes.templates.postlist"),
    ];
    // JSONC is generated multiple times, so we need to make a function for it.
    // This function is used to generate JSONC.
    // It is also used to generate the base for the javascript version.
    let anyways_this_is_jsonc = |config: CynthiaConf| -> String {
        let comment_this = |item: &str| -> String {
            let mut o = format!("// todo: Comment this:\n\"{}\":", item);
            for (x, y, z) in comments.clone().iter() {
                if z == &item {
                    if y.contains("\n") {
                        // If we have a multiline comment, we need to format it correctly.
                        o = format!("/*\n{}\n*/\n\"{}\":", y, x)
                    } else {
                        // If we have a single-line comment, we need to format it as one.
                        o = format!("// {}\n\"{}\":", y, x)
                    }
                } else {
                    continue;
                }
            }
            o.clone()
        };
        serde_json::to_string_pretty(&config)
            .unwrap()
            .replace("\"port\":", &comment_this("port"))
            .replace("\"cache\":", &comment_this("cache"))
            .replace("\"lifetimes\":", &comment_this("cache.lifetimes"))
            .replace("\"forwarded\":", &comment_this("cache.lifetimes.forwarded"))
            .replace(
                "\"javascript\":",
                &comment_this("cache.lifetimes.javascript"),
            )
            .replace("\"served\":", &comment_this("cache.lifetimes.served"))
            .replace(
                "\"stylesheets\":",
                &comment_this("cache.lifetimes.stylesheets"),
            )
            .replace("\"runtimes\":", &comment_this("runtimes"))
            .replace("\"ext_js_rt\":", &comment_this("runtimes.ext_js_rt"))
            .replace("\"pages\":", &comment_this("pages"))
            .replace("\"notfound_page\":", &comment_this("site.notfound_page"))
            .replace("\"site\":", &comment_this("site"))
            .replace("\"meta\":", &comment_this("site.meta"))
            .replace("\"enable_tags\":", &comment_this("site.meta.enable_tags"))
            .replace(
                "\"enable_search\":",
                &comment_this("site.meta.enable_search"),
            )
            .replace(
                "\"enable_sitemap\":",
                &comment_this("site.meta.enable_sitemap"),
            )
            .replace("\"enable_rss\":", &comment_this("site.meta.enable_rss"))
            .replace("\"enable_atom\":", &comment_this("site.meta.enable_atom"))
            .replace("\"site_baseurl\":", &comment_this("site.site_baseurl"))
            .replace("\"og_sitename\":", &comment_this("site.og_sitename"))
            .replace("\"logs\":", &comment_this("logs"))
            .replace("\"term_loglevel\":", &comment_this("logs.term_loglevel"))
            .replace("\"file_loglevel\":", &comment_this("logs.file_loglevel"))
            .replace("\"log_file\":", &comment_this("logs.log_file"))
            .replace("\"scenes\":", &comment_this("scenes"))
            .replace("\"name\":", &comment_this("scenes.name"))
            .replace("\"sitename\":", &comment_this("scenes.sitename"))
            .replace("\"script\":", &comment_this("scenes.script"))
            .replace("\"stylefile\":", &comment_this("scenes.stylefile"))
            .replace("\"templates\":", &comment_this("scenes.templates"))
            .replace("\"page\":", &comment_this("scenes.templates.page"))
            .replace("\"post\":", &comment_this("scenes.templates.post"))
            .replace("\"postlist\":", &comment_this("scenes.templates.postlist"))
    };

    let config_serialised: String = match to {
        "javascript" | "js" => {
            format!("/*\n\tCynthiaConfig.js\n\n\n\tThis is the configuration file for Cynthia. It is written in Javascript, a scripting language.\n\tThis kind of CynthiaConfig is the most powerful and flexible,\n\tbut also the most complex. It is recommended for advanced users\n\twho want to take full control of Cynthia's behavior.\n\n\n\tMore info about this config can be found on <{cynthiaconfdoclink}>\n\n\n\tTo convert it to another config language, use the `cynthiaweb convert` command.\n*/\nlet myCynthiaConfig = {};\n\n// We must return the configuration object at the end of the file.\nreturn myCynthiaConfig;\n",
                    regex::Regex::new(r#""([^"]*)":"#).unwrap().replace_all(&anyways_this_is_jsonc(config.hard_clone()), "\t$1:")
            )
        }
        "dhall" => {
            /*
            Dhall is a bit more complex, so we need to do some extra work here.
            Besides, we need to add some comments to the Dhall file.
            */
            let comment_this = |item: &str| -> String {
                let mut o = format!("-- todo: Comment this:\n  {} =", item);
                for (x, y, z) in comments.clone().iter() {
                    if z == &item {
                        if y.contains("\n") {
                            // If we have a multiline comment, we need to format it correctly.
                            o = format!("{{-\n{}\n-}}\n {} =", y, x)
                        } else {
                            // If we have a single-line comment, we need to format it as one.
                            o = format!("-- {}\n  {} =", y, x)
                        }
                    } else {
                        continue;
                    }
                }
                o.clone()
            };
            format!("{{\n{{-\n\tThis is the configuration file for Cynthia. It is written in Dhall, a Haskell-like language that is able to contain functions and types.\n\tMore info about this config can be found on <{cynthiaconfdoclink}>\n\n\tTo convert it to another config language, use the `cynthiaweb convert` command.\n-}}\n{}",
                serde_dhall::serialize(&config)
                    .static_type_annotation()
                    .to_string()
                    .unwrap()
                    .chars()
                    .skip(1)
                    .collect::<String>()
                    // .replace(",", ",\n")
                    .replace(",", "\n,")
                    .replace("{", "{\n")
                    .replace("}", "\n}\n")
                    .replace("\n", "\n ")
                    .replace(" port =", &comment_this("port"))
                    .replace(" cache =", &comment_this("cache"))
                    .replace(
                        " lifetimes =",
                        &comment_this("cache.lifetimes")
                    )
                        .replace(" forwarded =", &comment_this("cache.lifetimes.forwarded"))
                        .replace(" javascript =", &comment_this("cache.lifetimes.javascript"))
                        .replace(" served =", &comment_this("cache.lifetimes.served"))
                        .replace(" stylesheets =", &comment_this("cache.lifetimes.stylesheets"))
                    .replace(" runtimes =", &comment_this("runtimes"))
                        .replace(" node =", &comment_this("runtimes.ext_js_rt"))
                    .replace(" pages =", &comment_this("pages"))
                        .replace(" notfound_page =", &comment_this("site.notfound_page"))
                    .replace(" site =", &comment_this("site"))
                        .replace(" meta =", &comment_this("site.meta"))
                            .replace(" enable_tags =", &comment_this("site.meta.enable_tags"))
                            .replace(" enable_search =", &comment_this("site.meta.enable_search"))
                            .replace(" enable_sitemap =", &comment_this("site.meta.enable_sitemap"))
                            .replace(" enable_rss =", &comment_this("site.meta.enable_rss"))
                            .replace(" enable_atom =", &comment_this("site.meta.enable_atom"))
                        .replace(" site_baseurl =", &comment_this("site.site_baseurl"))
                        .replace(" og_sitename =", &comment_this("site.og_sitename"))
                    .replace(" logs =", &comment_this("logs"))
                        .replace(" term_loglevel =", &comment_this("logs.term_loglevel"))
                        .replace(" file_loglevel =", &comment_this("logs.file_loglevel"))
                        .replace(" log_file =", &comment_this("logs.log_file"))
                    .replace(" scenes =", &comment_this("scenes"))
                        .replace(" name =", &comment_this("scenes.name"))
                        .replace(" sitename =", &comment_this("scenes.sitename"))
                        .replace(" script =", &comment_this("scenes.script"))
                        .replace(" stylefile =", &comment_this("scenes.stylefile"))
                        .replace(" templates =", &comment_this("scenes.templates"))
                            .replace(" page =", &comment_this("scenes.templates.page"))
                            .replace(" post =", &comment_this("scenes.templates.post"))
                            .replace(" postlist =", &comment_this("scenes.templates.postlist"))
            )
        }
        "toml" => {
            let comment_this = |item: &str| -> String {
                let mut o = format!("# todo: Comment this:\n{} = ", item);
                for (x, y, z) in comments.clone().iter() {
                    if z == &item {
                        if y.contains("\n") {
                            // If we have a multiline comment, TOML doesn't. So... we need to format it correctly.
                            o = format!("# {}\n{} = ", y.replace("\n", "\n# "), x)
                        } else {
                            // If we have a single-line comment, we need to format it as one.
                            o = format!("# {}\n{} = ", y, x)
                        }
                    } else {
                        continue;
                    }
                }
                o.clone()
            };
            format!("# Cynthia.toml\n# \n# This is the configuration file for Cynthia. It is written in TOML, a YAML-like language that is focused on user readability.\n# More info about this config can be found on <{cynthiaconfdoclink}>\n# \n# \n# To convert it to another config language, use the `cynthiaweb convert` command.\n\n\n{}", toml::to_string_pretty(&config)
                .unwrap()
                .replace("\n","\n ")
                .replace(" port = ", &comment_this("port"))
                .replace(
                    " [cache.lifetimes]",
                    comment_this("cache.lifetimes")
                        .replace("lifetimes = ", "[cache.lifetimes]")
                        .as_str(),
                )
                .replace(" forwarded = ", &comment_this("cache.lifetimes.forwarded"))
                .replace(" javascript = ", &comment_this("cache.lifetimes.javascript"))
                .replace(" served = ", &comment_this("cache.lifetimes.served"))
                .replace(" stylesheets = ", &comment_this("cache.lifetimes.stylesheets"))
                .replace(
                    " [runtimes]",
                    comment_this("runtimes")
                        .replace("runtimes = ", "[runtimes]")
                        .as_str(),
                )
                .replace(" node = ", &comment_this("runtimes.ext_js_rt"))
                .replace(
                    " [pages]",
                    comment_this("pages")
                        .replace("pages = ", "[pages]")
                        .as_str(),
                )
                .replace(" notfound_page = ", &comment_this("site.notfound_page"))
                .replace(
                    " [site]",
                    comment_this("site")
                        .replace("site = ", "[site]")
                        .as_str(),
                )
                .replace(
                    " [site.meta]",
                    comment_this("site.meta")
                        .replace("meta = ", "[site.meta]")
                        .as_str(),
                )
                .replace(" enable_tags = ", &comment_this("site.meta.enable_tags"))
                .replace(" enable_search = ", &comment_this("site.meta.enable_search"))
                .replace(" enable_sitemap = ", &comment_this("site.meta.enable_sitemap"))
                .replace(" enable_rss = ", &comment_this("site.meta.enable_rss"))
                .replace(" enable_atom = ", &comment_this("site.meta.enable_atom"))
                .replace(" site_baseurl = ", &comment_this("site.site_baseurl"))
                .replace(" og_sitename = ", &comment_this("site.og_sitename"))
                .replace(
                    " [logs]",
                    comment_this("logs")
                        .replace("logs = ", "[logs]")
                        .as_str(),
                )
                    .replace(" term_loglevel = ", &comment_this("logs.term_loglevel"))
                    .replace(" file_loglevel = ", &comment_this("logs.file_loglevel"))
                    .replace(" log_file = ", &comment_this("logs.log_file"))
                .replace(" [[scenes]]", comment_this("scenes").replace("scenes = ", "[[scenes]]").as_str())
                    .replace(" name = ", &comment_this("scenes.name"))
                    .replace(" sitename = ", &comment_this("scenes.sitename"))
                    .replace(" script = ", &comment_this("scenes.script"))
                    .replace(" stylefile = ", &comment_this("scenes.stylefile"))
                    .replace(
                        " [scenes.templates]",
                        comment_this("scenes.templates")
                            .replace("templates = ", "[scenes.templates]")
                            .as_str(),
                    )
                        .replace(" page = ", &comment_this("scenes.templates.page"))
                        .replace(" post = ", &comment_this("scenes.templates.post"))
                        .replace(" postlist = ", &comment_this("scenes.templates.postlist"))
            )
        }
        "jsonc" => {
            format!("/*\n\tCynthia.jsonc\n\n\tThis is the configuration file for Cynthia. It is written in JSONC, a JSON-like language that is focused on user readability.\n\tMore info about this config can be found on <{cynthiaconfdoclink}>\n\n\tTo convert it to another config language, use the `cynthiaweb convert` command.\n*/\n\n{}",
                    anyways_this_is_jsonc(config.hard_clone()))
        }
        _ => {
            eprintln!(
                "{} Could not interpret format `{}`! Please use `jsonc`, `dhall` or `toml`.",
                "error:".color_red(),
                to
            );
            process::exit(1);
        }
    };
    let to_file = if to == "js" {
        cd.join("CynthiaConfig.js")
    } else {
        cd.join("Cynthia.".to_string() + to)
    };
    match fs::write(to_file.clone(), config_serialised) {
        Ok(_) => {
            if args.get(1) == Some(&String::from("convert")) {
                println!(
                    "{} Successfully exported the configuration to {}!",
                    "Success:".color_green(),
                    to_file
                        .clone()
                        .to_string_lossy()
                        .replace("\\\\?\\", "")
                        .color_bright_cyan()
                );
                if args.get(3).unwrap_or(&String::from("")).as_str() == "-k" {
                    println!(
                    "{} Exiting without deleting old formats ({} flag). This is not recommended.",
                    "Info:".color_yellow(),
                    "-k".color_bright_yellow()
                );
                    process::exit(0);
                }
            } else {
                return to_file;
            }
        }
        Err(e) => {
            eprintln!(
                "{} Could not write the configuration to `{}`! Error: {}",
                "error:".color_red(),
                cd.join("Cynthia.".to_string() + to)
                    .to_string_lossy()
                    .replace("\\\\?\\", ""),
                e
            );
            process::exit(1);
        }
    };
    // Remove old format(s)

    let mut config_locations: Vec<PathBuf> = CONFIG_LOCATIONS.iter().map(|p| cd.join(p)).collect();
    config_locations.retain(|p| p.exists());
    config_locations.retain(|p| p != &to_file);
    for p in config_locations {
        match fs::remove_file(p.clone()) {
            Ok(_) => {}
            Err(e) => {
                eprintln!(
                    "{} Could not remove the old configuration file at `{}`! Error: {}",
                    "error:".color_red(),
                    p.to_string_lossy().replace("\\\\?\\", ""),
                    e
                );
                process::exit(1);
            }
        }
    }

    process::exit(0);
}

pub(crate) fn choose_config_location_option() -> Option<ConfigLocations> {
    let cd = std::env::current_dir().unwrap();
    // In order of preference for Cynthia. I personally prefer TOML, but Cynthia would prefer Dhall. Besides, Dhall is far more powerful.
    // JS, Dhall, TOML, jsonc
    let config_locations: [ConfigLocations; 4] = [
        ConfigLocations::Js(cd.join("CynthiaConfig.js")),
        ConfigLocations::Dhall(cd.join("Cynthia.dhall")),
        ConfigLocations::Toml(cd.join("Cynthia.toml")),
        ConfigLocations::JsonC(cd.join("Cynthia.jsonc")),
    ];
    // let chosen_config_location = _chonfig_locations.iter().position(|p| p.exists());
    // Whichever config file is found first, is the one that is chosen. Put it in the enum.

    let mut a: Option<usize> = None;
    for (i, p) in config_locations.iter().enumerate() {
        if p.exists() {
            a = Some(i);
            break;
        }
    }
    a.map(|p| config_locations[p].clone())
}
