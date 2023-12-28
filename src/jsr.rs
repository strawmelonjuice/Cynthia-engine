use crate::logger::logger;

// Javascript runtimes:
//     NodeJS:
#[cfg(windows)]
pub const NODEJSR: &str = "node.exe";
#[cfg(not(windows))]
pub const NODEJSR: &'static str = "node";
//     Bun:
#[cfg(windows)]
pub const BUNJSR: &str = "bash.exe bun";
#[cfg(not(windows))]
pub const BUNJSR: &'static str = "bun";

// Javascript package managers:
//     NPM:
#[cfg(windows)]
pub const NODE_NPM: &str = "npm.cmd";
#[cfg(not(windows))]
pub const NODE_NPM: &str = "npm";
//     Bun:
#[cfg(windows)]
pub const BUN_NPM: &str = "bash.exe bun";
#[cfg(windows)]
pub const BUN_NPM_EX: &str = "bash.exe bunx";
#[cfg(not(windows))]
pub const BUN_NPM: &'static str = "bun";
#[cfg(not(windows))]
pub const BUN_NPM_EX: &'static str = "bunx";

//     NodeJS:
#[cfg(windows)]
pub const NODEJSR_EX: &str = "npx";
#[cfg(not(windows))]
pub const NODEJSR_EX: &'static str = "npx";
pub(crate) fn noderunner(args: Vec<&str>, cwd: std::path::PathBuf) -> String {
    if args[0] == "returndirect" {
        logger(1, String::from("Directreturn called on the JSR, this usually means something inside of Cynthia's Plugin Loader went wrong."));
        return args[1].to_string();
    }
    let output = match std::process::Command::new(jsruntime(false))
        .args(args.clone())
        .current_dir(cwd)
        .output()
    {
        Ok(result) => result,
        Err(_erro) => {
            logger(5, String::from("Couldn't launch Javascript runtime."));
            std::process::exit(1);
        }
    };
    if output.status.success() {
        return String::from_utf8_lossy(&output.stdout)
            .into_owned()
            .to_string();
    } else {
        println!("Script failed.");
        logger(12, String::from_utf8_lossy(&output.stderr).to_string());
    }
    String::from("")
}

pub(crate) fn jsruntime(mayfail: bool) -> &'static str {
    match std::process::Command::new(BUNJSR).arg("-v").output() {
        Ok(_t) => {
            return BUNJSR;
        }
        Err(_err) => {
            match std::process::Command::new(NODEJSR).arg("-v").output() {
                Ok(_t) => {
                    return NODEJSR;
                }
                Err(_err) => {
                    if !mayfail {
                        logger(
                            5,
                            String::from(
                                "No supported (Node.JS or Bun) Javascript runtimes found on path!",
                            ),
                        );
                        std::process::exit(1);
                    }
                    return "";
                }
            };
        }
    };
}
pub(crate) fn jspm(mayfail: bool) -> &'static str {
    return match std::process::Command::new(BUN_NPM).arg("-v").output() {
        Ok(_t) => BUN_NPM,
        Err(_err) => match std::process::Command::new(NODE_NPM).arg("-v").output() {
            Ok(_t) => NODE_NPM,
            Err(_err) => {
                if !mayfail {
                    logger(
                            5,
                            String::from(
                                "No supported (Node.JS or Bun) Javascript package managers found on path!",
                            ),
                        );
                    std::process::exit(1);
                }
                ""
            }
        },
    };
}
