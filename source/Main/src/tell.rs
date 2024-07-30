/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */
// This module is a adoptation of the Lumina logging module, also written by me.
//! ## Actions for gentle logging ("telling")
//! Logging doesn't need this, but for prettyness these are added as implementations on ServerVars.

use std::time::SystemTime;

use colored::Colorize;
use log::info;
use time::{format_description, OffsetDateTime};

use crate::config::Logging;
use crate::ServerContext;

const DATE_FORMAT_STR: &str = "[hour]:[minute]:[second]";

#[doc = r"A function that either prints as an [info] log, or prints as [log], depending on configuration. This because loglevel 3 is a bit too verbose, while loglevel 2 is too quiet."]
impl ServerContext {
    pub(crate) fn tell(&self, rmsg: impl AsRef<str>) {
        let msg = rmsg.as_ref();
        match &self.config.logs.clone() {
            None => {
                println!("{}", self.format_tell(msg));
                info!("{}", msg);
            }
            Some(l) => {
                l.clone().to_owned().tell(rmsg);
            }
        }
    }

    pub(crate) fn format_tell(&self, rmsg: impl AsRef<str>) -> String {
        let msg = rmsg.as_ref();
        let dt1: OffsetDateTime = SystemTime::now().into();
        let dt_fmt = format_description::parse(DATE_FORMAT_STR).unwrap();
        let times = dt1.format(&dt_fmt).unwrap();
        format!("{} {} {}", times, "[LOG] ".magenta(), msg)
    }
}
impl Logging {
    fn tell(self, rmsg: impl AsRef<str>) {
        let msg = rmsg.as_ref();
        let a = self;
        match a.term_loglevel {
            None => {
                let dt1: OffsetDateTime = SystemTime::now().into();
                let dt_fmt = format_description::parse(DATE_FORMAT_STR).unwrap();
                let times = dt1.format(&dt_fmt).unwrap();
                println!("{} {} {}", times, "[LOG] ".magenta(), msg);
                info!("{}", msg);
            }
            Some(s) => {
                // If the log level is set to erroronly or info-too, just return it as info. The only other case is really just 2, but I am funny.
                if s >= 3 || s <= 1 {
                    info!("{}", msg);
                } else {
                    {
                        let dt1: OffsetDateTime = SystemTime::now().into();
                        let dt_fmt = format_description::parse(DATE_FORMAT_STR).unwrap();
                        let times = dt1.format(&dt_fmt).unwrap();
                        println!("{} {} {}", times, "[LOG] ".magenta(), msg);
                        info!("{}", msg);
                    }
                }
            }
        }
    }
}
pub(crate) fn horizline() -> String {
    ("\u{2500}".repeat(termsize::get().unwrap().cols as usize)).to_string()
}
