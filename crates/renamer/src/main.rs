mod archive_org;

use clap::{App, AppSettings, Arg, SubCommand};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use termcolor::{ColorChoice, StandardStream};

#[tokio::main]
async fn main() {
    let app = App::new("renamer")
        .setting(AppSettings::VersionlessSubcommands)
        .subcommand(
            SubCommand::with_name("archive.org")
                .about("Batch rename archive.org files")
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
                        .help("Renames a file from source to the title")
                        .conflicts_with("rename_to_source"),
                )
                .arg(
                    Arg::with_name("rename_to_source")
                        .long("rename-to-source")
                        .help("Renames a file from title to the source")
                        .conflicts_with("rename_to_title"),
                )
                .arg(Arg::with_name("path").long("path").takes_value(true).help(
                    "Path of the files to rename, defaults to the current working directory",
                )),
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

        let to_title = {
            if matches.is_present("rename_to_title") {
                true
            } else if matches.is_present("rename_to_source") {
                false
            } else {
                println!("Nothing to do...exiting");
                return;
            }
        };

        conf.start(to_title).await;
    } else {
        println!("Nothing to do...exiting");
    }
}

#[derive(Clone, Default, Debug)]
pub struct Report {
    pub renamed: usize,
    pub untouched: usize,
    pub missing: usize,
}

impl Report {
    pub fn total(&self) -> usize {
        self.renamed + self.untouched + self.missing
    }
}

impl std::ops::Add for Report {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            renamed: self.renamed + rhs.renamed,
            untouched: self.untouched + rhs.untouched,
            missing: self.missing + rhs.missing,
        }
    }
}

impl std::ops::AddAssign for Report {
    fn add_assign(&mut self, rhs: Self) {
        self.renamed += rhs.renamed;
        self.untouched += rhs.untouched;
        self.missing += rhs.missing;
    }
}
