use crate::structs::CynthiaPluginManifestItem;
use crate::{
    jsr::{BUN_NPM, BUN_NPM_EX, NODE_NPM},
    logger,
    structs::CynthiaPluginRepoItem,
};
use colored::Colorize;
use curl::easy::Easy;
use flate2::read::GzDecoder;
use random_string::generate_rng;
use std::fs::remove_dir_all;
use std::io::{Error, ErrorKind, Write};
use std::path::PathBuf;
use std::{
    fs,
    io::Read,
    path::Path,
    process::{self, Command},
};
use tar::Archive;
use urlencoding::encode;

pub(crate) fn init() {
    let tempdir = Path::new("./.cynthiatemp/").join(format!(
        "{}_cyninittemp",
        generate_rng(3..7, random_string::charsets::ALPHANUMERIC)
    ));
    let mut tarfiledownload = Vec::new();
    let mut c: Easy = Easy::new();
    match c
        .url("https://codeload.github.com/CynthiaWebsiteEngine/cleanConfig/tar.gz/refs/heads/main")
    {
        Ok(oki) => {
            logger::general_log(String::from("Downloading clean CynthiaConfig..."));
            oki
        }
        Err(_) => {
            logger::general_error(String::from(
                "Could not start clean CynthiaConfig download!",
            ));
            process::exit(1);
        }
    };
    {
        let mut transfer = c.transfer();
        transfer
            .write_function(|new_data| {
                tarfiledownload.extend_from_slice(new_data);
                Ok(new_data.len())
            })
            .unwrap();
        match transfer.perform() {
            Ok(oki) => {
                logger::general_log(String::from("Download success."));

                oki
            }
            Err(_) => {
                logger::general_error(String::from("Could not download clean CynthiaConfig!"));
                process::exit(1);
            }
        }
    }
    let tarfilecontent = &tarfiledownload;
    // Originally, I wanted to avoid downloading this, but Cargo doesn't do a great job at packaging extra files with it.
    // > let tarfilecontent = include_bytes!("../clean-cyn.tar.gz");
    // println!("lots of bytes: {:#?}", tarfilecontent);
    fs::create_dir_all(&tempdir).unwrap();
    let ctempdir = fs::canonicalize(tempdir.clone()).unwrap();
    let mut f = fs::File::create(ctempdir.join("./cyn-clean.tar.gz")).unwrap();
    Write::write_all(&mut f, tarfilecontent).unwrap();
    let tar_gz = match fs::File::open(ctempdir.join("./cyn-clean.tar.gz")) {
        Ok(f) => f,
        Err(_) => {
            logger::general_error(String::from("Could not read clean CynthiaConfig!"));
            process::exit(1);
        }
    };
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    logger::general_log(format!(
        "Unpacking new CynthiaConfig to {}...",
        fs::canonicalize(Path::new("./"))
            .unwrap()
            .display()
            .to_string()
            .replace("\\\\?\\", "")
            .cyan()
    ));
    match archive.unpack(tempdir) {
        Ok(f) => f,
        Err(_) => {
            logger::general_error(String::from("Could not unpack clean CynthiaConfig!"));
            process::exit(1);
        }
    };
    let mut options = fs_extra::dir::CopyOptions::new();
    options.overwrite = true;
    options.content_only = true;
    fs_extra::dir::copy(
        ctempdir.join("./cleanConfig-main/"),
        Path::new("./"),
        &options,
    )
    .expect("Could not create target files.");
    remove_dir_all(ctempdir).unwrap_or_default();
    let pluginmanjson = Path::new("./cynthiapluginmanifest.json");
    logger::general_info(String::from(
        "Clean CynthiaConfig written! Please adjust then restart Cynthia!",
    ));
    if pluginmanjson.exists()
        && choice(
            String::from("Do you want to install recomended plugins"),
            true,
        )
    {
        logger::general_log(format!(
            "Installing plugins specified in '{0}' now...",
            pluginmanjson.display().to_string().blue()
        ));
        let mut o = fs::File::open(format!("{}", &pluginmanjson.display()).as_str())
            .expect("Could not read Cynthia plugin manifest file.");
        let mut contents = String::new();
        o.read_to_string(&mut contents)
            .expect("Could not read Cynthia plugin manifest file.");
        let unparsed: &str = contents.as_str();
        let cynplmn: Vec<CynthiaPluginManifestItem> = serde_json::from_str(unparsed)
            .expect("Could not read from Cynthia plugin manifest file.");
        let totalplugins: &usize = &cynplmn.len();
        let mut currentplugin: i32 = 1;
        for plugin in cynplmn {
            logger::general_info(format!(
                "Installing plugin {0}/{1}: {2}",
                currentplugin, totalplugins, plugin.id
            ));
            plugin_install(plugin.id, plugin.version);
            currentplugin += 1;
        }
    };
    process::exit(0);
}

