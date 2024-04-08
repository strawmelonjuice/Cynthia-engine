/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */

use crate::logger;

// Bun on windows is enabled by default. This is because choosing to have Bun on windows, means choosing for an experimental feature.

// Javascript runtimes:
//     NodeJS:
#[cfg(windows)]
pub const NODEJSR: &str = "node.exe";
#[cfg(not(windows))]
pub const NODEJSR: &str = "node";
//     Bun:
#[cfg(windows)]
pub const BUNJSR: &str = "bun.exe";
#[cfg(not(windows))]
pub const BUNJSR: &str = "bun";

// Javascript package managers:
//     NPM:
#[cfg(windows)]
pub const NODE_NPM: &str = "npm.cmd";
#[cfg(not(windows))]
pub const NODE_NPM: &str = "npm";
//     Bun:
#[cfg(windows)]
pub const BUN_NPM: &str = "bun.exe";
#[cfg(windows)]
pub const BUN_NPM_EX: &str = "bunx.exe";
#[cfg(not(windows))]
pub const BUN_NPM: &str = "bun";
#[cfg(not(windows))]
pub const BUN_NPM_EX: &str = "bunx";

//     NodeJS:
#[cfg(windows)]
pub const NODEJSR_EX: &str = "npx.cmd";
#[cfg(not(windows))]
pub const NODEJSR_EX: &str = "npx";
pub(crate) fn noderunner(args: Vec<&str>, cwd: std::path::PathBuf) -> String {
    if args[0] == "returndirect" {
        logger::general_warn( String::from("Directreturn called on the JSR, this usually means something inside of Cynthia's Plugin Loader went wrong."));
        return args[1].to_string();
    }
    let output = match std::process::Command::new(jsruntime(false))
        .args(args.clone())
        .current_dir(cwd)
        .output()
    {
        Ok(result) => result,
        Err(_erro) => {
            logger::general_error(String::from("Couldn't launch Javascript runtime."));
            std::process::exit(1);
        }
    };
    if output.status.success() {
        return String::from_utf8_lossy(&output.stdout)
            .into_owned()
            .to_string();
    } else {
        println!("Script failed.");
        logger::jsr_error(String::from_utf8_lossy(&output.stderr).to_string());
    }
    String::from("")
}

pub(crate) fn jsruntime(mayfail: bool) -> &'static str {
    return match std::process::Command::new(BUNJSR).arg("-v").output() {
        Ok(_t) => BUNJSR,
        Err(_err) => match std::process::Command::new(NODEJSR).arg("-v").output() {
            Ok(_t) => NODEJSR,
            Err(_err) => {
                if !mayfail {
                    logger::general_error(String::from(
                        "No supported (Node.JS or Bun) Javascript runtimes found on path!",
                    ));
                    std::process::exit(1);
                }
                ""
            }
        },
    };
}
pub(crate) fn jspm(mayfail: bool) -> &'static str {
    match std::process::Command::new(BUN_NPM).arg("-v").output() {
        Ok(_t) => BUN_NPM,
        Err(_err) => match std::process::Command::new(NODE_NPM).arg("-v").output() {
            Ok(_t) => NODE_NPM,
            Err(_err) => {
                if !mayfail {
                    logger::general_error(String::from(
                        "No supported (Node.JS or Bun) Javascript package managers found on path!",
                    ));
                    std::process::exit(1);
                }
                ""
            }
        },
    }
}
