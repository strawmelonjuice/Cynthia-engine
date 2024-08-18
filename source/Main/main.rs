/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */

use actix_web::web::Data;
use actix_web::{App, HttpServer};
use futures::join;
use log::info;
use log::LevelFilter;
use log::{debug, error};
use requestresponse::{assets_with_cache, serve};
use simplelog::{ColorChoice, CombinedLogger, TermLogger, TerminalMode, WriteLogger};
use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, process};
use tell::{CynthiaColors, CynthiaStyles};
use tokio::sync::{Mutex, MutexGuard};
use tokio::{spawn, task};

use crate::config::{CynthiaConf, CynthiaConfig, SceneCollectionTrait};
use crate::externalpluginservers::EPSRequest;
use crate::files::CynthiaCache;
use crate::tell::horizline;

mod config;
mod externalpluginservers;
mod files;
mod jsrun;
mod publications;
mod renders;
mod requestresponse;

struct LogSets {
    pub file_loglevel: LevelFilter,
    pub term_loglevel: LevelFilter,
    pub logfile: PathBuf,
}

#[derive(Debug)]
/// Server context, containing the configuration and cache. Also implements a `tell` method for easy logging.
struct ServerContext {
    config: CynthiaConf,
    cache: CynthiaCache,
    request_count: u64,
    start_time: u128,

    #[cfg(feature = "js_runtime")]
    external_plugin_server: EPSCommunicationData,
}
trait LockCallback {
    async fn lock_callback<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut MutexGuard<ServerContext>) -> T;
}
impl LockCallback for Mutex<ServerContext> {
    async fn lock_callback<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut MutexGuard<ServerContext>) -> T,
    {
        let mut s = self.lock().await;
        f(&mut s)
    }
}
impl LockCallback for Arc<Mutex<ServerContext>> {
    async fn lock_callback<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut MutexGuard<ServerContext>) -> T,
    {
        let mut s = self.lock().await;
        f(&mut s)
    }
}
impl LockCallback for Data<Arc<Mutex<ServerContext>>> {
    async fn lock_callback<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut MutexGuard<ServerContext>) -> T,
    {
        let mut s = self.lock().await;
        f(&mut s)
    }
}

type EPSCommunicationsID = u32;

#[cfg(feature = "js_runtime")]
use crate::externalpluginservers::EPSCommunicationData;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    println!(
        " \u{21E2} cynthiaweb {}",
        args.get(1)
            .unwrap_or(&String::from("start"))
            .to_ascii_lowercase()
            .as_str()
    );
    println!("{}", horizline().color_purple());

    println!(
        "{} - version {}\n by {}{}{} {}!",
        "CynthiaWeb".style_bold().color_lilac(),
        env!("CARGO_PKG_VERSION").to_string().color_green(),
        "Straw".color_bright_red(),
        "melon".color_green(),
        "juice".color_bright_yellow(),
        "Mar".color_pink()
    );
    println!("{}", horizline().color_purple());
    match args
        .get(1)
        .unwrap_or(&String::from(""))
        .to_ascii_lowercase()
        .as_str()
    {
        "help" => {
            println!(
                "{}",
                "Cynthia - a simple site generator/server with a focus on performance and ease of use. Targeted at smaller sites and personal projects.".color_pink()
            );
            println!(
                "{}",
                "Usage: cynthiaweb [command]\n\nCommands:".color_lime()
            );
            println!(
                "\t{}{}",
                "help".style_bold().color_yellow(),
                ": Displays this message.".color_lime()
            );
            println!(
                "\t{}{}",
                "start".style_bold().color_yellow(),
                ": Starts the server.".color_lime()
            );
            println!(
                "\t{}{}\n\t\t{}",
                "convert [format] <-k>".style_bold().color_yellow(),
                ": Converts the configuration to the specified format.".color_lime(),
                "Available formats: `dhall`, `toml`, `jsonc`.".style_clear()
            );
            println!("\t{} {{{}}} <{}> ({})
            Available subcommands:
                - Add:
                    Installs a new plugin as registered in the Cynthia Plugin Index. (Does not save it to the manifest file.)

                    Options:
                        - <{}>
                            Specifies the name of the plugin to install. Is required.
                        - {{{}}}
                            (Optional) Specifies the plugin version (this will not work if a plugin has a single-version channel)
                            If not specified, latest available will be used.
                - Install:
                    Installs plugins from {} using the Cynthia Plugin Index. Useful after cloning a config.",
                     "PM".style_bold().color_yellow(), "subcommand".color_lime(), "plugin name".color_bright_yellow(), "plugin version".color_lilac(),
                     "plugin name".color_bright_yellow(),
                     "plugin version".color_lilac(),

                     "cynthiapluginmanifest.json".color_lime(),);
            process::exit(0);
        }
        "start" => start().await,
        "convert" => {
            if args.len() < 3 {
                eprintln!(
                    "{} No format specified! Please run `cynthiaweb help` for a list of commands.",
                    "error:".color_red()
                );
                process::exit(1);
            }
            // config::actions::save_config(args.get(2).unwrap_or(&String::from("")), CynthiaConf::default());
            config::actions::save_config(
                args.get(2).unwrap_or(&String::from("")),
                config::actions::load_config().hard_clone(),
            );
        }
        "" => {
            eprintln!(
                "{} No command specified! Please run `cynthiaweb help` for a list of commands.\n\nRunning: `cynthiaweb start` from here on.",
                "error:".color_red()
            );
            start().await;
            println!("And next time, try to use the `start` command directly!");
        }
        _ => {
            eprintln!(
                "{} Could not interpret command `{}`! Please run `cynthiaweb help` for a list of commands.",
                "error:".color_red(),
                args.get(1).unwrap_or(&String::from("")).to_ascii_lowercase()
            );
            process::exit(1);
        }
    }
}

