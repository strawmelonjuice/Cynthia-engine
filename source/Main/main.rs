/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */

use actix_web::web::Data;
use actix_web::{App, HttpServer};
use colored::Colorize;
use futures::join;
#[allow(unused_imports)]
use log::info;
use log::LevelFilter;
use log::{debug, error};
use simplelog::{ColorChoice, CombinedLogger, TermLogger, TerminalMode, WriteLogger};
use std::fs::File;
use std::ops::Add;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, process};
use tokio::sync::{Mutex, MutexGuard};

use crate::config::{CynthiaConf, CynthiaConfig, SceneCollectionTrait};
use crate::externalpluginservers::EPSRequest;
use crate::files::CynthiaCache;
use crate::tell::horizline;

mod config;
mod externalpluginservers;
mod files;
mod publications;
mod renders;
mod requestresponse;
mod tell;
// mod jsrun;

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

    #[cfg(feature = "node")]
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

#[cfg(feature = "node")]
use crate::externalpluginservers::EPSCommunicationData;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    println!(
        " \u{21E2} cynthiaweb {}",
        args.get(1)
            .unwrap_or(&String::from(""))
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
                "convert [format]".bold().yellow(),
                ": Converts the configuration to the specified format.".bright_green(),
                "Available formats: `dhall`, `toml`, `json`."
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
            convert_config(args.get(2).unwrap());
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
const CONFIG_LOCATIONS: [&str; 3] = [("Cynthia.dhall"), ("Cynthia.toml"), ("Cynthia.json")];

fn load_config() -> CynthiaConf {
    let unfound = || {
        eprintln!("Could not find cynthia-configuration at `{}`! Have you initialised a Cynthia setup here? To do so, run `{}`.",
                  std::env::current_dir().unwrap().join("Cynthia.toml").clone().to_string_lossy().replace("\\\\?\\", "").bright_cyan(),
                  "cynthiaweb init".bright_green());
        process::exit(1);
    };
    let cd = std::env::current_dir().unwrap();
    // In order of preference for Cynthia. I personally prefer TOML, but Cynthia would prefer Dhall. Besides, Dhall is far more powerful.
    let config_locations: Vec<PathBuf> = CONFIG_LOCATIONS.iter().map(|p| cd.join(p)).collect();
    let chosen_config_location = config_locations.iter().position(|p| p.exists());
    if let None = chosen_config_location {
        unfound();
    }
    return match chosen_config_location.unwrap() {
        2 => {
            let cynthiaconfpathjson: PathBuf = config_locations
                .get(chosen_config_location.unwrap())
                .unwrap()
                .clone();
            match fs::read_to_string(cynthiaconfpathjson.clone()) {
                Ok(g) => match serde_json::from_str(&g) {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!(
                            "{}\n\nReason:\n{}",
                            format!(
                                "Could not interpret cynthia-configuration at `{}`!",
                                cynthiaconfpathjson
                                    .clone()
                                    .to_string_lossy()
                                    .replace("\\\\?\\", "")
                            )
                            .bright_red(),
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
                            cynthiaconfpathjson
                                .clone()
                                .to_string_lossy()
                                .replace("\\\\?\\", "")
                        )
                        .bright_red(),
                        format!("{}", e).on_red()
                    );
                    process::exit(1);
                }
            }
        }
        1 => {
            let cynthiaconfpathtoml: PathBuf = config_locations
                .get(chosen_config_location.unwrap())
                .unwrap()
                .clone();
            match fs::read_to_string(cynthiaconfpathtoml.clone()) {
                Ok(g) => match toml::from_str(&g) {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!(
                            "{}\n\nReason:\n{}",
                            format!(
                                "Could not interpret cynthia-configuration at `{}`!",
                                cynthiaconfpathtoml
                                    .clone()
                                    .to_string_lossy()
                                    .replace("\\\\?\\", "")
                            )
                            .bright_red(),
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
                            cynthiaconfpathtoml
                                .clone()
                                .to_string_lossy()
                                .replace("\\\\?\\", "")
                        )
                        .bright_red(),
                        format!("{}", e).on_red()
                    );
                    process::exit(1);
                }
            }
        }
        0 => {
            let cynthiaconfpathdhall: PathBuf = config_locations
                .get(chosen_config_location.unwrap())
                .unwrap()
                .clone();
            match fs::read_to_string(cynthiaconfpathdhall.clone()) {
                Ok(g) => match serde_dhall::from_str(&g).parse() {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!(
                            "{}\n\nReason:\n{}",
                            format!(
                                "Could not interpret cynthia-configuration at `{}`!",
                                cynthiaconfpathdhall
                                    .clone()
                                    .to_string_lossy()
                                    .replace("\\\\?\\", "")
                            )
                            .bright_red(),
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
                            cynthiaconfpathdhall
                                .clone()
                                .to_string_lossy()
                                .replace("\\\\?\\", "")
                        )
                        .bright_red(),
                        format!("{}", e).on_red()
                    );
                    process::exit(1);
                }
            }
        }
        _ => {
            unfound();
            unreachable!();
        }
    };
}

