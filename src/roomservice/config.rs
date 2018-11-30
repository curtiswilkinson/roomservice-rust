use std::collections::BTreeMap;
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug, PartialEq, Deserialize)]
pub struct Config {
    rooms: BTreeMap<String, RoomConfig>,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct RoomConfig {
    path: String,
    include: String,
    before: String,
    run_synchronous: String,
    run_parallel: String,
    after: String,
}

pub fn read() -> Config {
    let mut config_contents = String::new();
    let mut file = File::open("./roomservice.config.yml").expect("Unable to open config file");
    file.read_to_string(&mut config_contents)
        .expect("Error reading the config file");

    return serde_yaml::from_str(&config_contents).unwrap();
}