async fn start() {
    let cd = std::env::current_dir().unwrap();
    let config = config::actions::load_config();
    // Validate the configuration
    if config.port == 0 {
        eprintln!(
            "{} Could not set port to 0! Please set it to a valid port.",
            "error:".color_red()
        );
        process::exit(1);
    }
    if config.logs.is_none() {
        eprintln!("No log configuration found, using defaults");
    }
    // Validate scenes
    if !config.scenes.validate() {
        eprintln!(
            "{} Could not validate scenes! Please check your configuration.",
            "error:".color_red()
        );
        process::exit(1);
    }
    debug!("Configuration: {:?}", config);
    let logsets: LogSets = {
        fn matchlogmode(o: u16) -> LevelFilter {
            match o {
                0 => LevelFilter::Off,
                1 => LevelFilter::Error,
                2 => LevelFilter::Warn,
                3 => LevelFilter::Info,
                4 => LevelFilter::Debug,
                5 => LevelFilter::Trace,
                _ => {
                    eprintln!(
                        "{} Could not set loglevel `{}`! Ranges are 0-5 (quiet to verbose)",
                        "error:".color_red(),
                        o
                    );
                    process::exit(1);
                }
            }
        }
        match config.clone().logs {
            None => LogSets {
                file_loglevel: LevelFilter::Info,
                term_loglevel: LevelFilter::Warn,
                logfile: cd.join("./cynthia.log"),
            },
            Some(d) => LogSets {
                file_loglevel: match d.file_loglevel {
                    Some(l) => matchlogmode(l),
                    None => LevelFilter::Info,
                },
                term_loglevel: match d.term_loglevel {
                    Some(l) => matchlogmode(l),
                    None => LevelFilter::Warn,
                },
                logfile: match d.logfile {
                    Some(s) => cd.join(s.as_str()),
                    None => cd.join("./cynthia.log"),
                },
            },
        }
    };

    CombinedLogger::init(vec![
        TermLogger::new(
            logsets.term_loglevel,
            simplelog::Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            logsets.file_loglevel,
            simplelog::Config::default(),
            File::create(&logsets.logfile).unwrap(),
        ),
    ])
    .unwrap();
    use crate::config::CynthiaConfig;

    let (_to_eps_s, to_eps_r) = tokio::sync::mpsc::channel::<EPSRequest>(100);
    // Initialise context
    let server_context: ServerContext = ServerContext {
        config: config.hard_clone(),
        cache: vec![],
        request_count: 0,
        start_time: 0,

        #[cfg(feature = "js_runtime")]
        external_plugin_server: EPSCommunicationData::new(_to_eps_s),
    };
    let _ = &server_context.tell(format!(
        "Logging to {}",
        logsets
            .logfile
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .replace("\\\\?\\", "")
    ));
    let _ = fs::remove_dir_all("./.cynthiaTemp");
    match fs::create_dir_all("./.cynthiaTemp") {
        Ok(_) => {}
        Err(e) => {
            error!(
                "Could not create the Cynthia temp folder! Error: {}",
                e.to_string().color_bright_red()
            );
            process::exit(1);
        }
    }
    let server_context_arc_mutex: Arc<Mutex<ServerContext>> = Arc::new(Mutex::new(server_context));
    let server_context_data: Data<Arc<Mutex<ServerContext>>> =
        Data::new(server_context_arc_mutex.clone());
    let main_server = match HttpServer::new(move || {
        App::new()
            .service(assets_with_cache)
            .service(serve)
            .app_data(server_context_data.clone())
    })
    .bind(("localhost", config.port))
    {
        Ok(o) => {
            println!("Running on http://localhost:{}", config.port);
            o
        }
        Err(s) => {
            error!(
                "Could not bind to port {}, error message: {}",
                config.port, s
            );
            process::exit(1);
        }
    }
    .run();
    let _ = join!(
        main_server,
        close(server_context_arc_mutex.clone()),
        cache_manager(server_context_arc_mutex.clone()),
        start_timer(server_context_arc_mutex.clone()),
        externalpluginservers::main(server_context_arc_mutex.clone(), to_eps_r)
    );
}
async fn start_timer(server_context_mutex: Arc<Mutex<ServerContext>>) {
    let mut server_context: MutexGuard<ServerContext> = server_context_mutex.lock().await;
    server_context.start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
}
async fn close(server_context_mutex: Arc<Mutex<ServerContext>>) {
    let _ = tokio::signal::ctrl_c().await;
    let server_context: MutexGuard<ServerContext> = server_context_mutex.lock().await;
    // Basically now that we block the main thread, we have all the time lol
    // let _ = server_context
    //     .external_plugin_server
    //     .request_channel_sender
    //     .send(EPSRequest {
    //         id: 0,
    //         command: "close".to_string(),
    //     });
    let total_run_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        - server_context.start_time;
    let total_run_time_locale = chrono::Duration::milliseconds(total_run_time as i64);
    let run_time_hours = total_run_time_locale.num_hours();
    let run_time_minutes =
        total_run_time_locale.num_minutes() - (total_run_time_locale.num_hours() * 60);
    let run_time_seconds =
        total_run_time_locale.num_seconds() - (total_run_time_locale.num_minutes() * 60);
    let run_time_string = format!(
        "{}h {}m {}s",
        run_time_hours, run_time_minutes, run_time_seconds
    );
    let s = if server_context.request_count == 1 {
        ""
    } else {
        "s"
    };
    server_context.tell(format!(
        "Closing:\n\n\n\nBye! I served {} request{s} in this run of {}!\n",
        server_context.request_count, run_time_string
    ));
    println!("{}", horizline().color_lilac());
    process::exit(0);
}
use std::time::Duration;
use tokio::time;
async fn cache_manager(server_context_mutex: Arc<Mutex<ServerContext>>) {
    let server_context_mutex_clone = server_context_mutex.clone();
    let forever = task::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(2));
        loop {
            debug!("Cache manager tick");
            interval.tick().await;
            {
                let mut server_context: MutexGuard<ServerContext> =
                    server_context_mutex_clone.lock().await;
                // trace!("Cache: {:?}", server_context.cache);
                if server_context.estimate_cache_size() > server_context.config.cache.max_cache_size
                {
                    info!(
                        "Maximum cache size of {} exceeded, clearing cache now.",
                        server_context.config.cache.max_cache_size
                    );
                    server_context.clear_cache();
                } else {
                    server_context.evaluate_cache();
                }
            }
        }
    });
    spawn(forever);
}
pub(crate) mod tell {
    // This module is a adoptation of the Lumina logging module, also written by me.
    //! ## Actions for gentle logging ("telling")
    //! Logging doesn't need this, but for prettyness these are added as implementations on ServerVars.

