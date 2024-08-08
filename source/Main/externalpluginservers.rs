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

use crate::config::ConfigExternalJavascriptRuntime;

#[cfg(feature = "js_runtime")]
#[derive(Debug)]
pub(crate) struct EPSCommunicationData {
    #[cfg(feature = "js_runtime")]
    /// The sender to the (NodeJS) external plugin server not to be used directly.
    sender: tokio::sync::mpsc::Sender<EPSRequest>,
    /// The responses from the external plugin servers
    #[cfg(feature = "js_runtime")]
    response_queue: Vec<Option<EPSResponse>>,
    /// The IDs that have been sent to the external plugin servers but have not been returned yet.
    #[cfg(feature = "js_runtime")]
    unreturned_ids: Vec<EPSCommunicationsID>,
}

#[cfg(feature = "js_runtime")]
impl EPSCommunicationData {
    #[cfg(feature = "js_runtime")]
    pub(crate) fn new(sender: tokio::sync::mpsc::Sender<EPSRequest>) -> Self {
        Self {
            sender,
            response_queue: vec![],
            unreturned_ids: vec![],
        }
    }
}

#[cfg(feature = "js_runtime")]
use std::process::Command;
use std::sync::Arc;

use actix_web::web::Data;
#[cfg(feature = "js_runtime")]
use interactive_process::InteractiveProcess;

use log::warn;
#[cfg(feature = "js_runtime")]
use log::{debug, error, info};
use serde::{Deserialize, Serialize};

#[cfg(feature = "js_runtime")]
use serde_json::from_str;
use tokio::sync::mpsc::Receiver;
use tokio::sync::Mutex;

use crate::EPSCommunicationsID;
use crate::ServerContext;
#[cfg(feature = "js_runtime")]
use crate::{config::CynthiaConfig, files::tempfolder};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct EPSRequest {
    id: EPSCommunicationsID,
    pub(crate) body: EPSRequestBody,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "for")]
pub(crate) enum EPSRequestBody {
    Close,
    Test {
        test: String,
    },
    ContentRenderRequest {
        template_path: String,
        template_data: crate::renders::PageLikePublicationTemplateData,
    },
    WebRequest {
        page_id: String,
        headers: Vec<(String, String)>, // Name, Value
        method: String,
    },
}
#[derive(Serialize, Deserialize, Debug, Clone)]
struct EPSResponse {
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
    Disabled,
}

#[cfg(not(feature = "js_runtime"))]
pub(crate) async fn main(_: Arc<Mutex<ServerContext>>, _: Receiver<EPSRequest>) {
    warn!("The NodeJS runtime is not enabled. The external node plugin servers will not work.");
}

#[cfg(feature = "js_runtime")]
pub(crate) async fn main(
    server_context_mutex: Arc<Mutex<ServerContext>>,
    mut eps_r: Receiver<EPSRequest>,
) {
    let config_clone = {
        // We need to clone the config because we can't hold the lock while we're in the tokio runtime.
        let server_context = server_context_mutex.lock().await;
        server_context.config.clone()
    };
    // We gotta write the javascript to a temporary file and then run it.
    let jstempfolder = tempfolder().join("js");
    std::fs::create_dir_all(&jstempfolder).unwrap();
    let jsfile = include_bytes!("../../target/generated/js/plugins-runtime.js");
    std::fs::write(jstempfolder.join("main.mjs"), jsfile).unwrap();
    // now we can run the javascript
    let node_runtime: &str = config_clone.runtimes.ext_js_rt.as_ref();
    let mut r = Command::new(node_runtime);
    if config_clone.runtimes.ext_js_rt.validate().is_err() {
        error!("Invalid node runtime path. Plugins will not run.");
        loop {
            if let Some(o) = eps_r.recv().await {
                let q = EPSResponse {
                    id: o.id,
                    body: EPSResponseBody::Disabled,
                };
                and_now(q, server_context_mutex.clone()).await
            }
        }
    };
    let rt = tokio::runtime::Runtime::new().unwrap();
    if node_runtime.contains("deno") {
        r.arg("run");
        r.arg("--allow-read");
    }
    r.arg(jstempfolder.join("main.mjs"));
    let p = Arc::new(std::sync::Mutex::new(String::new()));
    let mut proc = InteractiveProcess::new(&mut r, move |line| {
        let y = p.clone();
        if let Ok(o) = line {
            if o.starts_with("parse: ") {
                let l = o.split("parse: ").collect::<Vec<&str>>()[1];
                let mut z = y.lock().unwrap();
                z.push_str(l);
                debug!("JsPluginRuntime is now parsing `{l}` of `{z}`");
                let q = from_str::<EPSResponse>(z.as_str());
                if let Ok(o) = q {
                    debug!("JsPluginRuntime parsed a response: {:?}", o);
                    rt.spawn(and_now(o, server_context_mutex.clone()));
                    z.clear();
                }
            } else if o.replace("\n", "").is_empty() {
                //     Just wait for the next line
            } else {
                let mut z = y.lock().unwrap();
                z.clear();
                if o.starts_with("info: ") {
                    info!(
                        "[JsPluginRuntime]: {}",
                        o.split("info: ").collect::<Vec<&str>>()[1]
                    );
                } else if o.starts_with("debug: ") {
                    debug!(
                        "[JsPluginRuntime]: {}",
                        o.split("debug: ").collect::<Vec<&str>>()[1]
                    );
                } else if o.starts_with("error: ") {
                    error!(
                        "[JsPluginRuntime]: {}",
                        o.split("error: ").collect::<Vec<&str>>()[1]
                    );
                } else if o.starts_with("warn: ") {
                    warn!(
                        "[JsPluginRuntime]: {}",
                        o.split("warn: ").collect::<Vec<&str>>()[1]
                    );
                } else if o.starts_with("log: ") {
                    config_clone.clone().tell(format!(
                        "[JsPluginRuntime]: {}",
                        o.split("log: ").collect::<Vec<&str>>()[1]
                    ));
                }
            }
        }
    })
    .unwrap();
    loop {
        if let Some(o) = eps_r.recv().await {
            let mut s = String::from("parse: ");
            s.push_str(serde_json::to_string(&o).unwrap().as_str());
            debug!("Sending to JsPluginRuntime: `{}`", s);
            proc.send(s.as_str()).unwrap();
        }
    }
}

