#![allow(dead_code)]
use std::time::SystemTime;

use crate::config;
use colored::Colorize;
use time::{format_description, OffsetDateTime};

const DATE_FORMAT_STR: &str = "[year]-[month]-[day]-[hour]:[minute]:[second]:[subsecond digits:3]";
const SPACES: usize = 32;
const DIVVER: &str = "\t";
pub(crate) fn general_log(msg: String) {
    if !config::main().logging.enabled {
        return;
    };
    // General log item -- '[log]'
    log_by_act_num(1, msg)
}

pub(crate) fn cache_log(msg: String) {
    if !config::main().logging.cache {
        return;
    };
    // Log item for caching -- '[cache]'
    log_by_act_num(31, msg)
}

pub(crate) fn general_error(msg: String) {
    // Check if these log items are enabled
    if !config::main().logging.error {
        return;
    };
    // General error item -- '[ERROR]'
    log_by_act_num(5, msg)
}
pub(crate) fn fatal_error(msg: String) {
    // Fatal error item -- '[FATAL ERROR]'
    log_by_act_num(5000, msg)
}

pub(crate) fn general_warn(msg: String) {
    // Check if these log items are enabled
    if !config::main().logging.warn {
        return;
    };
    // General warning item -- '[WARN]'
    log_by_act_num(15, msg)
}

pub(crate) fn jsr_error(msg: String) {
    // Check if these log items are enabled
    if !config::main().logging.jsr_errors {
        return;
    };
    // Error in JavaScript runtime
    log_by_act_num(12, msg)
}

pub(crate) fn general_info(msg: String) {
    // Check if these log items are enabled
    if !config::main().logging.info {
        return;
    };
    // General info item -- '[INFO]'
    log_by_act_num(10, msg)
}

pub(crate) fn req_ok(msg: String) {
    // Check if these log items are enabled
    if !config::main().logging.requests {
        return;
    };
    // Request that on Cynthia's part succeeded (and is so responded to) -- '[CYNGET/OK]'
    log_by_act_num(200, msg)
}

pub(crate) fn req_notfound(msg: String) {
    // Check if these log items are enabled
    if !config::main().logging.requests {
        return;
    };
    // Request for an item that does not exist Cynthia published.jsonc
    log_by_act_num(404, msg)
}

pub(crate) fn req_serve_proxied(msg: String) {
    // Check if these log items are enabled
    if !config::main().logging.proxy_requests {
        return;
    };
    // Proxying a request to a plugin
    log_by_act_num(49038, msg)
}
pub(crate) fn req_serve_plugin_asset(msg: String) {
    // Check if these log items are enabled
    if !config::main().logging.plugin_asset_requests {
        return;
    };
    // Serving a plugin asset
    log_by_act_num(293838, msg)
}

pub(crate) fn log_by_act_num(act: i32, msg: String) {
    let log_enabled: bool = config::main().logging.enabled;
    if !log_enabled {
        return;
    };
    /*

    Acts:
    0: Debug log, only act if logging is set to verbose

    88: Silly
     */

    let tabs: String = "\t\t".to_string();
    let dt1: OffsetDateTime = SystemTime::now().into();
    let dt_fmt = format_description::parse(DATE_FORMAT_STR).unwrap();
    let times = dt1.format(&dt_fmt).unwrap();

    match act {
        200 | 2 => {
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
