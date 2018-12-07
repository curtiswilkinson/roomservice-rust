#[macro_use]
extern crate serde_derive;
extern crate checksums;
extern crate clap;
extern crate colored;
extern crate glob;
extern crate globwalk;
extern crate rayon;
extern crate serde_yaml;
extern crate subprocess;

use clap::{App, Arg, SubCommand};

pub mod roomservice;
use roomservice::config;
use roomservice::room::{Hooks, RoomBuilder};
use roomservice::RoomserviceBuilder;

fn main() {
    let matches = App::new("Roomservice")
        .arg(
            Arg::with_name("project")
                .short("p")
                .long("project")
                .takes_value(true),
        ).subcommand(SubCommand::with_name("cache"))
        .get_matches();

    let project = matches.value_of("project").unwrap_or("./");

    println!("Project: {}", project);

    let mut roomservice = RoomserviceBuilder::new(project.to_string());

    let cfg = config::read(project);
    // println!("{:?}", cfg);

    for (name, room_config) in cfg.rooms {
        roomservice.add_room(RoomBuilder::new(
            name.to_string(),
            room_config.path.to_string(),
            room_config.include,
            Hooks {
                before: room_config.before,
                run_synchronously: room_config.run_synchronous,
                run_parallel: room_config.run_parallel,
                after: room_config.after,
                finally: room_config.finally,
            },
        ))
    }

    // roomservice.add_room(Room::new("./", None, "./**/*"));
    roomservice.exec();
    println!("{:?}", roomservice);

    // println!("{:?}", roomservice);
}
