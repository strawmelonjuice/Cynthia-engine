/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */

const FILES: [&str; 1] = [""];

fn main() {
    p(
        &["-v"],
        "bunx or npx not found. Please install bunx or npx to build this project.",
    );
    p(
        &["i"],
        "Failed to install node dependencies. Please re-run `bun install` manually.",
    );
    p(
        &["--bun", "run", "build:deps"],
        "Failed to build dependencies with any runtime. Please re-run `bun run build:deps` manually.",
    );
    for file in FILES.iter() {
        println!("cargo:rerun-if-changed={}", file);
    }
}

fn p(args: &[&str], if_fails: &str) {
    match if cfg!(windows) {
        ["bunx.exe", "npx.cmd"]
    } else {
        ["bunx", "npx"]
    }
    .iter()
    .find(|&runtime| {
        std::process::Command::new(runtime)
            .args(["-y", "bun"])
            .args(args)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }) {
        Some(_) => {}
        None => {
            println!("cargo:warning={}", if_fails);
        }
    }
}
