/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */

use std::fs;

use actix_web::HttpResponse;
use colored::Colorize;
use curl::easy::Easy;
use markdown::{to_html_with_options, CompileOptions, Options};

use crate::config::CynthiaConf;
use crate::files::{cacheplacer, cacheretriever};
use crate::{logger, structs::*};

use self::postlists::postlist_table_gen;

pub mod combiner;
mod postlists;

pub(crate) fn s_server(
    filter_s: &String,
    probableurl: String,
    plugins: Vec<PluginMeta>,
) -> HttpResponse {
    let filters: PostListFilter = PostListFilter {
        category: None,
        tag: None,
        searchline: Some(filter_s.to_string()),
    };
    let cynres = combiner::combine_content(
        String::from("root"),
        postlist_table_gen(Postlist {
            filters: Some(filters),
        }),
        generate_menus(String::from("root"), &probableurl),
        plugins.clone(),
    );

    if cynres == *"unknownexeception" {
        logger::general_error(format!(
            "--> postlist: [{0} - {1}] ({2})",
            "Search".magenta(),
            filter_s,
            probableurl.blue().underline()
        ));
        return HttpResponse::ExpectationFailed().into();
    }
    logger::req_ok(format!(
        "--> postlist: [{0} - {1}] ({2})",
        "Search".magenta(),
        filter_s,
        probableurl.blue().underline()
    ));
    HttpResponse::Ok()
        .append_header(("Accept-Charset", "UTF-8"))
        .body(cynres)
}

pub(crate) fn f_server(
    filter_t: bool,
    filter_s: &String,
    probableurl: String,
    plugins: Vec<PluginMeta>,
) -> HttpResponse {
    let filtertype = if filter_t { "Category" } else { "Tag" };
    let filters = if filter_t {
        PostListFilter {
            category: Some(filter_s.to_string()),
            tag: None,
            searchline: None,
        }
    } else {
        PostListFilter {
            category: None,
            tag: Some(filter_s.to_string()),
            searchline: None,
        }
    };
    let cynres = combiner::combine_content(
        String::from("root"),
        postlist_table_gen(Postlist {
            filters: Some(filters),
        }),
        generate_menus(String::from("root"), &probableurl),
        plugins.clone(),
    );

    if cynres == *"unknownexeception" {
        logger::general_error(format!(
            "--> postlist: [{0} - {1}] ({2})",
            filtertype,
            filter_s,
            probableurl.blue().underline()
        ));
        return HttpResponse::ExpectationFailed().into();
    }
    logger::req_ok(format!(
        "--> postlist: [{0} - {1}] ({2})",
        filtertype,
        filter_s,
        probableurl.blue().underline()
    ));
    HttpResponse::Ok().body(cynres)
}

pub(crate) fn p_content(pgid: String) -> String {
    let published_jsonc = crate::read_published_jsonc();
    for i in &published_jsonc {
        if i.id == pgid {
            let post: &CynthiaContentMetaData = i;
            if post.kind == *"postlist" {
                return match &post.postlist {
                    Some(list) => {
                        format!(
                            "<h1>{}</h1>{}",
                            post.title,
                            postlist_table_gen(list.clone())
                        )
                    }
                    None => String::from("unknownexeception"),
                };
            };
            let rawcontent = match post.content.location.to_owned().as_str() {
                "external" => {
                    let mut data = Vec::new();
                    let mut c = Easy::new();
                    c.url(&(post.content.data)).unwrap();
                    {
                        let mut transfer = c.transfer();
                        match transfer.write_function(|new_data| {
                            data.extend_from_slice(new_data);
                            Ok(new_data.len())
                        }) {
                            Ok(v) => v,
                            Err(_e) => {
                                logger::general_error(String::from(
                                    "Could not download external content!",
                                ));

                                return "contentlocationerror".to_owned();
                            }
                        };
                        match transfer.perform() {
                            Ok(v) => v,
                            Err(_e) => {
                                logger::general_error(String::from(
                                    "Could not download external content!",
                                ));

                                return "contentlocationerror".to_owned();
                            }
                        };
                    }
                    let resp = match std::str::from_utf8(&data) {
                        Ok(v) => v,
                        Err(_e) => {
                            logger::general_error(String::from(
                                "Could not download external content!",
                            ));

                            return "contentlocationerror".to_owned();
                        }
                    };
                    resp.to_owned()
                }
                "local" => {
                    let contentpath_ = std::path::Path::new("./cynthiaFiles/pages/");
                    let contentpath = &contentpath_.join(post.content.data.to_owned().as_str());
                    fs::read_to_string(contentpath).unwrap_or("contenterror".to_string())
                }
                "inline" => post.content.data.to_owned(),
                &_ => {
                    return "contentlocationerror".to_owned();
                }
            };
            return match post.content.markup_type.to_owned().to_lowercase().as_str() {
                "html" | "webfile" => {
                    format!(
                        "<div id=\"pagecontent\">{}</div>",
                        rawcontent
                            .replace('&', "&amp;")
                            .replace('<', "&lt;")
                            .replace('>', "&gt;")
                            .replace('"', "&quot;")
                            .replace('\'', "&#039;")
                    )
                }
                "text" | "raw" => {
                    format!("<div><pre>{rawcontent}</pre></div>")
                }
                "markdown" | "md" | "" => {
                    format!(
                        "<div id=\"pagecontent\">{}</div>",
                        to_html_with_options(
                            &rawcontent,
                            &Options {
                                compile: CompileOptions {
                                    allow_dangerous_html: true,
                                    allow_dangerous_protocol: true,
                                    ..CompileOptions::default()
                                },
                                ..Options::gfm()
                            },
                        )
                        .unwrap()
                    )
                }
                &_ => "contenttypeerror".to_owned(),
            };
        }
    }
    String::from("404error")
}

