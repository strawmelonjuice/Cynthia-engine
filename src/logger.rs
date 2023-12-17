use std::time::SystemTime;

use colored::Colorize;
use time::{format_description, OffsetDateTime};

const DATE_FORMAT_STR: &'static str =
    "[year]-[month]-[day]-[hour]:[minute]:[second]:[subsecond digits:3]";

pub(crate) fn logger(act: i32, msg: String) {
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
    let dt1: OffsetDateTime = SystemTime::now().into();
    let dt_fmt = format_description::parse(DATE_FORMAT_STR).unwrap();
    let times = dt1.format(&dt_fmt).unwrap();
    if act == 1 {
        let name = format!("[{} - Log]", times).blue();
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
