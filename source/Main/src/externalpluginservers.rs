/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */

// Some plugins in Cynthia v2 served assets and pages over their own servers. These would be "proxied" by Cynthia.
// That functionality was derived from Cynthia v0/typescript, which would just hook those plugins onto the main
// server without requiring plugins to be written in Rust.
// This module will be a testing ground. V2 was unreliable and had a lot of issues, especially because it didn't keep the servers attached. It just let them run.
// This module will be a testing ground for a new system that will be more reliable and more secure.
// More specifically: The plugins will attach to js again, but inside of a controlled environment.
pub(crate) struct EPSRequest {
    pub(crate) id: u64,
    pub(crate) command: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct EPSResponse {
    pub(crate) id: u64,
    pub(crate) body: EPSResponseBody,
}
#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum EPSResponseBody {
    NoneOk,
    Json(String),
//     We'll add more types later, these will be very specific to the calls made by the server.
}

use interactive_process::InteractiveProcess;
use serde_json::from_str;
use std::process::Command;
use std::sync::{Arc};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Receiver;
use crate::files::tempfolder;
use crate::ServerContext;


pub(crate) async fn main(_server_context_mutex: Arc<tokio::sync::Mutex<ServerContext>>, mut eps_r: Receiver<EPSRequest>) {
    let config_clone = {
        // We need to clone the config because we can't hold the lock while we're in the tokio runtime.
        let server_context = _server_context_mutex.lock().await;
        server_context.config.clone()
    };
    // We gotta write the javascript to a temporary file and then run it.
    let jstempfolder = tempfolder().join("js");
    std::fs::create_dir_all(&jstempfolder).unwrap();
    let jsfile = include_bytes!("../target/generated/js/main.js");
    std::fs::write(jstempfolder.join("main.js"), jsfile).unwrap();
    // now we can run the javascript
    let node_runtime: &str = config_clone.runtimes.node.as_ref();
    let mut r = Command::new(node_runtime);
    r.arg(jstempfolder.join("main.js"));
    let p = Arc::new(std::sync::Mutex::new(String::new()));
    let mut proc = InteractiveProcess::new(&mut r, move |line| {
        let y = p.clone();
        match line {
            Ok(o) => {
                if o.starts_with("send: ") {
                    let l = o.split("send: ").collect::<Vec<&str>>()[1];
                    let mut z = y.lock().unwrap();
                    z.push_str(l);
                    let q = from_str::<EPSResponse>(z.as_str());
                    match q {
                        Ok(o) => {
                            println!("{:#?}", o);
                            z.clear();
                        }
                        _ => {}
                    }
                } else {
                    if o.replace("\n", "").is_empty() {
                    //     Just wait for the next line lol
                    } else {
                        let mut z = y.lock().unwrap();
                        z.clear();
                        println!("{}", o);
                    }
                }
            }
            _ => {}
        }
    })
    .unwrap();
    loop {
        match eps_r.recv().await {
            Some(r) => {
                match r.command.as_str() {
                    "close" => {
                        proc.send("close").unwrap()
                    }
                    _ => {}
                }
            }
            _ => {}
        }

    }
}

fn contact_eps() {
    todo!()
}