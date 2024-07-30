/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */

use crate::{logger, LoadedData};
use actix_web::web::Data;
use actix_web::{post, web, HttpResponse};
use async_std::task;
use normalize_path::NormalizePath;
use random_string::generate_rng;
use serde::Deserialize;
use std::fs;
use std::fs::File;
use std::io::{Error, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

#[derive(Deserialize)]
struct DashAPIData {
    passkey: String,
    command: String,
    subcommand: String,
    params: String,
}

#[derive(Deserialize)]
struct PluginDashInstallParams(String, Option<String>);

#[derive(Deserialize)]
struct PluginDashRemoveParams {
    plugin_name: String,
}

pub(crate) async fn main() {
    // As long as we don't have anything to fill this with.
    task::sleep(Duration::from_secs(1)).await;
}

pub(crate) async fn d_main(enabled: bool) {
    if !enabled {
        task::sleep(Duration::from_secs(1)).await;
        info!(String::from("CynthiaDash is disabled."));
    } else {
        main().await
    }
}
