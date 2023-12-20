use crate::{logger::logger, structs::*};

use markdown::{to_html_with_options, CompileOptions, Options};

pub(crate) fn postlist_table_gen(postlistobject: Postlist) -> String {
    let published_jsonc = crate::read_published_jsonc();
    let mut lastdatestamp: i64 = 1;
    let mut fullpostlist: Vec<&CynthiaContentMetaData> = Vec::new();
    for i in &published_jsonc {
        if i.kind == *"post" {
            let timestamp: i64 = match &i.dates {
                Some(d) => d.published,
                None => {
                    logger(5, format!("Post with id '{}' has invalid date data.", i.id));
                    1
                }
            };
            if lastdatestamp == 1 {
                fullpostlist.push(i);
            } else {
                if timestamp >= lastdatestamp {
                    fullpostlist.insert(0, i);
                } else {
                    fullpostlist.push(i);
                }
                lastdatestamp = timestamp;
            }
        }
    }
    let vectorofposts = fullpostlist.clone();
    fullpostlist = match postlistobject.filters {
        Some(f) => match f.category {
            Some(c) => {
                let mut categorisedpostlist: Vec<&CynthiaContentMetaData> = Vec::new();
                for i in vectorofposts {
                    let cat: String = match &i.category {
                        Some(d) => d.to_string(),
                        None => {
                            logger(5, format!("Post with id '{}' has no category data.", i.id));
                            String::from("")
                        }
                    };
                    if cat == c {
                        categorisedpostlist.push(i)
                    }
                }
                categorisedpostlist
            }
            None => match f.tag {
                Some(t) => {
                    let mut taggedpostlist: Vec<&CynthiaContentMetaData> = Vec::new();
                    for i in vectorofposts {
                        let tags = &i.tags;
                        if tags.contains(&t) {
                            taggedpostlist.push(i)
                        }
                    }
                    taggedpostlist
                }
                None => match f.searchline {
                    Some(s) => {
                        let mut taggedpostlist: Vec<&CynthiaContentMetaData> = Vec::new();
                        for i in vectorofposts {
                            let id = &i.id;
                            let desc = &i.short.clone().unwrap();
                            let tags = &i.tags;

                            if super::p_content(id.to_string()).contains(&s)
                                | i.title.contains(&s)
                                | desc.contains(&s)
                                | tags.contains(&s)
                            {
                                taggedpostlist.push(i);
                            }
                        }
                        taggedpostlist
                    }
                    None => fullpostlist,
                },
            },
        },
        None => vectorofposts,
    };
    // Now, we generate the HTML
    let mut table_html = String::from(
        r#"<table class="post-listpreview"><tr id="post-listpreview-h"><th id="h-post-date">Posted on</th><th id="h-post-title">Title</th><th id="h-post-category">Category</th></tr>"#,
    );
    for post in fullpostlist.clone() {
        let category: &str = match &post.category {
            Some(c) => c.as_str(),
            None => "",
        };
        let description = match &post.short {
            Some(s) => s.as_str(),
            None => "",
        };
        let timestamp: i64 = match &post.dates {
            Some(d) => d.published,
            None => {
                logger(
                    5,
                    format!("Post with id '{}' has invalid date data.", post.id),
                );
                1
            }
        };
        let addition = format!(
            r#"<tr><td class="post-date"><span class="unparsedtimestamp post-date">{0}</span></td><td><a href="/p/{1}"><span class="post-title">{2}</span></a></td><td class="post-category"><a href="/c/{3}">{3}</a></td></tr><tr><td></td><td class="post-desc"><p>{4}</p></td></tr>"#,
            &timestamp,
            &post.id,
            to_html_with_options(
                &post.title,
                &Options {
                    compile: CompileOptions {
                        allow_dangerous_html: false,
                        ..CompileOptions::default()
                    },
                    ..Options::default()
                },
            )
            .unwrap(),
            category,
            to_html_with_options(
                &description,
                &Options {
                    compile: CompileOptions {
                        allow_dangerous_html: false,
                        ..CompileOptions::default()
                    },
                    ..Options::default()
                },
            )
            .unwrap()
        );
        table_html.push_str(addition.as_str());
    }
    table_html.push_str("</table>");
    if fullpostlist.len() == 0 {
        table_html = String::from("<p>No results.</p>");
    }
    table_html
}
