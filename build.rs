/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */

const FILES: [&str; 1] = ["cleansheet"];

fn main() {
    {
        #[cfg(feature = "selfinit")]
        {
            let packed_folder = {
                let mut pkd = Vec::new();
                let mut archive = tar::Builder::new(&mut pkd);
                archive.append_dir_all("", "cleansheet").unwrap();
                archive.finish().unwrap();
                drop(archive);
                pkd
            };

            let compressed_file = lzma::compress(&packed_folder, 9).unwrap();
            std::fs::write("./target/cleansheet.tar.xz", compressed_file).unwrap();
            {
                let mut filelist: String = String::new();
                let paths = std::fs::read_dir("./cleansheet/").unwrap();

                for path in paths {
                    filelist.push_str(
                        format!("{}\n", path.unwrap().path().display())
                            .as_str()
                            .replace("./cleansheet/", "")
                            .as_str(),
                    );
                }
                std::fs::write("./target/cleansheet.filelist.txt", filelist).unwrap();
            }
        }
    }
    #[cfg(not(feature = "js_runtime"))]
    println!("cargo:warning=Node features are disabled. This means you won't need a node runtime to build or run Cynthia. It also means that some features are disabled.");

    #[cfg(feature = "js_runtime")]
    {
        exec_runx(
            &["-v"],
            "bunx or npx not found. Please install bunx or npx to build this project, or disable the node feature. <https://doc.rust-lang.org/cargo/reference/features.html#command-line-feature-options>",
        );
        exec_runx(
            &["i"],
            "Failed to install node dependencies. Please re-run `bun install` manually.",
        );
        exec_runx(
        &["--bun", "run", "build:deps"],
        "Failed to build dependencies with any runtime. Please re-run `bun run build:deps` manually.",
        );
    }
    for file in FILES.iter() {
        println!("cargo:rerun-if-changed={}", file);
    }
}
#[cfg(feature = "js_runtime")]
fn exec_runx(args: &[&str], if_fails: &str) {
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
