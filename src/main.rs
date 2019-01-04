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

use clap::{App, Arg};

pub mod roomservice;
use roomservice::config;
use roomservice::room::{Hooks, RoomBuilder};
use roomservice::RoomserviceBuilder;

fn main() {
    use std::time::Instant;
    let start_time = Instant::now();
    let matches = App::new("Roomservice")
        .arg(
            Arg::with_name("project")
                .short("p")
                .long("project")
                .takes_value(true),
        )
        .arg(Arg::with_name("force").long("force").short("f"))
        .arg(
            Arg::with_name("only")
                .long("only")
                .takes_value(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("ignore")
                .long("ignore")
                .takes_value(true)
                .multiple(true),
        )
        .arg(Arg::with_name("update-hashes").long("update-hashes"))
        // Hooks
        .arg(Arg::with_name("no-after").long("no-after"))
        .get_matches();

    let project = matches.value_of("project").unwrap_or("./");
    let no_after = matches.is_present("no-after");
    let force = matches.is_present("force");

    let only: Vec<_> = match matches.values_of("only") {
        Some(only_values) => only_values.collect(),
        None => vec![],
    };

    let ignore: Vec<_> = match matches.values_of("ignore") {
        Some(ignore_values) => ignore_values.collect(),
        None => vec![],
    };

    if only.len() > 0 && ignore.len() > 0 {
        panic!("Some & None options provided, only one of these should be provided at a time")
    }

    let path_buf = std::path::Path::new(&project)
        .canonicalize()
        .unwrap()
        .join(".roomservice");

    let cache_dir = path_buf.to_str().unwrap().to_owned().to_string();

    let mut roomservice = RoomserviceBuilder::new(project.to_string(), cache_dir.clone(), force);

    let cfg = config::read(project);
    // println!("{:?}", cfg);

    for (name, room_config) in cfg.rooms {
        let mut should_add = true;

        // @Note Check to see if it's in the only array
        if only.len() > 0 {
            for only_name in &only {
                if only_name.to_string() != name {
                    should_add = false
                }
            }
        }

        // @Note Check to see if it's in the ignore array
        if ignore.len() > 0 {
            for ignore_name in &ignore {
                if ignore_name.to_string() == name {
                    should_add = false
                }
            }
        }

        if should_add {
            roomservice.add_room(RoomBuilder::new(
                name.to_string(),
                room_config.path.to_string(),
                cache_dir.clone(),
                room_config.include,
                Hooks {
                    before: room_config.before,
                    run_synchronously: room_config.run_synchronous,
                    run_parallel: room_config.run_parallel,
                    after: if no_after { None } else { room_config.after },
                    finally: room_config.finally,
                },
            ))
        }
    }

    let update_hashes_only = matches.is_present("update-hashes");
    roomservice.exec(update_hashes_only);

    println!("\nTime taken: {}s", start_time.elapsed().as_secs())
}
