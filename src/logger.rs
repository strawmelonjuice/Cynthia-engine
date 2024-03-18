#![allow(dead_code)]
use std::time::SystemTime;

use crate::config;
use colored::Colorize;
use time::{format_description, OffsetDateTime};

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

pub(crate) fn log_by_act_num(act: i32, msg: String) {
    let log = config::main().logging;
    if !log.enabled {
        return;
    };
    let tabs: String = "\t\t".to_string();
    let dt1: OffsetDateTime = SystemTime::now().into();
    let dt_fmt = format_description::parse(DATE_FORMAT_STR).unwrap();
    let times = dt1.format(&dt_fmt).unwrap();

    match act {
        200 | 2 => {
            if !log.requests { return; };
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
            if !log.requests { return; };
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
            // Check if these log items are enabled
            if !log.error {
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
        5000 => {
            let name = format!("[{} - [FATAL ERROR]", times);
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
            // Check if these log items are enabled
            if !log.warn {
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
            // Check if these log items are enabled
            if !log.jsr_errors {
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
            // Check if these log items are enabled
            if !log.proxy_requests {
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
            // Check if these log items are enabled
            if !log.plugin_asset_requests {
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
            // Check if these log items are enabled
            if !log.info {
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
            if !log.cache {
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
        88 => {
            let name = String::from("[SILLY]");
            let spaceleft = if name.chars().count() < SPACES {
                SPACES - name.chars().count()
            } else {
                0
            };
            let title = format!("{}", name.bold().bright_magenta());
            let preq = format!("{0}{2}{1}", title, " ".repeat(spaceleft), tabs);
            println!("{0}❕{DIVVER}{1}", preq, msg.bright_green());
        }
        _ => {
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