pub(crate) fn plugin_install(wantedplugin: String, wantedpluginv: String) {
    let plugin_repo_url: &str = &format!(
        "https://raw.githubusercontent.com/CynthiaWebsiteEngine/Plugins/{}/index.json",
        crate::CYNTHIAPLUGINCOMPAT
    );
    if wantedplugin == *"none" {
        logger::general_error(String::from("No plugin selected."));
        process::exit(1);
    }
    logger::general_log(String::from("Creating temporary directories..."));
    let tempdir = Path::new("./.cynthiatemp/").join(format!(
        "{}_cyninsttemp",
        generate_rng(3..7, random_string::charsets::ALPHANUMERIC)
    ));
    let mut indexdownload = Vec::new();
    let mut c: Easy = Easy::new();
    match c.url(plugin_repo_url) {
        Ok(oki) => {
            logger::general_log(String::from("Downloading Cynthia Plugin Index..."));
            oki
        }
        Err(_) => {
            logger::general_error(String::from(
                "Could not start clean CynthiaConfig download!",
            ));
            process::exit(1);
        }
    };
    {
        let mut transfer = c.transfer();
        transfer
            .write_function(|new_data| {
                indexdownload.extend_from_slice(new_data);
                Ok(new_data.len())
            })
            .unwrap();
        match transfer.perform() {
            Ok(oki) => {
                logger::general_log(String::from("Download success."));

                oki
            }
            Err(_) => {
                logger::general_error(String::from("Could not download clean CynthiaConfig!"));
                process::exit(1);
            }
        }
    }
    let indexcontent = &indexdownload;
    fs::create_dir_all(tempdir.clone()).unwrap();
    let ctempdir = fs::canonicalize(tempdir.clone()).unwrap();
    let mut f = fs::File::create(ctempdir.join("./plugin_index.json")).unwrap();
    Write::write_all(&mut f, indexcontent).unwrap();

    let repositoryfile = ctempdir.join("./plugin_index.json");

    logger::general_log(String::from("Loading Cynthia Plugin Index..."));

    let mut o = fs::File::open(repositoryfile).expect("Could not read Cynthia Plugin Index.");
    let mut contents = String::new();
    o.read_to_string(&mut contents)
        .expect("Could not read Cynthia Plugin Index.");
    let unparsed: &str = contents.as_str();
    let cynplind: Vec<CynthiaPluginRepoItem> =
        serde_json::from_str(unparsed).expect("Could not read from Cynthia Plugin Index");
    logger::general_log(format!(
        "Searching Cynthia plugin index for '{wantedplugin}'..."
    ));
    let mut wantedpkg: &CynthiaPluginRepoItem = &CynthiaPluginRepoItem {
        id: "none".to_string(),
        host: "none".to_string(),
        referrer: "none".to_string(),
    };
    for cynplug in &cynplind {
        if cynplug.id == wantedplugin {
            logger::general_log(String::from("Found!").green().to_string());
            wantedpkg = cynplug;
            addtocynplmn(&wantedplugin, &wantedpluginv);
            break;
        }
        // println!("{:#?}", cynplug);
    }
    if wantedpkg.id.to_lowercase() == *"none" {
        logger::general_error(String::from("Not found!").red().to_string());
        process::exit(1);
    }
    let mut tarballurl = "unknown".to_string();
    if wantedpkg.host.to_lowercase() == *"npm" {
        println!(
            " --> Cynthia Plugin Index: {0} is on NPM as {1}!",
            wantedplugin, wantedpkg.referrer
        );
        logger::general_log(String::from("Asking NPM about this..."));
        let npmpackagename: String = format!("{1}@{0}", wantedpluginv, wantedpkg.referrer);
        let output: process::Output = match crate::jsr::jspm(false) {
            BUN_NPM => Command::new(BUN_NPM_EX)
                .arg("npm")
                .arg("view")
                .arg(npmpackagename)
                .arg("dist.tarball")
                .output()
                .expect("Could not call `bunx NPM view`."),
            NODE_NPM => Command::new(NODE_NPM)
                .arg("view")
                .arg(npmpackagename)
                .arg("dist.tarball")
                .output()
                .expect("Could not call NPM."),
            &_ => {
                logger::general_error(String::from(
                    "Something went wrong while contacting the Javascript package manager.",
                ));
                process::exit(1);
            }
        };

        tarballurl = format!("{}", String::from_utf8_lossy(&output.stdout));
        if output.status.success() {
            logger::general_info(format!("{} {}", "->".green(), tarballurl.blue()));
        } else {
            logger::general_error(String::from_utf8_lossy(&output.stderr).to_string());
        }
    } else if wantedpkg.host.to_lowercase() == "direct-tar" {
        println!("Skipping step 5... Archive is not hosted on NPM.");
        tarballurl = wantedpkg.referrer.to_owned();
    }
    if tarballurl == *"none" {
        print!("Error: Could not fetch tarball url for some reason.");
        process::exit(1);
    }
    let tarballfilepath = ctempdir.join(wantedplugin.clone());
    logger::general_log(format!(
        "Downloading {1} to '{0}'...",
        tarballfilepath.display(),
        wantedplugin
    ));
    let mut tarfiledownload = Vec::new();
    let mut curl: Easy = Easy::new();
    let safetarballurl = {
        encode(
            &tarballurl
                .replace("https://registry.npmjs.org/", "npmjsreg")
                .replace('\n', ""),
        )
        .replace("%2F", "/")
        .replace("npmjsreg", "https://registry.npmjs.org/")
    };

    match curl.url(&safetarballurl) {
        Ok(oki) => {
            logger::general_log(String::from("Downloading plugin archive..."));
            oki
        }
        Err(_) => {
            logger::general_error(String::from("Could not start clean plugin download!"));
            process::exit(1);
        }
    };
    {
        let mut transfer = curl.transfer();
        transfer
            .write_function(|new_data| {
                tarfiledownload.extend_from_slice(new_data);
                Ok(new_data.len())
            })
            .unwrap();
        transfer.perform().unwrap();
        // match transfer.perform() {
        //     Ok(oki) => {
        //         logger::general_log( String::from("Download success."));
        //
        //         oki
        //     }
        //     Err(_) => {
        //         logger::general_error( String::from("Could not download plugin!"));
        //         process::exit(1);
        //     }
        // }
    }
    let tarfilecontent = &tarfiledownload;
    let mut f = fs::File::create(tarballfilepath.clone()).unwrap();
    Write::write_all(&mut f, tarfilecontent).expect("Failed to write plugin.");
    logger::general_log(String::from("Download complete, starting unpack..."));
    let tar_gz = fs::File::open(&tarballfilepath).expect("Could not unpack plugin.");
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(&tempdir).expect("Could not unpack plugin.");
    let packagedir = ctempdir.join("./package");
    let mut options = fs_extra::dir::CopyOptions::new();
    options.overwrite = true;
    options.content_only = true;
    let pd = Path::new("./plugins/");
    let pdp = pd.join(wantedplugin);
    fs::create_dir_all(&pdp).expect("Could not create plugin folders.");
    fs_extra::dir::copy(packagedir, &pdp, &options).expect("Could not create target files.");
    logger::general_log(String::from("Cleaning temp files..."));
    remove_dir_all(tempdir).unwrap();
    logger::general_log(String::from("Installing dependencies for this plugin..."));
    let output = Command::new(crate::jsr::jspm(false))
        .arg("install")
        // Disabled as it fails to run on Bun
        // .arg("--production")
        .current_dir(pdp.clone())
        .output()
        .expect("Could not run the package manager.");
    if !output.status.success() {
        logger::general_error(format!(
            "Installing dependencies failed:\n\n\t{}",
            String::from_utf8_lossy(&output.stderr)
                .to_string()
                .replace('\n', "\n\t")
                .replace('\r', "\t")
        ));
    }
    logger::general_log(format!(
        "{} Installed to {}",
        "Done!".bright_green(),
        pdp.display()
    ));
}
fn getcynplmn() -> Result<Vec<CynthiaPluginManifestItem>, Error> {
    let pluginmanjson = Path::new("./cynthiapluginmanifest.json");
    return if pluginmanjson.exists() {
        logger::general_log(format!(
            "Installing plugins specified in '{0}' now...",
            pluginmanjson.display().to_string().blue()
        ));
        let mut o = fs::File::open(format!("{}", &pluginmanjson.display()).as_str())?;
        let mut contents = String::new();
        o.read_to_string(&mut contents)?;
        let unparsed: &str = contents.as_str();
        let cynplmn: Vec<CynthiaPluginManifestItem> = serde_json::from_str(unparsed)?;
        Ok(cynplmn)
    } else {
        logger::general_error(format!(
            "No Cynthia plugin manifest file found at '{0}'!",
            pluginmanjson.display().to_string().blue()
        ));
        Err(Error::from(ErrorKind::Other))
    };
}

