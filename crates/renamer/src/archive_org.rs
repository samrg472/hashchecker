use crate::Report;
use rru_common::XmlDoc;
use std::{
    fs,
    io::{Read, Write},
    mem,
    path::{Path, PathBuf},
    sync::Arc,
    sync::Mutex,
};
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

pub struct ArchiveOrgConf {
    pub stdout: Arc<Mutex<StandardStream>>,
    pub metadata_loc: String,
    pub files_path: PathBuf,
}

impl ArchiveOrgConf {
    pub async fn start(&self, to_title: bool) {
        let xml_doc = retrieve_xml(&self.metadata_loc).await;
        let metadata = XmlDoc::parse(&xml_doc);

        assert_eq!(
            metadata.name(),
            "files",
            "invalid archive.org xml root element"
        );

        let mut latest_report = Report::default();

        for file in metadata.children() {
            assert_eq!(file.name(), "file", "expected file element");
            let format = file.get_child("format").unwrap();
            if format.value().unwrap() == "Metadata"
                || file.get_attrib("source").unwrap() == "metadata"
            {
                // Ignore metadata generated by archive.org as they have no relevancy during
                // checks
                continue;
            }

            let mut old_name = file.get_attrib("name").unwrap();
            let mut old_path = Path::join(&self.files_path, old_name);

            let mut new_name = match file.get_child("title") {
                Some(new_name) => new_name.value().unwrap(),
                None => {
                    latest_report += self.rename(&old_path, None);
                    continue;
                }
            };

            let mut new_path = Path::join(&self.files_path, new_name);
            new_path.set_extension(old_path.extension().unwrap());

            if !to_title {
                mem::swap(&mut old_name, &mut new_name);
                mem::swap(&mut old_path, &mut new_path);
            }

            let report = self.rename(&old_path, Some(&new_path));
            latest_report += report;
        }

        let mut stdout = self.stdout.lock().unwrap();
        write!(&mut stdout, "\n").unwrap();
        writeln!(&mut stdout, "SUMMARY:").unwrap();
        writeln!(&mut stdout, "    Renamed files: {}", latest_report.renamed).unwrap();
        writeln!(
            &mut stdout,
            "    Untouched files: {}",
            latest_report.untouched
        )
        .unwrap();
        writeln!(&mut stdout, "    Missing files: {}", latest_report.missing).unwrap();
        writeln!(&mut stdout, "    Total checks: {}", latest_report.total()).unwrap();
    }

    fn rename(&self, old_path: &Path, new_path: Option<&Path>) -> Report {
        let mut report = Report::default();

        let old_name = pretty_path_name(old_path);
        match new_path {
            Some(new_path) => {
                let new_name = pretty_path_name(new_path);
                if old_path.is_file() {
                    if new_path.is_file() {
                        panic!(
                            "Cannot rename {} as the new name already exists at {}",
                            old_name, new_name
                        );
                    }

                    fs::rename(old_path, new_path).unwrap();
                    report.renamed += 1;
                    self.write_quick_report(Color::Green, "RENAMED", old_name, Some(&new_name));
                } else if new_path.is_file() {
                    report.untouched += 1;
                    self.write_quick_report(Color::Green, "UNTOUCHED", new_name, None);
                } else {
                    report.missing += 1;
                    self.write_quick_report(Color::Red, "MISSING", old_name, None);
                }
            }
            None => {
                if old_path.is_file() {
                    report.untouched += 1;
                    self.write_quick_report(Color::Green, "UNTOUCHED", old_name, None);
                } else {
                    report.missing += 1;
                    self.write_quick_report(Color::Red, "MISSING", old_name, None);
                }
            }
        }

        report
    }

    fn write_quick_report(&self, status_color: Color, status: &str, old: &str, new: Option<&str>) {
        let mut stdout = self.stdout.lock().unwrap();
        stdout
            .set_color(ColorSpec::new().set_fg(Some(status_color)))
            .unwrap();
        write!(stdout, "{}", status).unwrap();
        stdout.reset().unwrap();
        match new {
            Some(new) => {
                writeln!(stdout, " ... '{}' => '{}'", old, new).unwrap();
            }
            None => writeln!(stdout, " ... '{}'", old).unwrap(),
        }
    }
}

fn pretty_path_name(p: &Path) -> &str {
    p.file_name().unwrap().to_str().unwrap()
}

async fn retrieve_xml(loc: &str) -> String {
    if Path::is_file(&PathBuf::from(loc)) {
        let mut file = std::fs::File::open(loc).expect("Failed to open metadata file");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Failed to read XML");
        return contents;
    }

    let res = reqwest::get(loc).await.unwrap();
    res.text().await.unwrap()
}
