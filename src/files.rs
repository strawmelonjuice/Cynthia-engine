use crate::jsr::{jsruntime, BUNJSR, BUN_NPM_EX, NODEJSR, NODEJSR_EX};
use crate::logger::logger;
use crate::structs::CynthiaCacheIndexObject;
use rand::Rng;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn cachefolder() -> PathBuf {
    let fl = Path::new("./.cynthiaTemp/").join(format!("cache/{}", std::process::id()));
    fs::create_dir_all(&fl).unwrap();
    fl
}
fn cached(file: String, max_age: u64) -> Result<String, Error> {
    // Returns either a cached file path (in string), or an error.
    let cacheindex: Vec<CynthiaCacheIndexObject> =
        match fs::read_to_string(cachefolder().join("./index.json")) {
            Ok(g) => serde_json::from_str(g.as_str()).unwrap(),
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
            } else {
                if Path::new(f.cachepath.as_str()).exists() {
                    fs::remove_file(Path::new(f.cachepath.as_str())).unwrap();
                }

            };
        }
    }
    return Err(Error::from(ErrorKind::Other));
}

fn cacher(fileid: String, contents: String) -> String {
    let mut cacheindex: Vec<CynthiaCacheIndexObject> =
        match fs::read_to_string(cachefolder().join("./index.json")) {
            Ok(g) => serde_json::from_str(g.as_str()).unwrap(),
            Err(_) => [].to_vec(),
        };
    let cachepath = cachefolder()
        .join(format!(
            "{}_cyncache",
            rand::thread_rng().gen_range(10000000..999999999)
        ))
        .display()
        .to_string();
    let mut cachedfile = File::create(cachepath.clone()).unwrap();
    write!(cachedfile, "{}", contents).unwrap();
    let new = CynthiaCacheIndexObject {
        fileid,
        cachepath: cachepath.to_string(),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };
    cacheindex.push(new);
    if cachefolder().join("./index.json").exists() {
        fs::remove_file(cachefolder().join("./index.json")).unwrap();
    }
    let mut cacheindexfile = File::create(cachefolder().join("./index.json")).unwrap();
    let line = serde_json::to_string(&cacheindex).unwrap();
    let linestr = line.as_str();
    write!(cacheindexfile, "{}", linestr).unwrap();
    return contents;
}

pub(crate) fn import_js_minified(scriptfile: String) -> String {
    return match cached((&scriptfile).to_string(), 120) {
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
                let res: String = String::from_utf8_lossy(&output.stdout).parse().unwrap();
                cacher(scriptfile, format!(
                        "\n\r// Minified internally by Cynthia using Terser\n\n{}\n\n\r// Cached after minifying, so might be ~2 minutes behind.\n\r",
                        res
                    ))
            }
            NODEJSR => {
                let output = match std::process::Command::new(NODEJSR_EX)
                    .args([
                        "-y",
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
                let res: String = String::from_utf8_lossy(&output.stdout).parse().unwrap();
                cacher(scriptfile, format!(
                        "\n\r// Minified internally by Cynthia using Terser\n\n{}\n\n\r// Cached after minifying, so might be ~2 minutes behind.\n\r",
                        res
                    ))
            }
            _ => {
                logger(5, String::from("Couldn't minify inlined javascript because there is no found javascript run time, may increase bandwidth and slow down served web pages."));
                let output =
                    fs::read_to_string(scriptfile).expect("Couldn't find or open a JS file.");
                output
            }
        },
    };
}
