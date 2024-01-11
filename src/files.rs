use crate::jsr::{jsruntime, BUNJSR, BUN_NPM_EX, NODEJSR, NODEJSR_EX};
use crate::logger::logger;
use crate::structs::CynthiaCacheIndexObject;
use colored::Colorize;
use normalize_path::NormalizePath;
use rand::Rng;
use std::fs::File;
use std::io::Write;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs;
use dotenv::dotenv;

fn cachefolder() -> PathBuf {
    let fl = PathBuf::from("./.cynthiaTemp/cache/")
        .join(format!("{}", std::process::id()))
        .normalize();
    // logger(31, format!("Cache folder: {}", fl.display()));
    fs::create_dir_all(&fl).unwrap();
    fl
}
pub(crate) fn cacheretriever(file: String, max_age: u64) -> Result<PathBuf, Error> {
    // Returns either a cached file path (in string), or an error.
    let cacheindex: Vec<CynthiaCacheIndexObject> =
        match fs::read_to_string(cachefolder().join("./index.json")) {
            Ok(g) => serde_json::from_str(g.as_str())?,
            Err(_) => return Err(Error::from(ErrorKind::Other)),
        };
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    for f in cacheindex {
        if f.fileid == file {
            if (now - f.timestamp) < max_age {
                return Ok(f.cachepath);
            } else if Path::new(&f.cachepath).exists() {
                logger(31, format!("Cache {}: {} at {}, reason: Too old!","removed".red(), file, &f.cachepath.display()));
                fs::remove_file(Path::new(&f.cachepath)).unwrap();
            };
        }
    }
    Err(Error::from(ErrorKind::Other))
}

pub(crate) fn cacheplacer(fileid: String, contents: String) -> String {
    let mut cacheindex: Vec<CynthiaCacheIndexObject> =
        match fs::read_to_string(cachefolder().join("./index.json")) {
            Ok(g) => serde_json::from_str(g.as_str()).unwrap(),
            Err(_) => [].to_vec(),
        };
    let cachepath = cachefolder()
        .join(
            rand::thread_rng()
                .gen_range(10000000..999999999)
                .to_string(),
        )
        .normalize();

    let mut cachedfile = File::create(cachepath.clone()).unwrap();
    write!(cachedfile, "{}", contents).unwrap();
    logger(31, format!("Cache {}: {} in {}", "placed".green(), fileid, cachepath.display()));
    let new = CynthiaCacheIndexObject {
        fileid,
        cachepath,
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };
    cacheindex.push(new);
    if cachefolder().join("./index.json").exists() {
        let _ = fs::remove_file(cachefolder().join("./index.json"));
    }
    let mut cacheindexfile = File::create(cachefolder().join("./index.json")).unwrap();
    let line = serde_json::to_string(&cacheindex).unwrap();
    let linestr = line.as_str();
    write!(cacheindexfile, "{}", linestr).unwrap();
    contents
}

pub(crate) fn import_js_minified(scriptfile: String) -> String {
    dotenv().ok();
    let jscachelifetime: u64 = match std::env::var("JAVASCRIPT_CACHE_LIFETIME") {
        Ok(g) => g.parse::<u64>().unwrap(),
        Err(_) => 1200,
    };
    return match cacheretriever(scriptfile.to_string(), jscachelifetime) {
        Ok(o) => fs::read_to_string(o).expect("Couldn't find or open a JS file."),
        Err(_) => match jsruntime(true) {
            BUNJSR => {
                let output = match std::process::Command::new(BUN_NPM_EX)
                    .args([
                        "terser",
                        scriptfile.as_str(),
                        "--compress",
                        "--keep-fnames",
                        "--keep-classnames",
                    ])
                    .output()
                {
                    Ok(result) => result,
                    Err(_erro) => {
                        logger(5, String::from("Couldn't launch Javascript runtime."));
                        std::process::exit(1);
                    }
                };
                if output.status.success() {
                    let res: String = String::from_utf8_lossy(&output.stdout).parse().unwrap();
                    cacheplacer(scriptfile, format!(
                        "\n\r// Minified internally by Cynthia using Terser\n\n{res}\n\n\r// Cached after minifying, so might be somewhat behind.\n\r"
                    ))
                } else {
                    logger(
                        5,
                        format!(
                            "Failed running Terser in {}, couldn't minify to embed JS.",
                            "Bunx".purple()
                        ),
                    );
                    fs::read_to_string(scriptfile).expect("Couldn't find or open a JS file.")
                }
            }
            NODEJSR => {
                let output = match std::process::Command::new(NODEJSR_EX)
                    .args([
                        "--yes",
                        "terser",
                        scriptfile.as_str(),
                        "--compress",
                        "--keep-fnames",
                        "--keep-classnames",
                    ])
                    .output()
                {
                    Ok(result) => result,
                    Err(_erro) => {
                        logger(5, String::from("Couldn't launch Javascript runtime."));
                        std::process::exit(1);
                    }
                };
                if !output.status.success() {
                    logger(
                        5,
                        format!(
                            "Failed running Terser in {}, couldn't minify to embed JS.",
                            "NPX".purple()
                        ),
                    );
                    fs::read_to_string(scriptfile).expect("Couldn't find or open a JS file.")
                } else {
                    let res: String = String::from_utf8_lossy(&output.stdout).parse().unwrap();
                    cacheplacer(scriptfile, format!(
                        "\n\r// Minified internally by Cynthia using Terser\n\n{res}\n\n\r// Cached after minifying, so might be somewhat behind.\n\r"
                    ))
                }
            }
            _ => {
                logger(15, String::from("Couldn't minify inlined javascript because there is no found javascript run time, may incre ase bandwidth and slow down served web pages."));
                fs::read_to_string(scriptfile).expect("Couldn't find or open a JS file.")
            }
        },
    };
}

