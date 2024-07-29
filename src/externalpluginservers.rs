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

use interactive_process::InteractiveProcess;
use serde_json::from_str;
use std::process::Command;
use std::sync::{Arc, Mutex};
#[derive(Debug, serde::Deserialize)]
struct Testje {
    test: String,
    geel: Vec<String>,
}

pub(crate) async fn main() {
    let mut r = Command::new("node");
    r.arg("../src-js/main.js");
    let p = Arc::new(Mutex::new(String::new()));
    let mut proc = InteractiveProcess::new(&mut r, move |line| {
        let y = p.clone();
        match line {
            Ok(o) => {
                if o.starts_with("send: ") {
                    let l = o.split("send: ").collect::<Vec<&str>>()[1];
                    let mut z = y.lock().unwrap();
                    z.push_str(l);
                    let q = from_str::<Testje>(z.as_str());
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
}
