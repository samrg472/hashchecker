#[macro_use]
mod util;
mod archive_org;

use clap::{App, AppSettings, Arg, SubCommand};
use std::{
    fs::File,
    io::{ErrorKind, Read, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

#[tokio::main]
async fn main() {
    let app =
        App::new("hashchecker")
            .setting(AppSettings::VersionlessSubcommands)
            .subcommand(
                SubCommand::with_name("archive.org")
                    .about("Verifies integrity of files downloaded from archive.org")
                    .arg(
                        Arg::with_name("metadata")
                            .required(true)
                            .long("metadata")
                            .takes_value(true)
                            .help("Path or URL to the archive.org files XML metadata"),
                    )
                    .arg(
                        Arg::with_name("rename_to_title")
                            .long("rename-to-title")
                            .help("Renames a file to the title specified in the metadata"),
                    )
                    .arg(Arg::with_name("path").long("path").takes_value(true).help(
                        "Path of the files to test, defaults to the current working directory",
                    )),
            )
            .subcommand(
                SubCommand::with_name("generate")
                    .about("Generates a checksum")
                    .arg(
                        Arg::with_name("algorithm")
                            .long("algorithm")
                            .short("a")
                            .default_value("1")
                            .takes_value(true)
                            .help("The checksum algorithm to generate"),
                    )
                    .arg(
                        Arg::with_name("file")
                            .required(true)
                            .takes_value(true)
                            .help("The path to the file to generate a checksum"),
                    ),
            );
    let matches = match app.get_matches_safe() {
        Ok(matches) => matches,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    let stdout = Arc::new(Mutex::new(StandardStream::stdout(ColorChoice::Auto)));

    if let Some(matches) = matches.subcommand_matches("archive.org") {
        let meta_loc = matches.value_of("metadata").unwrap();
        let test_path = match matches.value_of("path") {
            Some(path) => {
                let path = PathBuf::from(path);
                if !Path::is_dir(&path) {
                    panic!("Invalid path specified");
                }
                path
            }
            None => std::env::current_dir().unwrap(),
        };

        let conf = archive_org::ArchiveOrgConf {
            stdout,
            metadata_loc: meta_loc.into(),
            files_path: test_path,
        };

        conf.start().await;
    } else if let Some(matches) = matches.subcommand_matches("generate") {
        let file = matches.value_of("file").unwrap();
        let mut file = match File::open(&file) {
            Ok(file) => file,
            Err(e) => {
                let mut stdout = stdout.lock().unwrap();
                match e.kind() {
                    ErrorKind::NotFound => {
                        stdout
                            .set_color(ColorSpec::new().set_fg(Some(Color::Red)))
                            .unwrap();
                        writeln!(stdout, "No such file").unwrap();
                    }
                    _ => {
                        writeln!(stdout, "Failed to open file at with error {:?}", e).unwrap();
                    }
                }
                return;
            }
        };

        let digest = hash_file!(file);
        let digest = faster_hex::hex_string(digest.as_slice()).unwrap();
        println!("{}", digest);
    } else {
        println!("Nothing to do...exiting");
    }
}

#[derive(Clone, Default, Debug)]
pub struct Report {
    pub passed: usize,
    pub failed: usize,
    pub missing: usize,
}

impl Report {
    pub fn total(&self) -> usize {
        self.passed + self.failed + self.missing
    }
}

impl std::ops::Add for Report {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            passed: self.passed + rhs.passed,
            failed: self.failed + rhs.failed,
            missing: self.missing + rhs.missing,
        }
    }
}

impl std::ops::AddAssign for Report {
    fn add_assign(&mut self, rhs: Self) {
        self.passed += rhs.passed;
        self.failed += rhs.failed;
        self.missing += rhs.missing;
    }
}
