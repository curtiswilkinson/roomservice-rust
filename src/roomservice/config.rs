use std::collections::BTreeMap;
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug, PartialEq, Deserialize)]
pub struct Config {
    pub rooms: BTreeMap<String, RoomConfig>,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct RoomConfig {
    pub path: String,
    #[serde(default = "default_include")]
    pub include: String,
    pub before: Option<String>,
    #[serde(rename = "runSynchronous")]
    pub run_synchronous: Option<String>,
    #[serde(rename = "runParallel")]
    pub run_parallel: Option<String>,
    pub after: Option<String>,
    pub finally: Option<String>,
}

fn default_include() -> String {
    "./**/*.*".to_string()
}

pub fn read(path_to_project: &str) -> Config {
    let mut config_contents = String::new();
    let mut file = File::open([path_to_project, "roomservice.config.yml"].join("/"))
        .expect("Unable to open config file");
    file.read_to_string(&mut config_contents)
        .expect("Error reading the config file");

    serde_yaml::from_str(&config_contents).unwrap()
}
