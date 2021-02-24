use crate::util::Failable;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Deserialize)]
pub struct Config {
    #[serde(rename = "beforeAll")]
    pub before_all: Option<String>,
    pub rooms: BTreeMap<String, RoomConfig>,
    #[serde(rename = "afterAll")]
    pub after_all: Option<String>,
}

fn default_include() -> String {
    "./**/*.*".into()
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct RoomConfig {
    pub path: String,
    #[serde(default = "default_include")]
    pub include: String,
    #[serde(rename = "beforeSynchronous")]
    pub before_synchronous: Option<String>,
    pub before: Option<String>,
    #[serde(rename = "runSynchronous")]
    pub run_synchronous: Option<String>,
    #[serde(rename = "runParallel")]
    pub run_parallel: Option<String>,
    pub after: Option<String>,
    pub finally: Option<String>,

    pub error: bool,
    pub new_hash: ,
}

impl RoomConfig {
    pub fn error(&mut self) {
        self.error = true
    }
}

pub const CONFIG_NAME: &'static str = "roomservice.config.yml";

pub fn find_project(maybe_base_or_file: Option<PathBuf>) -> Option<PathBuf> {
    match maybe_base_or_file {
        Some(base_or_file) => {
            if base_or_file.is_file() {
                base_or_file.as_path().parent().map(|x| x.to_path_buf())
            } else if base_or_file.is_dir() {
                Some(base_or_file)
            } else {
                None
            }
        }
        None => find_config("./").map(|x| x.as_path().parent().unwrap().to_path_buf()),
    }
}

pub fn find_config(base_path: &str) -> Option<PathBuf> {
    let path = Path::new(base_path);
    let maybe_config_path = Path::new(&path).join(CONFIG_NAME);

    if maybe_config_path.exists() {
        return Some(maybe_config_path);
    } else {
        let parent = maybe_config_path.parent()?;

        if Path::new(parent).exists() {
            let relative_path = if &base_path[..2] == "./" {
                Path::new("../").join(&base_path[2..])
            } else {
                Path::new("../").join(base_path)
            };

            find_config(relative_path.clone().to_str().unwrap())
        } else {
            None
        }
    }
}

pub fn read(path_to_project: &Path) -> Config {
    let mut config_contents = String::new();
    let mut file = File::open(path_to_project).unwrap_fail("Unable to open config");

    file.read_to_string(&mut config_contents)
        .expect("Error reading the config file");

    serde_yaml::from_str(&config_contents).unwrap()
}
