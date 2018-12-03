#[macro_use]
extern crate serde_derive;
extern crate checksums;
extern crate colored;
extern crate glob;
extern crate rayon;
extern crate serde_yaml;
extern crate subprocess;

pub mod roomservice;
use roomservice::config;
use roomservice::room::{Hooks, RoomBuilder};
use roomservice::RoomserviceBuilder;

fn main() {
    let config = config::read();
    println!("{:?}", config);

    let mut roomservice = RoomserviceBuilder::new("./".to_string());

    roomservice.add_room(RoomBuilder::new(
        "room_one".to_string(),
        "./".to_string(),
        "./**/*.rs".to_string(),
        Hooks {
            before: Some("sleep 2".to_string()),
            run_synchronously: Some("sleep 4".to_string()),
            run_parallel: Some("sleep 1".to_string()),
            after: Some("sleep 2".to_string()),
            finally: None,
        },
    ));

    roomservice.add_room(RoomBuilder::new(
        "room_two".to_string(),
        "./".to_string(),
        "./**/*.rs".to_string(),
        Hooks {
            before: Some("eco HAHAHA".to_string()),
            run_synchronously: Some("sleep 3".to_string()),
            run_parallel: Some("sleep 2".to_string()),
            after: Some("sleep 4".to_string()),
            finally: None,
        },
    ));

    // roomservice.add_room(Room::new("./", None, "./**/*"));
    roomservice.exec();

    // println!("{:?}", roomservice);
}