#[cfg(feature = "js_runtime")]
async fn and_now(res: EPSResponse, _server_context_mutex: Arc<Mutex<ServerContext>>) {
    let mut server_context = _server_context_mutex.lock().await;
    server_context
        .external_plugin_server
        .response_queue
        .push(Some(res));
    debug!("Added response to external plugin server queue.");
    // panic!("The function runs! Finally! It runs!");
}
/**
This function sends a request over mpsc to the externalpluginservers::main function, then periodically locks the server mutex and checks if a corresponding response (matched by `id`) is added, if not, it will try again.
It is recommended to use this function instead of other methods of sending requests to the external plugin server.
*/
#[cfg(feature = "js_runtime")]
pub(crate) async fn contact_eps(
    server_context_mutex: Data<Arc<Mutex<ServerContext>>>,
    req: EPSRequestBody,
) -> EPSResponseBody {
    use crate::LockCallback;
    if server_context_mutex
        .lock_callback(|server_context| -> Option<EPSResponseBody> {
            if server_context.config.runtimes.ext_js_rt.validate().is_err()
                || server_context.config.runtimes.ext_js_rt == "disabled"
            {
                Some(EPSResponseBody::Disabled)
            } else {
                None
            }
        })
        .await
        .is_some()
    {
        return EPSResponseBody::Disabled;
    };
    let random_id = {
        let mut d: EPSCommunicationsID;
        loop {
            d = rand::random::<EPSCommunicationsID>();
            //     Verify that this number is not already in the vector of unreturned responses.
            let mut server_context = server_context_mutex.lock().await;
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
                break;
            } else {
                continue;
            };
        }
        d
    };

    let eps_r = {
        let server_context = server_context_mutex.lock().await;
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
    // After sending, check for received responses.
    let mut wait = tokio::time::interval(tokio::time::Duration::from_micros(60));
    loop {
        wait.tick().await;
        {
            // Lock the server context mutex and check if the response is in the queue.
            let mut server_context = server_context_mutex.lock().await;
            // Remove every none value from server_context.external_plugin_server.response_queue
            server_context
                .external_plugin_server
                .response_queue
                .retain(|o| o.is_some());

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
                        let p = o.take().unwrap().body;
                        drop(server_context);
                        {
                            let mut server_context = server_context_mutex.lock().await;
                            server_context
                                .external_plugin_server
                                .unreturned_ids
                                .retain(|a| a != &random_id);
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

#[cfg(not(feature = "js_runtime"))]
pub(crate) async fn contact_eps(
    _: Data<Arc<Mutex<ServerContext>>>,
    _: EPSRequestBody,
) -> EPSResponseBody {
    EPSResponseBody::Disabled
}
