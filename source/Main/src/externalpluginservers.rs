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

use std::process::Command;
use std::sync::Arc;

use actix_web::web::Data;
use interactive_process::InteractiveProcess;
use log::debug;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use tokio::sync::mpsc::Receiver;
use tokio::sync::Mutex;

use crate::{EPSCommunicationsID, ServerContext};
use crate::config::CynthiaConfig;
use crate::files::tempfolder;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct EPSRequest {
    id: EPSCommunicationsID,
    pub(crate) body: EPSRequestBody,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "for")]
pub(crate) enum EPSRequestBody {
    Close,
    Test { test: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct EPSResponse {
    id: EPSCommunicationsID,
    pub(crate) body: EPSResponseBody,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "as")]
pub(crate) enum EPSResponseBody {
    NoneOk,
    OkString { value: String },
    Json { value: String },
    Error { message: Option<String> },
}

pub(crate) async fn main(
    _server_context_mutex: Arc<Mutex<ServerContext>>,
    mut eps_r: Receiver<EPSRequest>,
) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let config_clone = {
        // We need to clone the config because we can't hold the lock while we're in the tokio runtime.
        let server_context = _server_context_mutex.lock().await;
        server_context.config.clone()
    };
    // We gotta write the javascript to a temporary file and then run it.
    let jstempfolder = tempfolder().join("js");
    std::fs::create_dir_all(&jstempfolder).unwrap();
    let jsfile = include_bytes!("../../../target/generated/js/plugins-runtime.js");
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
                if o.starts_with("parse: ") {
                    let l = o.split("parse: ").collect::<Vec<&str>>()[1];
                    let mut z = y.lock().unwrap();
                    z.push_str(l);
                    debug!("JsPluginRuntime is now parsing `{l}` of `{z}`");
                    let q = from_str::<EPSResponse>(z.as_str());
                    match q {
                        Ok(o) => {
                            debug!("JsPluginRuntime parsed a response: {:?}", o);
                            rt.spawn(and_now(o, _server_context_mutex.clone()));
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
                        config_clone
                            .clone()
                            .tell(format!("[JsPluginRuntime]: {}", o));
                    }
                }
            }
            _ => {}
        }
    })
    .unwrap();
    loop {
        match eps_r.recv().await {
            Some(o) => {
                let mut s = String::from("parse: ");
                s.push_str(serde_json::to_string(&o).unwrap().as_str());
                debug!("Sending to JsPluginRuntime: `{}`", s);
                proc.send(s.as_str()).unwrap();
            }
            _ => {}
        }
    }
}

async fn and_now(res: EPSResponse, _server_context_mutex: Arc<Mutex<ServerContext>>) {
    let mut server_context = _server_context_mutex.lock().await;
    server_context
        .external_plugin_server
        .response_queue
        .push(Some(res));
    debug!("Added response to external plugin server queue.");
    // panic!("The function runs! Finally! It runs!");
}

pub(crate) async fn contact_eps(
    _server_context_mutex: Data<Arc<Mutex<ServerContext>>>,
    req: EPSRequestBody,
) -> EPSResponse {
    let random_id = {
        let mut d: EPSCommunicationsID;
        loop {
            d = rand::random::<EPSCommunicationsID>();
            //     Verify that this number is not already in the vector of unreturned responses.
            let mut server_context = _server_context_mutex.lock().await;
            if !server_context
                .external_plugin_server
                .response_queue
                .iter()
                .any(|o| match o {
                    Some(a) => a.id == d,
                    None => false,
                })
            {
                // It's unique! Now add it to the vector to claim it.
                server_context.external_plugin_server.unreturned_ids.push(d);
                break
            } else {
                continue;
            };
        }
        d
    };

    let eps_r = {
        let server_context = _server_context_mutex.lock().await;
        server_context.external_plugin_server.sender.clone()
    };
    match eps_r
        .send(EPSRequest {
            id: random_id,
            body: req,
        })
        .await
    {
        Ok(_) => {
            debug!("Sent request to external plugin server.");
        }
        _ => {
            panic!("Failed to send request to external plugin server.");
        }
    };
    // This function sends a request over mpsc to the externalpluginservers::main function, then periodically locks the server mutex and checks if a corresponding response (matched by `id`) is added, if not, it will try again.
    // After sending, check for received responses.
    let mut wait = tokio::time::interval(tokio::time::Duration::from_micros(60));
    loop {
        wait.tick().await;
        {
            // Lock the server context mutex and check if the response is in the queue.
            let mut server_context = _server_context_mutex.lock().await;
            // Remove every none value from server_context.external_plugin_server.response_queue
            server_context
                .external_plugin_server
                .response_queue
                .retain(|o| match o {
                    Some(_) => true,
                    None => false,
                });

            let left_threads = server_context.external_plugin_server.unreturned_ids.len();
            for o in server_context
                .external_plugin_server
                .response_queue
                .iter_mut()
            {
                if let Some(a) = o {
                    debug!("[EPSQuechecker]: Checking response from external plugin server queue: {:?}", a);
                    if a.id == random_id {
                        // Match! Return the response and remove it from the vector.
                        drop(wait);
                        // Remove it from the unreturned vec
                        let p = o.take().unwrap();
                        drop(server_context);
                        {
                            let mut server_context = _server_context_mutex.lock().await;
                            server_context.external_plugin_server.unreturned_ids.retain(
                                |a| a != &random_id
                            );
                            return p;
                        }
                    } else {
                        debug!(
                            "[EPSQuechecker]: No match. Continuing.\n\n\n\r{} <-- What we expected\n\r{} <-- What we got",
                            random_id, a.id
                        );
                        // No match! Another thread wants this. Keep it in the vector and continue.
                        // Unless there should be no other thread! Check for this by:
                        if left_threads <= 1 {
                            panic!("Incorrect data in the js queue. Might the ID's be altered by js's rounding?")
                        }
                    }
                };
            }
        }
    }
}