async fn start() {
    let cd = std::env::current_dir().unwrap();
    let config = load_config();
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

        #[cfg(feature = "node")]
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
    use requestresponse::serve;
    let main_server = match HttpServer::new(move || {
        App::new()
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

fn convert_config(to: &str) {
    let cd = std::env::current_dir().unwrap();
    let config = load_config().hard_clone();
    let config_serialised = match to {
        "dhall" => {
            // Dhall is a bit more complex, so we need to do some extra work here.
            // Besides, we need to add some comments to the Dhall file.
            // todo!("Add all comments to the Dhall file");
            
            let mut o = String::from("{\n\t{-\n\t\tThis is the configuration file for Cynthia. It is written in Dhall, a Haskell-like language that is able to contain functions and types.\n\n\t-}\n");
            o.push_str(
                serde_dhall::serialize(&config)
                    .static_type_annotation()
                    .to_string()
                    .unwrap()
                    .replace(",", "\n,")
                    .replace("{", "{\n")
                    .replace("}", "\n}\n")
                    .chars()
                    .skip(1)
                    .collect::<String>()
                    .as_str(),
            );
            o.replace("cache =", "{-\nThese rules are set for a reason: The higher they are set, the less requests we have to do to Node, external servers, etc.\nHigh caching lifetimes can speed up Cynthia a whole lot, so think wisely before you lower any of these numbers!\n-}\n cache =")
        }
        "toml" => { 
            todo!("Add all comments to the TOML file");
            toml::to_string_pretty(&config).unwrap() },
        "json" => { 
            todo!("Add all comments to the JSON file");
            serde_json::to_string_pretty(&config).unwrap() },
        _ => {
            eprintln!(
                "{} Could not interpret format `{}`! Please use either `dhall` or `toml`.",
                "error:".red(),
                to
            );
            process::exit(1);
        }
    };
    let config_file = cd.join("Cynthia.".to_string() + to);
    match fs::write(config_file, config_serialised) {
        Ok(_) => {
            println!(
                "{} Successfully converted the configuration to {}!",
                "Success:".green(),
                to
            );
        }
        Err(e) => {
            eprintln!(
                "{} Could not write the configuration to `{}`! Error: {}",
                "error:".red(),
                cd.join("Cynthia.".to_string() + to)
                    .to_string_lossy()
                    .replace("\\\\?\\", ""),
                e
            );
            process::exit(1);
        }
    };
    // Remove old format(s)
    let config_file = cd.join("Cynthia.".to_string() + to);

    let mut config_locations: Vec<PathBuf> = CONFIG_LOCATIONS.iter().map(|p| cd.join(p)).collect();
    config_locations.retain(|p| p.exists());
    config_locations.retain(|p| p != &config_file);
    for p in config_locations {
        match fs::remove_file(p.clone()) {
            Ok(_) => {}
            Err(e) => {
                eprintln!(
                    "{} Could not remove the old configuration file at `{}`! Error: {}",
                    "error:".red(),
                    p.to_string_lossy().replace("\\\\?\\", ""),
                    e
                );
                process::exit(1);
            }
        }
    }
}
