use crate::{
    jsr::{BUN_NPM, BUN_NPM_EX, NODE_NPM},
    logger,
    structs::CynthiaPluginRepoItem,
};
use colored::Colorize;
use curl::easy::Easy;
use flate2::read::GzDecoder;
use rand::Rng;
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
        rand::thread_rng().gen_range(10000000..999999999)
    ));
    let mut tarfiledownload = Vec::new();
    let mut c: Easy = Easy::new();
    match c.url(
        "https://codeload.github.com/CynthiaWebsiteEngine/cleanConfig/tar.gz/refs/heads/main",
    ) {
        Ok(oki) => {
            logger(1, String::from("Downloading clean CynthiaConfig..."));
            oki
        }
        Err(_) => {
            logger(
                5,
                String::from("Could not start clean CynthiaConfig download!"),
            );
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
                logger(1, String::from("Download success."));

                oki
            }
            Err(_) => {
                logger(5, String::from("Could not download clean CynthiaConfig!"));
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
    std::io::Write::write_all(&mut f, tarfilecontent).unwrap();
    let tar_gz = match fs::File::open(ctempdir.join("./cyn-clean.tar.gz")) {
        Ok(f) => f,
        Err(_) => {
            logger(5, String::from("Could not read clean CynthiaConfig!"));
            process::exit(1);
        }
    };
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    logger(
        1,
        format!(
            "Unpacking new CynthiaConfig to {}...",
            fs::canonicalize(Path::new("./"))
                .unwrap()
                .display()
                .to_string()
                .replace("\\\\?\\", "")
                .cyan()
        ),
    );
    match archive.unpack(tempdir) {
        Ok(f) => f,
        Err(_) => {
            logger(5, String::from("Could not unpack clean CynthiaConfig!"));
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
    fs::remove_dir_all(ctempdir).unwrap_or_default();
    let pluginmanjson = Path::new("./cynthiapluginmanifest.json");
    logger(
        10,
        String::from("Clean CynthiaConfig written! Please adjust then restart Cynthia!"),
    );
    if pluginmanjson.exists() {
        if choice(
            String::from("Do you want to install recomended plugins"),
            true,
        ) {
            logger(
                1,
                format!(
                    "Installing plugins specified in '{0}' now...",
                    pluginmanjson.display().to_string().blue()
                ),
            );
            let mut o = fs::File::open(format!("{}", &pluginmanjson.display()).as_str())
                .expect("Could not read Cynthia plugin manifest file.");
            let mut contents = String::new();
            o.read_to_string(&mut contents)
                .expect("Could not read Cynthia plugin manifest file.");
            let unparsed: &str = &contents.as_str();
            let cynplmn: Vec<crate::structs::CynthiaPluginManifestItem> =
                serde_json::from_str(unparsed)
                    .expect("Could not read from Cynthia plugin manifest file.");
            let totalplugins: &usize = &cynplmn.len();
            let mut currentplugin: i32 = 1;
            for plugin in cynplmn {
                logger(10, format!(
                    "Installing plugin {0}/{1}: {2}",
                    currentplugin, totalplugins, plugin.id
                ));
                plugin_install(plugin.id, plugin.version);
                currentplugin += 1;
            }
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
        logger(5, String::from("No plugin selected."));
        process::exit(1);
    }
    logger(1, String::from("Creating temporary directories..."));
    let tempdir = Path::new("./.cynthiatemp/").join(format!(
        "{}_cyninsttemp",
        rand::thread_rng().gen_range(10000000..999999999)
    ));
    let mut indexdownload = Vec::new();
    let mut c: Easy = Easy::new();
    match c.url(plugin_repo_url) {
        Ok(oki) => {
            logger(1, String::from("Downloading Cynthia Plugin Index..."));
            oki
        }
        Err(_) => {
            logger(
                5,
                String::from("Could not start clean CynthiaConfig download!"),
            );
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
                logger(1, String::from("Download success."));

                oki
            }
            Err(_) => {
                logger(5, String::from("Could not download clean CynthiaConfig!"));
                process::exit(1);
            }
        }
    }
    let indexcontent = &indexdownload;
    // Originally, I wanted to avoid downloading this, but Cargo doesn't do a great job at packaging extra files with it.
    // > let tarfilecontent = include_bytes!("../clean-cyn.tar.gz");
    // println!("lots of bytes: {:#?}", tarfilecontent);
    fs::create_dir_all(tempdir.clone()).unwrap();
    let ctempdir = fs::canonicalize(tempdir.clone()).unwrap();
    let mut f = fs::File::create(ctempdir.join("./plugin_index.json")).unwrap();
    std::io::Write::write_all(&mut f, indexcontent).unwrap();

    let repositoryfile = ctempdir.join("./plugin_index.json");

    logger(1, String::from("Loading Cynthia Plugin Index..."));

    let mut o = fs::File::open(&repositoryfile).expect("Could not read Cynthia Plugin Index.");
    let mut contents = String::new();
    o.read_to_string(&mut contents)
        .expect("Could not read Cynthia Plugin Index.");
    let unparsed: &str = contents.as_str();
    let cynplind: Vec<CynthiaPluginRepoItem> =
        serde_json::from_str(unparsed).expect("Could not read from Cynthia Plugin Index");
    logger(
        1,
        format!("Searching Cynthia plugin index for '{wantedplugin}'..."),
    );
    let mut wantedpkg: &CynthiaPluginRepoItem = &CynthiaPluginRepoItem {
        id: "none".to_string(),
        host: "none".to_string(),
        referrer: "none".to_string(),
    };
    for cynplug in &cynplind {
        if cynplug.id == wantedplugin {
            logger(1, String::from("Found!").green().to_string());

            wantedpkg = cynplug;
            break;
        }
        // println!("{:#?}", cynplug);
    }
    if wantedpkg.id.to_lowercase() == *"none" {
        logger(5, String::from("Not found!").red().to_string());
        process::exit(1);
    }
    let mut tarballurl = "unknown".to_string();
    if wantedpkg.host.to_lowercase() == *"npm" {
        println!(
            " --> Cynthia Plugin Index: {0} is on NPM as {1}!",
            wantedplugin, wantedpkg.referrer
        );
        logger(1, String::from("Asking NPM about this..."));
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
                logger(
                    5,
                    String::from(
                        "Something went wrong while contacting the Javascript package manager.",
                    ),
                );
                process::exit(1);
            }
        };

        tarballurl = format!("{}", String::from_utf8_lossy(&output.stdout));
        if output.status.success() {
            logger(10, format!("{} {}", "->".green(), tarballurl.blue()));
        } else {
            logger(5, String::from_utf8_lossy(&output.stderr).to_string());
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
    logger(
        1,
        format!(
            "Downloading {1} to '{0}'...",
            tarballfilepath.display(),
            wantedplugin
        ),
    );
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
            logger(1, String::from("Downloading plugin archive..."));
            oki
        }
        Err(_) => {
            logger(5, String::from("Could not start clean plugin download!"));
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
        //         logger(1, String::from("Download success."));
        //
        //         oki
        //     }
        //     Err(_) => {
        //         logger(5, String::from("Could not download plugin!"));
        //         process::exit(1);
        //     }
        // }
    }
    let tarfilecontent = &tarfiledownload;
    let mut f = fs::File::create(tarballfilepath.clone()).unwrap();
    std::io::Write::write_all(&mut f, tarfilecontent).expect("Failed to write plugin.");
    logger(1, String::from("Download complete, starting unpack..."));
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
    logger(1, String::from("Cleaning temp files..."));
    fs::remove_dir_all(tempdir).unwrap();
    logger(
        1,
        String::from("Installing dependencies for this plugin..."),
    );
    let output = Command::new(crate::jsr::jspm(false))
        .arg("install")
        // Disabled as it fails to run on Bun
        // .arg("--production")
        .current_dir(pdp.clone())
        .output()
        .expect("Could not run the package manager.");
    if !output.status.success() {
        logger(5, format!("Installing dependencies failed:\n\n\t{}", String::from_utf8_lossy(&output.stderr).to_string().replace("\n", "\n\t").replace("\r", "\t")));
    }
    logger(
        1,
        format!("{} Installed to {}", "Done!".bright_green(), pdp.display()),
    );
}

fn choice(m: String, d: bool) -> bool {
    let mut result = d;
    let mut input = String::new();
    let mut waiting = true;
    while waiting {
        if d == true {
        logger(10,format!("{} (Y/n)?", m));
    } else {
        logger(10,format!("{} (y/N)?", m));
    };
        input.clear();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        if input == *"\r\n" {
            waiting = false;
        }
        input = input.replace('\n', "").replace('\r', "");
        if input.to_lowercase() == *"y" {
            waiting = false;
            result = true;
        } else if input.to_lowercase() == *"n" {
            result = false;
            waiting = false;
        }
    }
    print!("\n");
    return result;
}
