/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */

#![allow(dead_code)]

use crate::config;
use colored::Colorize;
use std::time::SystemTime;
use time::{format_description, OffsetDateTime};

use std::io::{prelude::*, Seek, SeekFrom};
const DATE_FORMAT_STR: &str = "[year]-[month]-[day]-[hour]:[minute]:[second]:[subsecond digits:3]";
const SPACES: usize = 32;
const DIVVER: &str = "\t";
pub(crate) fn general_log(msg: String) {
    // General log item -- '[log]'
    log_by_act_num(1, msg)
}

pub(crate) fn cache_log(msg: String) {
    // Log item for caching -- '[cache]'
    log_by_act_num(31, msg)
}

pub(crate) fn general_error(msg: String) {
    // General error item -- '[ERROR]'
    log_by_act_num(5, msg)
}
pub(crate) fn fatal_error(msg: String) {
    // Fatal error item -- '[FATAL ERROR]'
    log_by_act_num(5000, msg)
}

pub(crate) fn general_warn(msg: String) {
    // General warning item -- '[WARN]'
    log_by_act_num(15, msg)
}

pub(crate) fn jsr_error(msg: String) {
    // Error in JavaScript runtime
    log_by_act_num(12, msg)
}

pub(crate) fn general_info(msg: String) {
    // General info item -- '[INFO]'
    log_by_act_num(10, msg)
}

pub(crate) fn req_ok(msg: String) {
    // Request that on Cynthia's part succeeded (and is so responded to) -- '[CYNGET/OK]'
    log_by_act_num(200, msg)
}

pub(crate) fn req_notfound(msg: String) {
    // Request for an item that does not exist Cynthia published.jsonc
    log_by_act_num(404, msg)
}

