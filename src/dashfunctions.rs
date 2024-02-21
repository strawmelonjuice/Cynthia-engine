use normalize_path::NormalizePath;
use random_string::generate_rng;
use std::fs;
use std::fs::File;
use std::io::{ Error, Write };
use std::path::PathBuf;
use std::sync::Mutex;
use actix_web::{ post, HttpResponse, web };
use actix_web::web::Data;
use serde::Deserialize;
use crate::logger::logger;
use crate::structs::PluginMeta;

#[derive(Deserialize)]
struct DashAPIData {
    passkey: String,
    command: String,
    subcommand: String,
    params: String,
}
#[post("/dashapi/")]
pub(crate) async fn dashserver(
    data: web::Form<DashAPIData>,
    _pluginsmex: Data<Mutex<Vec<PluginMeta>>>
) -> HttpResponse {
    if data.passkey != passkey().unwrap_or(String::from("")) {
        logger(
            15,
            String::from(
                "An unauthorized external entity just tried performing an action on this instance."
            )
        );
        return HttpResponse::Forbidden().body(
            String::from("Wrong passkey entered. A report has been sent to the server owner.")
        );
    }

    match data.command.as_str() {
        "log" => {
            logger(
                data.subcommand.parse().unwrap_or(1),
                format!("{{CynthiaDash}} {}", data.params.clone())
            );
        }
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