pub(crate) fn import_css_minified(stylefile: String) -> String {
    let csscachelifetime: u64 = match std::env::var("STYLESHEET_CACHE_LIFETIME") {
        Ok(g) => g.parse::<u64>().unwrap(),
        Err(_) => 1200,
    };
    return match cacheretriever(stylefile.to_string(), csscachelifetime) {
        Ok(o) => fs::read_to_string(o).expect("Couldn't find or open a JS file."),
        Err(_) => match jsruntime(true) {
            BUNJSR => {
                let output = match std::process::Command::new(BUN_NPM_EX)
                    .args([
                        "clean-css-cli@4",
                        "-O2",
                        "--inline",
                        "none",
                        stylefile.as_str(),
                    ])
                    .output()
                {
                    Ok(result) => result,
                    Err(_erro) => {
                        logger(5, String::from("Couldn't launch Javascript runtime."));
                        std::process::exit(1);
                    }
                };
                if output.status.success() {
                    let res: String = String::from_utf8_lossy(&output.stdout).parse().unwrap();
                    cacheplacer(stylefile, format!(
                        "\n\r/* Minified internally by Cynthia using clean-css */\n\n{res}\n\n\r/* Cached after minifying, so might be somewhat behind. */\n\r"
                    ))
                } else {
                    logger(
                        5,
                        format!(
                            "Failed running clean-css in {}, couldn't minify to embed CSS.",
                            "Bunx".purple()
                        ),
                    );
                    fs::read_to_string(stylefile).expect("Couldn't find or open a JS file.")
                }
            }
            NODEJSR => {
                let output = match std::process::Command::new(NODEJSR_EX)
                    .args([
                        "--yes",
                        "clean-css-cli@4",
                        "-O2",
                        "--inline",
                        "none",
                        stylefile.as_str(),
                    ])
                    .output()
                {
                    Ok(result) => result,
                    Err(_erro) => {
                        logger(5, String::from("Couldn't launch Javascript runtime."));
                        std::process::exit(1);
                    }
                };
                if !output.status.success() {
                    logger(
                        5,
                        format!(
                            "Failed running clean-css in {}, couldn't minify to embed CSS.",
                            "NPX".purple()
                        ),
                    );
                    fs::read_to_string(stylefile).expect("Couldn't find or open a CSS file.")
                } else {
                    let res: String = String::from_utf8_lossy(&output.stdout).parse().unwrap();
                    cacheplacer(stylefile, format!(
                        "\n\r/* Minified internally by Cynthia using clean-css */\n\n{res}\n\n\r/* Cached after minifying, so might be somewhat behind. */\n\r"
                    ))
                }
            }
            _ => {
                logger(15, String::from("Couldn't minify inlined javascript because there is no found javascript run time, may incre ase bandwidth and slow down served web pages."));
                fs::read_to_string(stylefile).expect("Couldn't find or open a CSS file.")
            }
        },
    };
}