pub(crate) fn req_serve_proxied(msg: String) {
    // Proxying a request to a plugin
    log_by_act_num(49038, msg)
}
pub(crate) fn req_serve_plugin_asset(msg: String) {
    // Serving a plugin asset
    log_by_act_num(293838, msg)
}
use std::fs::OpenOptions;
pub(crate) fn log_by_act_num(act: i32, msg: String) {
    let tabs: String = "\t\t".to_string();
    let dt1: OffsetDateTime = SystemTime::now().into();
    let dt_fmt = format_description::parse(DATE_FORMAT_STR).unwrap();
    let times = dt1.format(&dt_fmt).unwrap();
    if 5000 == act {
        let name = format!("[{} - [FATAL ERROR]", times);
        let spaceleft = if name.chars().count() < SPACES {
            SPACES - name.chars().count()
        } else {
            0
        };
        let title = format!("{}", name.bold().red().on_bright_yellow());
        let preq = format!("{0}{2}{1}", title, " ".repeat(spaceleft), tabs);
        eprintln!("{0}{1}", preq, msg.bright_red());
        return;
    };
    let log = config::main().logging;
    let clog = log.console.clone();
    if !clog.enabled {
        return;
    };
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(log.clone().file.filepath)
        .unwrap();
    file.seek(SeekFrom::End(0)).unwrap();
    match act {
        200 | 2 => {
            if log.file.clone().enabled && log.file.clone().requests {
                file.write_all(
                    format!(
                        "[{}]\t200/SUCCESS\t\t{}\n",
                        times,
                        strip_ansi_escapes::strip_str(msg.clone().as_str())
                    )
                    .as_bytes(),
                )
                .unwrap();
            };
            if !clog.requests {
                return;
            };
            let name = format!("[{} - [CynGET/OK]", times);
            let spaceleft = if name.chars().count() < SPACES {
                SPACES - name.chars().count()
            } else {
                0
            };
            let title = format!("{}", name.bold().yellow());
            let preq = format!("{0}{2}{1}", title, " ".repeat(spaceleft), tabs);
            println!("{0}👍{DIVVER}{1}", preq, msg);
        }
        3 | 404 => {
            if log.file.clone().enabled && log.file.clone().requests {
                file.write_all(
                    format!(
                        "[{}]\t404/NOTFOUND\t\t{}\n",
                        times,
                        strip_ansi_escapes::strip_str(msg.clone().as_str())
                    )
                    .as_bytes(),
                )
                .unwrap();
            };
            if !clog.requests {
                return;
            };
            let name = format!("[{} - [CynGET/404]", times);
            let spaceleft = if name.chars().count() < SPACES {
                SPACES - name.chars().count()
            } else {
                0
            };
            let title = format!("{}", name.bold().yellow());
            let preq = format!("{0}{2}{1}", title, " ".repeat(spaceleft), tabs);
            println!("{0}👎{DIVVER}{1}", preq, msg);
        }
        5 => {
            if log.file.clone().enabled && log.file.clone().error {
                file.write_all(
                    format!(
                        "[{}]\tERROR\t\t\t{}\n",
                        times,
                        strip_ansi_escapes::strip_str(msg.clone().as_str())
                    )
                    .as_bytes(),
                )
                .unwrap();
            };
            // Check if these log items are enabled
            if !clog.error {
                return;
            };
            let name = format!("[{} - [ERROR]", times);
            let spaceleft = if name.chars().count() < SPACES {
                SPACES - name.chars().count()
            } else {
                0
            };
            let title = format!("{}", name.bold().red().on_bright_yellow());
            let preq = format!("{0}{2}{1}", title, " ".repeat(spaceleft), tabs);
            eprintln!("{0}{1}", preq, msg.bright_red());
        }
        15 => {
            if log.file.clone().enabled && log.file.clone().warn {
                file.write_all(
                    format!(
                        "[{}]\tWARNING\t\t{}\n",
                        times,
                        strip_ansi_escapes::strip_str(msg.clone().as_str())
                    )
                    .as_bytes(),
                )
                .unwrap();
            };
            // Check if these log items are enabled
            if !clog.warn {
                return;
            };
            let name = format!("[{} - [WARN]", times);
            let spaceleft = if name.chars().count() < SPACES {
                SPACES - name.chars().count()
            } else {
                0
            };
            let title = format!("{}", name.bold().black().on_bright_yellow());
            let preq = format!("{0}{2}{1}", title, " ".repeat(spaceleft), tabs);
            eprintln!("{0}⚠{DIVVER}{1}", preq, msg.on_bright_magenta().black());
        }
        12 => {
            if log.file.clone().enabled && log.file.clone().jsr_errors {
                file.write_all(
                    format!(
                        "[{}]\tERROR-JS\t\t\t{}\n",
                        times,
                        strip_ansi_escapes::strip_str(msg.clone().as_str())
                    )
                    .as_bytes(),
                )
                .unwrap();
            };
            // Check if these log items are enabled
            if !clog.jsr_errors {
                return;
            };
            let name = "[JS/ERROR]";
            let spaceleft = if name.chars().count() < SPACES {
                SPACES - name.chars().count()
            } else {
                0
            };
            let title = format!("{}", name.bold().black().on_bright_yellow());
            let preq = format!("{0}{2}{1}", title, " ".repeat(spaceleft), tabs);
            eprintln!("{0}{1}", preq, msg.bright_red().on_bright_yellow());
        }
        49038 => {
            if log.file.clone().enabled && log.file.clone().proxy_requests {
                file.write_all(
                    format!(
                        "[{}]\tPROXY\t\t\t{}\n",
                        times,
                        strip_ansi_escapes::strip_str(msg.clone().as_str())
                    )
                    .as_bytes(),
                )
                .unwrap();
            };
            // Check if these log items are enabled
            if !clog.proxy_requests {
                return;
            };
            let name = format!("[{} - [PROXY]", times);
            let spaceleft = if name.chars().count() < SPACES {
                SPACES - name.chars().count()
            } else {
                0
            };
            let title = format!("{}", name.bold().bright_magenta());
            let preq = format!("{0}{2}{1}", title, " ".repeat(spaceleft), tabs);
            println!("{0}❕{DIVVER}{1}", preq, msg.bright_green());
        }
        293838 => {
            if log.file.clone().enabled && log.file.clone().plugin_asset_requests {
                file.write_all(
                    format!(
                        "[{}]\t200/PLUGIN\t\t{}\n",
                        times,
                        strip_ansi_escapes::strip_str(msg.clone().as_str())
                    )
                    .as_bytes(),
                )
                .unwrap();
            };

            // Check if these log items are enabled
            if !clog.plugin_asset_requests {
                return;
            };
            let name = format!("[{} - [PLUGIN ASSET]", times);
            let spaceleft = if name.chars().count() < SPACES {
                SPACES - name.chars().count()
            } else {
                0
            };
            let title = format!("{}", name.bold().bright_magenta());
            let preq = format!("{0}{2}{1}", title, " ".repeat(spaceleft), tabs);
            println!("{0}❕{DIVVER}{1}", preq, msg.bright_green());
        }
        10 => {
            if log.file.clone().enabled && log.file.clone().info {
                file.write_all(
                    format!(
                        "[{}]\tNOTE\t\t\t{}\n",
                        times,
                        strip_ansi_escapes::strip_str(msg.clone().as_str())
                    )
                    .as_bytes(),
                )
                .unwrap();
            };
            // Check if these log items are enabled
            if !clog.info {
                return;
            };
            let name = format!("[{} - [NOTE]", times);
            let spaceleft = if name.chars().count() < SPACES {
                SPACES - name.chars().count()
            } else {
                0
            };
            let title = format!("{}", name.bold().bright_magenta());
            let preq = format!("{0}{2}{1}", title, " ".repeat(spaceleft), tabs);
            println!("{0}❕{DIVVER}{1}", preq, msg.bright_green());
        }
        31 => {
            if log.file.clone().enabled && log.file.clone().cache {
                file.write_all(
                    format!(
                        "[{}]\tCACHING\t\t{}\n",
                        times,
                        strip_ansi_escapes::strip_str(msg.clone().as_str())
                    )
                    .as_bytes(),
                )
                .unwrap();
            };
            if !clog.cache {
                return;
            };
            let name = format!("[{} - [CACHE]", times);
            let spaceleft = if name.chars().count() < SPACES {
                SPACES - name.chars().count()
            } else {
                0
            };
            let title = format!("{}", name.white());
            let preq = format!("{0}{2}{1}", title, " ".repeat(spaceleft), tabs);
            println!("{0}♻️{DIVVER}{1}", preq, msg.bright_white().italic());
        }
        _ => {
            if log.file.clone().enabled {
                file.write_all(
                    format!(
                        "[{}]\tLOG\t\t\t\t{}\n",
                        times,
                        strip_ansi_escapes::strip_str(msg.clone().as_str())
                    )
                    .as_bytes(),
                )
                .unwrap();
            };
            let name = format!("[{} - [LOG]", times).blue();
            let spaceleft = if name.chars().count() < SPACES {
                SPACES - name.chars().count()
            } else {
                0
            };
            let title = format!("{}", name.bold().blue());
            let preq = format!("{0}{2}{1}", title, " ".repeat(spaceleft), tabs);
            println!("{0}{1}", preq, msg);
        }
    }
}
