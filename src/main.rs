#![feature(test)]
#[macro_use]
extern crate serde_derive;
extern crate clap;
extern crate colored;
extern crate ignore;
extern crate meowhash;
extern crate rayon;
extern crate serde_yaml;
extern crate subprocess;
extern crate test;

use clap::{App, Arg};

pub mod roomservice;
pub mod util;

use std::collections::BTreeMap;

use roomservice::config::{self, RoomConfig};
use roomservice::room::{Hooks, RoomBuilder};
use roomservice::RoomserviceBuilder;

use std::path::Path;
use util::{fail, Failable};

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
        .arg(Arg::with_name("after").long("after"))
        .arg(Arg::with_name("dry").long("dry").short("d"))
        .arg(Arg::with_name("dump-scope").long("dump-scope"))
        .arg(Arg::with_name("update-hashes").long("update-hashes"))
        // Hooks
        .arg(Arg::with_name("no-after").long("no-after"))
        .get_matches();

    let project = matches.value_of("project");
    let no_after = matches.is_present("no-after");
    let force = matches.is_present("force");
    let after = matches.is_present("after");

    let only = split_matches(matches.values_of("only"));
    let ignore = split_matches(matches.values_of("ignore"));

    if only.len() > 0 && ignore.len() > 0 {
        fail("--only & --ignore options provided, only one of these should be provided at a time")
    }

    if after && no_after {
        fail("Both --after & --no-after options provided.")
    }

    let project_path = find_project(project).unwrap_fail("No config found.");

    let path_buf = project_path.join(".roomservice");

    let cache_dir = path_buf.to_str().unwrap().to_owned().to_string();

    let mut roomservice = RoomserviceBuilder::new(
        project_path.to_str().unwrap().to_string(),
        cache_dir.clone(),
        force,
    );

    let cfg = if Path::new(project.unwrap()).is_file() {
        config::read(&project.unwrap())
    } else {
        config::read(
            project_path
                .join("roomservice.config.yml")
                .to_str()
                .unwrap(),
        )
    };

    if cfg.before_all.is_some() {
        roomservice.add_before_all(&cfg.before_all.unwrap())
    }

    if cfg.after_all.is_some() {
        roomservice.add_after_all(&cfg.after_all.unwrap())
    }

    check_room_provided_to_flag("only", &only, &cfg.rooms);

    check_room_provided_to_flag("ignore", &ignore, &cfg.rooms);

    for (name, room_config) in cfg.rooms {
        let mut should_add = true;

        // @Note Check to see if it's in the only array
        if only.len() > 0 {
            if only.contains(&name.as_str()) {
                should_add = true
            } else {
                should_add = false
            }
        }

        // @Note Check to see if it's in the ignore array
        if ignore.len() > 0 {
            if ignore.contains(&name.as_str()) {
                should_add = false
            } else {
                should_add = true
            }
        }

        if should_add {
            roomservice.add_room(RoomBuilder::new(
                name.to_string(),
                room_config.path.to_string(),
                cache_dir.clone(),
                room_config.include,
                Hooks {
                    before: if after { None } else { room_config.before },
                    run_synchronously: if after {
                        None
                    } else {
                        room_config.run_synchronous
                    },
                    run_parallel: if after {
                        None
                    } else {
                        room_config.run_parallel
                    },
                    after: if no_after { None } else { room_config.after },
                    finally: if after { None } else { room_config.finally },
                },
            ))
        }
    }

    let update_hashes_only = matches.is_present("update-hashes");
    let dry = matches.is_present("dry");
    let dump_scope = matches.is_present("dump-scope");

    roomservice.exec(update_hashes_only, dry, dump_scope);

    println!("\nTime taken: {}s", start_time.elapsed().as_secs())
}

fn find_project(maybe_base_or_file: Option<&str>) -> Option<&Path> {
    match maybe_base_or_file {
        Some(base_or_file_str) => {
            let base_or_file = Path::new(base_or_file_str);
            if base_or_file.is_file() {
                base_or_file.parent()
            } else if base_or_file.is_dir() {
                Some(base_or_file)
            } else {
                None
            }
        }
        None => find_config("./"),
    }
}

fn find_config(base_path: &str) -> Option<&Path> {
    let path = Path::new(base_path);
    let maybe_config_path = Path::new(&path).join("roomservice.config.yml");

    if maybe_config_path.exists() {
        return Some(&maybe_config_path);
    } else {
        let parent = maybe_config_path.parent()?;

        if Path::new(parent).exists() {
            let relative_path = if &base_path[..2] == "./" {
                Path::new("../").join(&base_path[2..])
            } else {
                Path::new("../").join(base_path)
            };

            find_config(relative_path.to_str().unwrap())
        } else {
            None
        }
    }
}

fn check_room_provided_to_flag(
    flag: &str,
    provided_to_flag: &Vec<&str>,
    rooms: &BTreeMap<String, RoomConfig>,
) {
    if provided_to_flag.len() > 0 {
        for name in provided_to_flag {
            if !rooms.keys().any(|room_name| room_name == name) {
                fail(format!(
                    "\"{}\" was provided to --{} and does not exist in config",
                    name, flag
                ))
            }
        }
    }
}

fn split_matches<'a>(val: Option<clap::Values<'a>>) -> Vec<&'a str> {
    match val {
        Some(ignore_values) => {
            let values: Vec<_> = ignore_values.collect();

            values[0].split(',').collect()
        }

        None => vec![],
    }
}

#[cfg(test)]
mod tests {
    use test::Bencher;

    use crate::roomservice::room::{Hooks, RoomBuilder};
    #[bench]
    fn hash_dir(b: &mut Bencher) {
        let room = RoomBuilder::new(
            "Test".into(),
            "../unity/services/core-portal/apps".into(),
            "".into(),
            "".into(),
            Hooks {
                before: None,
                run_synchronously: None,
                run_parallel: None,
                after: None,
                finally: None,
            },
        );

        b.iter(|| room.generate_hash(false))
    }
}
