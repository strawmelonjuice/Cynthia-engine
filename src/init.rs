use crate::logger;
use colored::Colorize;
use curl::easy::Easy;
use flate2::read::GzDecoder;
use tar::Archive;

pub(crate) fn init() {
    let tempdir = std::path::Path::new("./.cynthiatemp/");
    let mut tarfiledownload = Vec::new();
    let mut c: Easy = Easy::new();
    match c.url(
        "https://codeload.github.com/strawmelonjuice/CynthiaCMS-cleanConfig/tar.gz/refs/heads/main",
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
            std::process::exit(1);
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
                std::process::exit(1);
            }
        }
    }
    let tarfilecontent = &tarfiledownload;
    // Originally, I wanted to avoid downloading this, but Cargo doesn't do a great job at packaging extra files with it.
    // > let tarfilecontent = include_bytes!("../clean-cyn.tar.gz");
    // println!("lots of bytes: {:#?}", tarfilecontent);
    std::fs::create_dir_all(tempdir).unwrap();
    let ctempdir = std::fs::canonicalize(tempdir.clone()).unwrap();
    let mut f = std::fs::File::create(ctempdir.join("./cyn-clean.tar.gz")).unwrap();
    std::io::Write::write_all(&mut f, tarfilecontent).unwrap();
    let tar_gz = match std::fs::File::open(ctempdir.join("./cyn-clean.tar.gz")) {
        Ok(f) => f,
        Err(_) => {
            logger(5, String::from("Could not read clean CynthiaConfig!"));
            std::process::exit(1);
        }
    };
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    logger(
        1,
        format!(
            "Unpacking new CynthiaConfig to {}...",
            std::fs::canonicalize(ctempdir.parent().unwrap())
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
            std::process::exit(1);
        }
    };
    // println!("{}", ctempdir.join("./CynthiaCMS-cleanConfig-main/").display());
    // std::fs::remove_file(ctempdir.join("/CynthiaCMS-cleanConfig-main").join("README.MD")).unwrap_or_default();
    let mut options = fs_extra::dir::CopyOptions::new();
    options.overwrite = true;
    options.content_only = true;
    fs_extra::dir::copy(
        ctempdir.join("./CynthiaCMS-cleanConfig-main/"),
        ctempdir.parent().unwrap(),
        &options,
    )
    .expect("Could not create target files.");
    std::fs::remove_dir_all(ctempdir).unwrap_or_default();
    logger(
        10,
        String::from("Clean CynthiaConfig written! Please adjust then restart Cynthia!"),
    );
    std::process::exit(0);
}
