/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */

use actix_web::web::Data;
use actix_web::{App, HttpServer};
use colored::Colorize;
use futures::join;
use log::info;
use log::{debug, error};
use log::{trace, LevelFilter};
use requestresponse::{assets_with_cache, serve};
use simplelog::{ColorChoice, CombinedLogger, TermLogger, TerminalMode, WriteLogger};
use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, process};
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
    println!("{}", horizline().purple());

    println!(
        "{} - version {}\n by {}{}{} {}!",
        "CynthiaWeb".bold().bright_purple(),
        env!("CARGO_PKG_VERSION").to_string().green(),
        "Straw".bright_red(),
        "melon".green(),
        "juice".bright_yellow(),
        "Mar".magenta()
    );
    println!("{}", horizline().purple());
    match args
        .get(1)
        .unwrap_or(&String::from(""))
        .to_ascii_lowercase()
        .as_str()
    {
        "help" => {
            println!(
                "{}",
                "Cynthia - a simple site generator/server with a focus on performance and ease of use. Targeted at smaller sites and personal projects.".bright_magenta()
            );
            println!(
                "{}",
                "Usage: cynthiaweb [command]\n\nCommands:".bright_green()
            );
            println!(
                "\t{}{}",
                "help".bold().yellow(),
                ": Displays this message.".bright_green()
            );
            println!(
                "\t{}{}",
                "start".bold().yellow(),
                ": Starts the server.".bright_green()
            );
            println!(
                "\t{}{}\n\t\t{}",
                "convert [format] <-k>".bold().yellow(),
                ": Converts the configuration to the specified format.".bright_green(),
                "Available formats: `dhall`, `toml`, `jsonc`.".clear()
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
                     "PM".bold().yellow(),"subcommand".bright_green(),"plugin name".bright_yellow(), "plugin version".bright_purple(),
                     "plugin name".bright_yellow(),
                     "plugin version".bright_purple(),

            "cynthiapluginmanifest.json".bright_green(),);
            process::exit(0);
        }
        "start" => start().await,
        "convert" => {
            if args.len() < 3 {
                eprintln!(
                    "{} No format specified! Please run `cynthiaweb help` for a list of commands.",
                    "error:".red()
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
                "error:".red()
            );
            start().await;
            println!("And next time, try to use the `start` command directly!");
        }
        _ => {
            eprintln!(
            "{} Could not interpret command `{}`! Please run `cynthiaweb help` for a list of commands.",
            "error:".red(),
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
            "error:".red()
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
            "error:".red()
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
                        "error:".red(),
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
                e.to_string().bright_red()
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
    println!("{}", horizline().bright_purple());
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

    use colored::Colorize;
    use log::info;
    use time::{format_description, OffsetDateTime};

    use crate::config::{CynthiaConfClone, Logging};
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
}