    use std::time::SystemTime;

    use log::info;
    use time::{format_description, OffsetDateTime};

    use crate::config::{CynthiaConfClone, Logging};
    use crate::ServerContext;

    const DATE_FORMAT_STR: &str = "[hour]:[minute]:[second]";

    #[doc = r"A function that either prints as an [info] log, or prints as [log], depending on configuration. This because crate::tell::CynthiaColors; use loglevel 3 is a bit too verbose, while loglevel 2 is too quiet."]
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
            format!("{} {} {}", times, "[LOG] ".color_magenta(), msg)
        }
    }
    /// For when context is unavailable to be locked, confclone should be able to tell too.
    impl CynthiaConfClone {
        pub(crate) fn tell(&self, rmsg: impl AsRef<str>) {
            let msg = rmsg.as_ref();
            match &self.logs.clone() {
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
            format!("{} {} {}", times, "[LOG] ".color_magenta(), msg)
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
                    println!("{} {} {}", times, "[LOG] ".color_magenta(), msg);
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
                            println!("{} {} {}", times, "[LOG] ".color_magenta(), msg);
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
    type CynthiaStyledString = String;

    #[allow(dead_code)]
    pub(crate) trait CynthiaStyles {
        fn style_bold(self) -> CynthiaStyledString;
        fn style_italic(self) -> CynthiaStyledString;
        fn style_underline(self) -> CynthiaStyledString;
        fn style_strikethrough(self) -> CynthiaStyledString;
        fn style_dim(self) -> CynthiaStyledString;
        fn style_blink(self) -> CynthiaStyledString;
        fn style_reverse(self) -> CynthiaStyledString;
        fn style_clear(self) -> CynthiaStyledString;
    }
    impl CynthiaStyles for &str {
        #[inline]
        fn style_bold(self) -> CynthiaStyledString {
            format!("\u{001b}[1m{}\u{001b}[0m", self)
        }
        #[inline]
        fn style_italic(self) -> CynthiaStyledString {
            format!("\u{001b}[3m{}\u{001b}[0m", self)
        }
        #[inline]
        fn style_underline(self) -> CynthiaStyledString {
            format!("\u{001b}[4m{}\u{001b}[0m", self)
        }
        #[inline]
        fn style_strikethrough(self) -> CynthiaStyledString {
            format!("\u{001b}[9m{}\u{001b}[0m", self)
        }
        #[inline]
        fn style_dim(self) -> CynthiaStyledString {
            format!("\u{001b}[2m{}\u{001b}[0m", self)
        }
        #[inline]
        fn style_blink(self) -> CynthiaStyledString {
            format!("\u{001b}[5m{}\u{001b}[0m", self)
        }
        #[inline]
        fn style_reverse(self) -> CynthiaStyledString {
            format!("\u{001b}[7m{}\u{001b}[0m", self)
        }
        #[inline]
        fn style_clear(self) -> CynthiaStyledString {
            format!("\u{001b}[0m{}\u{001b}[0m", self)
        }
    }
    impl CynthiaStyles for String {
        #[inline]
        fn style_bold(self) -> CynthiaStyledString {
            format!("\u{001b}[1m{}\u{001b}[0m", self)
        }
        #[inline]
        fn style_italic(self) -> CynthiaStyledString {
            format!("\u{001b}[3m{}\u{001b}[0m", self)
        }
        #[inline]
        fn style_underline(self) -> CynthiaStyledString {
            format!("\u{001b}[4m{}\u{001b}[0m", self)
        }
        #[inline]
        fn style_strikethrough(self) -> CynthiaStyledString {
            format!("\u{001b}[9m{}\u{001b}[0m", self)
        }
        #[inline]
        fn style_dim(self) -> CynthiaStyledString {
            format!("\u{001b}[2m{}\u{001b}[0m", self)
        }
        #[inline]
        fn style_blink(self) -> CynthiaStyledString {
            format!("\u{001b}[5m{}\u{001b}[0m", self)
        }
        #[inline]
        fn style_reverse(self) -> CynthiaStyledString {
            format!("\u{001b}[7m{}\u{001b}[0m", self)
        }
        #[inline]
        fn style_clear(self) -> CynthiaStyledString {
            format!("\u{001b}[0m{}\u{001b}[0m", self)
        }
    }
    type CynthiaColoredString = String;
    #[allow(dead_code)]
    pub(crate) trait CynthiaColors {
        fn by_rgb(self, r: u32, g: u32, b: u32) -> CynthiaColoredString;
        fn color_green(self) -> CynthiaColoredString;
        fn color_ok_green(self) -> CynthiaColoredString;
        fn color_lime(self) -> CynthiaColoredString;
        fn color_red(self) -> CynthiaColoredString;
        fn color_error_red(self) -> CynthiaColoredString;
        fn color_bright_red(self) -> CynthiaColoredString;
        fn color_black(self) -> CynthiaColoredString;
        fn color_bright_black(self) -> CynthiaColoredString;
        fn color_white(self) -> CynthiaColoredString;
        fn color_bright_white(self) -> CynthiaColoredString;
        fn color_yellow(self) -> CynthiaColoredString;
        fn color_bright_yellow(self) -> CynthiaColoredString;
        fn color_cyan(self) -> CynthiaColoredString;
        fn color_bright_cyan(self) -> CynthiaColoredString;
        fn color_magenta(self) -> CynthiaColoredString;
        fn color_pink(self) -> CynthiaColoredString;
        fn color_blue(self) -> CynthiaColoredString;
        fn color_lightblue(self) -> CynthiaColoredString;
        fn color_orange(self) -> CynthiaColoredString;
        fn color_bright_orange(self) -> CynthiaColoredString;
        fn color_purple(self) -> CynthiaColoredString;
        fn color_lilac(self) -> CynthiaColoredString;
    }
    impl CynthiaColors for &str {
        #[inline]
        fn by_rgb(self, r: u32, g: u32, b: u32) -> CynthiaColoredString {
            format!("\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, self)
        }
        #[inline]
        fn color_green(self) -> CynthiaColoredString {
            self.by_rgb(0, 255, 0)
        }
        #[inline]
        fn color_ok_green(self) -> CynthiaColoredString {
            self.by_rgb(116, 204, 140)
        }
        #[inline]
        fn color_lime(self) -> CynthiaColoredString {
            self.by_rgb(66, 245, 78)
        }
        #[inline]
        fn color_red(self) -> CynthiaColoredString {
            self.by_rgb(255, 0, 0)
        }
        #[inline]
        fn color_error_red(self) -> CynthiaColoredString {
            self.by_rgb(184, 28, 74)
        }
        #[inline]
        fn color_bright_red(self) -> CynthiaColoredString {
            self.by_rgb(237, 68, 62)
        }
        #[inline]
        fn color_black(self) -> CynthiaColoredString {
            self.by_rgb(41, 40, 40)
        }
        #[inline]
        fn color_bright_black(self) -> CynthiaColoredString {
            self.by_rgb(0, 0, 0)
        }
        #[inline]
        fn color_white(self) -> CynthiaColoredString {
            self.by_rgb(240, 240, 240)
        }
        #[inline]
        fn color_bright_white(self) -> CynthiaColoredString {
            self.by_rgb(255, 255, 255)
        }
        #[inline]
        fn color_yellow(self) -> CynthiaColoredString {
            self.by_rgb(243, 201, 35)
        }
        #[inline]
        fn color_bright_yellow(self) -> CynthiaColoredString {
            self.by_rgb(255, 234, 150)
        }
        #[inline]
        fn color_cyan(self) -> CynthiaColoredString {
            self.by_rgb(16, 227, 227)
        }
        #[inline]
        fn color_bright_cyan(self) -> CynthiaColoredString {
            self.by_rgb(0, 255, 255)
        }
        #[inline]
        fn color_magenta(self) -> CynthiaColoredString {
            self.by_rgb(255, 0, 255)
        }
        #[inline]
        fn color_pink(self) -> CynthiaColoredString {
            self.by_rgb(243, 154, 245)
        }
        #[inline]
        fn color_blue(self) -> CynthiaColoredString {
            self.by_rgb(0, 0, 255)
        }
        #[inline]
        fn color_lightblue(self) -> CynthiaColoredString {
            self.by_rgb(145, 220, 255)
        }
        #[inline]
        fn color_orange(self) -> CynthiaColoredString {
            self.by_rgb(255, 165, 0)
        }
        #[inline]
        fn color_bright_orange(self) -> CynthiaColoredString {
            self.by_rgb(255, 157, 0)
        }
        #[inline]
        fn color_purple(self) -> CynthiaColoredString {
            self.by_rgb(97, 18, 181)
        }
        #[inline]
        fn color_lilac(self) -> CynthiaColoredString {
            self.by_rgb(200, 162, 200)
        }
    }
    impl CynthiaColors for String {
        #[inline]
        fn by_rgb(self, r: u32, g: u32, b: u32) -> CynthiaColoredString {
            self.as_str().by_rgb(r, g, b)
        }
        #[inline]
        fn color_green(self) -> CynthiaColoredString {
            self.as_str().color_green()
        }
        #[inline]
        fn color_ok_green(self) -> CynthiaColoredString {
            self.as_str().color_ok_green()
        }
        #[inline]
        fn color_lime(self) -> CynthiaColoredString {
            self.as_str().color_lime()
        }
        #[inline]
        fn color_red(self) -> CynthiaColoredString {
            self.as_str().color_red()
        }
        #[inline]
        fn color_error_red(self) -> CynthiaColoredString {
            self.as_str().color_error_red()
        }
        #[inline]
        fn color_bright_red(self) -> CynthiaColoredString {
            self.as_str().color_bright_red()
        }
        #[inline]
        fn color_black(self) -> CynthiaColoredString {
            self.as_str().color_black()
        }
        #[inline]
        fn color_bright_black(self) -> CynthiaColoredString {
            self.as_str().color_bright_black()
        }
        #[inline]
        fn color_white(self) -> CynthiaColoredString {
            self.as_str().color_white()
        }
        #[inline]
        fn color_bright_white(self) -> CynthiaColoredString {
            self.as_str().color_bright_white()
        }
        #[inline]
        fn color_yellow(self) -> CynthiaColoredString {
            self.as_str().color_yellow()
        }
        #[inline]
        fn color_bright_yellow(self) -> CynthiaColoredString {
            self.as_str().color_bright_yellow()
        }
        #[inline]
        fn color_cyan(self) -> CynthiaColoredString {
            self.as_str().color_cyan()
        }
        #[inline]
        fn color_bright_cyan(self) -> CynthiaColoredString {
            self.as_str().color_bright_cyan()
        }
        #[inline]
        fn color_magenta(self) -> CynthiaColoredString {
            self.as_str().color_magenta()
        }
        #[inline]
        fn color_pink(self) -> CynthiaColoredString {
            self.as_str().color_pink()
        }
        #[inline]
        fn color_blue(self) -> CynthiaColoredString {
            self.as_str().color_blue()
        }
        #[inline]
        fn color_lightblue(self) -> CynthiaColoredString {
            self.as_str().color_lightblue()
        }
        #[inline]
        fn color_orange(self) -> CynthiaColoredString {
            self.as_str().color_orange()
        }
        #[inline]
        fn color_bright_orange(self) -> CynthiaColoredString {
            self.as_str().color_bright_orange()
        }
        #[inline]
        fn color_purple(self) -> CynthiaColoredString {
            self.as_str().color_purple()
        }
        #[inline]
        fn color_lilac(self) -> CynthiaColoredString {
            self.as_str().color_lilac()
        }
    }
}