fn removefromcynplmn(plugin_id: &String) {
    let wantedplugin: String = plugin_id.to_string();
    let cynplmns = getcynplmn().unwrap();
    let mut cynplmn: Vec<CynthiaPluginManifestItem> = vec![];
    for cynplug in cynplmns {
        if cynplug.id != wantedplugin {
            cynplmn.push(cynplug);
        }
    }
    let mut o = fs::File::create("./cynthiapluginmanifest.json").unwrap();
    let contents = serde_json::to_string(&cynplmn).unwrap();
    o.write_all(contents.as_bytes()).unwrap();
}

fn addtocynplmn(s_wantedplugin: &String, s_wantedpluginv: &String) {
    let wantedplugin: String = s_wantedplugin.to_string();
    let wantedpluginv: String = s_wantedpluginv.to_string();

    let mut cynplmn = getcynplmn().unwrap();
    let mut found = false;
    for cynplug in &mut cynplmn {
        if cynplug.id == wantedplugin {
            cynplug.version = wantedpluginv.to_string();
            found = true;
            break;
        }
    }
    if !found {
        cynplmn.push(CynthiaPluginManifestItem {
            id: wantedplugin.to_string(),
            version: wantedpluginv.to_string(),
        });
    }
    let mut o = fs::File::create("./cynthiapluginmanifest.json").unwrap();
    let contents = serde_json::to_string(&cynplmn).unwrap();
    o.write_all(contents.as_bytes()).unwrap();
}
fn choice(m: String, d: bool) -> bool {
    let mut result = d;
    let mut input = String::new();
    let mut waiting = true;
    while waiting {
        if d {
            logger::general_info(format!("{} (Y/n)?", m));
        } else {
            logger::general_info(format!("{} (y/N)?", m));
        };
        input.clear();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        if input == *"\r\n" {
            waiting = false;
        }
        input = input.replace(['\n', '\r'], "");
        if input.to_lowercase() == *"y" {
            waiting = false;
            result = true;
        } else if input.to_lowercase() == *"n" {
            result = false;
            waiting = false;
        }
    }
    println!();
    result
}

pub(crate) fn plugin_remove(plugin_id: String) {
    let pluginpath: PathBuf = Path::new("./plugins/").join(&plugin_id);
    if pluginpath.exists() {
        remove_dir_all(pluginpath).unwrap_or_default();
    }
    removefromcynplmn(&plugin_id);
    logger::general_info(format!("Plugin {} removed.", plugin_id));
}

pub(crate) fn install_from_plugin_manifest() {
    match getcynplmn() {
        Ok(cynplmn) => {
            let totalplugins: &usize = &cynplmn.len();
            let mut currentplugin: i32 = 1;
            for plugin in cynplmn {
                logger::general_info(format!(
                    "Installing plugin {0}/{1}: {2}",
                    currentplugin, totalplugins, plugin.id
                ));
                plugin_install(plugin.id, plugin.version);
                currentplugin += 1;
            }
        }
        Err(_) => {
            logger::general_error(String::from("Could not read Cynthia plugin manifest file!"));
            process::exit(1);
        }
    }
    process::exit(0);
}
