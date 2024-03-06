use crate::logger;
use crate::structs::PluginMeta;
use actix_web::web::Data;
use actix_web::{post, web, HttpResponse};
use normalize_path::NormalizePath;
use random_string::generate_rng;
use serde::Deserialize;
use std::fs;
use std::fs::File;
use std::io::{Error, Write};
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Deserialize)]
struct DashAPIData {
    passkey: String,
    command: String,
    subcommand: String,
    params: String,
}

#[derive(Deserialize)]
struct PluginDashInstallParams {
    plugin_name: String,
    plugin_version: String,
}
#[derive(Deserialize)]
struct PluginDashRemoveParams {
    plugin_name: String
}

#[post("/dashapi/")]
pub(crate) async fn dashserver(
    data: web::Form<DashAPIData>,
    _pluginsmex: Data<Mutex<Vec<PluginMeta>>>,
) -> HttpResponse {
    if data.passkey != passkey().unwrap_or(String::from("")) {
        logger::general_warn(
            String::from(
                "An unauthorized external entity just tried performing an action on this instance.",
            ),
        );
        return HttpResponse::Forbidden().body(String::from(
            "<h1>NO ACCESS!</h1>Wrong passkey entered. A report has been sent to the server logs.",
        ));
    }

    match data.command.as_str() {
        "log" => {
            logger::log_by_act_num(
                data.subcommand.parse().unwrap_or(1),
                format!("{{CynthiaDash}} {}", data.params.clone()),
            );
        }
        "plugin" => match data.subcommand.as_str() {
            "remove" => match serde_json::from_str(&data.params) {
                Ok(s) => {
                    let plugindata: PluginDashRemoveParams = s;
                    crate::subcommand::plugin_remove(
                        plugindata.plugin_name
                    );
                },
                Err(_e) => {
                    return HttpResponse::BadRequest().body(String::from("Invalid plugin."));
                }
            }
            "install" => match serde_json::from_str(&data.params) {
                Ok(s) => {
                    let plugindata: PluginDashInstallParams = s;
                    crate::subcommand::plugin_install(
                        plugindata.plugin_name,
                        plugindata.plugin_version,
                    );
                },

                Err(_e) => {
                    return HttpResponse::BadRequest().body(String::from("Invalid plugin."));
                }
            },
            _ => {
                return HttpResponse::BadRequest().body(String::from("Invalid command."));
            }
        },
        _ => {
            return HttpResponse::BadRequest().body(String::from("Invalid command."));
        }
    }
    HttpResponse::Ok().body(String::from("OK!"))
}

pub fn passkey() -> Result<String, Error> {
    let fl = PathBuf::from("./.cynthiaTemp/dashpasskeys/");
    fs::create_dir_all(&fl).unwrap();
    let filepath = fl.join(format!("{}.TXT", std::process::id())).normalize();
    if filepath.exists() {
        let pk = fs::read_to_string(filepath)?;
        Ok(pk)
    } else {
        let pk: String = generate_rng(15..70, random_string::charsets::ALPHANUMERIC);
        let mut fi: File = File::create(filepath.clone())?;
        write!(fi, "{}", pk)?;
        Ok(pk)
    }
}