fn fourohfour(
    pgid: &String,
    probableurl: String,
    plugins: Vec<PluginMeta>,
    config: CynthiaConf,
) -> HttpResponse {
    if pgid == &config.pages.notfound_page {
        return HttpResponse::NotFound().body("Could not find requested page.");
    }
    p_server(
        &config.clone().pages.notfound_page,
        probableurl,
        plugins,
        config,
    )
}

pub(crate) fn p_server(
    pgid: &String,
    probableurl: String,
    plugins: Vec<PluginMeta>,
    config: CynthiaConf,
) -> HttpResponse {
    if pgid == &config.pages.notfound_page {
        logger::req_notfound(format!(
            "--> {0} ({1})",
            pgid,
            probableurl.blue().underline()
        ));
    }
    let servecache: u64 = config.cache.lifetimes.served;
    match cacheretriever(format!("@web@/p/{}", pgid), servecache) {
        Ok(d) if servecache != 0 => HttpResponse::Ok()
            .append_header(("Accept-Charset", "UTF-8"))
            .body(
                fs::read_to_string(d)
                    .unwrap_or(String::from("Cache error. Please try again later.")),
            ),
        _ => {
            let cynres = combiner::combine_content(
                pgid.to_string(),
                p_content(pgid.to_string()),
                generate_menus(pgid.to_string(), &probableurl),
                plugins.clone(),
            );
            if cynres == *"404error" {
                return fourohfour(pgid, probableurl, plugins, config);
            }
            if cynres == *"unknownexeception" {
                logger::general_error(format!(
                    "--> {0} ({1})",
                    pgid,
                    probableurl.blue().underline()
                ));
                return HttpResponse::ExpectationFailed().into();
            }
            if cynres == *"contentlocationerror" {
                logger::general_error(format!(
                    "--> {0} ({1}) : Post location error",
                    pgid,
                    probableurl.blue().underline()
                ));
                return HttpResponse::ExpectationFailed().into();
            }
            if pgid != &config.pages.notfound_page {
                logger::req_ok(format!(
                    "--> {0} ({1})",
                    pgid,
                    probableurl.blue().underline()
                ));
            }
            if servecache != 0 {
                HttpResponse::Ok()
                    .append_header(("Accept-Charset", "UTF-8"))
                    .body(cacheplacer(format!("@web@/p/{}", pgid), cynres))
            } else {
                HttpResponse::Ok()
                    .append_header(("Accept-Charset", "UTF-8"))
                    .body(cynres)
            }
        }
    }
}

pub(crate) fn generate_menus(pgid: String, probableurl: &String) -> Menulist {
    let mut published_jsonc = crate::read_published_jsonc();
    for post in &mut published_jsonc {
        if post.id == pgid {
            let mode_to_load = post
                .mode
                .get_or_insert_with(|| String::from("default"))
                .to_string();
            let mode = crate::load_mode(mode_to_load).1;
            let mut mlist1 = String::from("");
            match !mode.menulinks.is_empty() {
                true => {
                    for ele in mode.menulinks {
                        let link: String = if ele.href == *probableurl {
                            format!(
                                r#"<a href="{0}" class="menulink active">{1}</a>"#,
                                ele.href, ele.name
                            )
                        } else {
                            format!(
                                r#"<a href="{0}" class="menulink">{1}</a>"#,
                                ele.href, ele.name
                            )
                        };
                        mlist1.push_str(link.as_str());
                    }
                }
                false => (),
            }
            let mut mlist2 = String::from("");
            if !mode.menu2links.is_empty() {
                for ele in mode.menu2links {
                    let link: String = if ele.href == *probableurl {
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
                menu1: mlist1,
                menu2: mlist2,
            };

            return menus;
        }
    }
    let menus: Menulist = Menulist {
        menu1: String::from(""),
        menu2: String::from(""),
    };
    menus
}

pub(crate) fn fetcher(uri: String, config: &CynthiaConf) -> String {
    let cachelifetime: u64 = config.cache.lifetimes.forwarded;
    return match cacheretriever(uri.clone(), cachelifetime) {
        Ok(o) => fs::read_to_string(o).expect("Couldn't find or open a JS file."),
        Err(_) => {
            let mut data = Vec::new();
            let mut c = Easy::new();
            c.url(&uri).unwrap();
            {
                let mut transfer = c.transfer();
                match transfer.write_function(|new_data| {
                    data.extend_from_slice(new_data);
                    Ok(new_data.len())
                }) {
                    Ok(v) => v,
                    Err(_e) => {
                        logger::general_error(String::from("Could not fetch external content!"));

                        return "contentlocationerror".to_owned();
                    }
                };
                match transfer.perform() {
                    Ok(v) => v,
                    Err(_e) => {
                        logger::general_error(String::from("Could not fetch external content!"));

                        return "contentlocationerror".to_owned();
                    }
                };
            }
            let resp = match std::str::from_utf8(&data) {
                Ok(v) => v,
                Err(_e) => {
                    logger::general_error(String::from("Could not fetch external content!"));

                    return "contentlocationerror".to_owned();
                }
            };
            cacheplacer(uri, resp.to_owned())
        }
    };
}
