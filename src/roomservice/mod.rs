use colored::Colorize;
use rayon::prelude::*;

pub mod config;
pub mod room;
use roomservice::room::RoomBuilder;
use std::path::Path;

#[derive(Debug)]
pub struct RoomserviceBuilder {
    pub rooms: Vec<room::RoomBuilder>,
    project: String,
    cache_dir: String,
    force: bool,
}

impl RoomserviceBuilder {
    pub fn new(project: String, cache_dir: String, force: bool) -> RoomserviceBuilder {
        println!("Project {}", project);

        match std::fs::create_dir(&cache_dir) {
            Ok(_) => (),
            Err(e) => match e.kind() {
                std::io::ErrorKind::AlreadyExists => (),
                _ => panic!("Unable to create `.roomservice` directory in project"),
            },
        };

        RoomserviceBuilder {
            project,
            force,
            cache_dir: cache_dir,
            rooms: Vec::new(),
        }
    }

    pub fn add_room(&mut self, mut room: room::RoomBuilder) {
        room.path = Path::new(&self.project)
            .join(room.path)
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        self.rooms.push(room);
    }

    pub fn exec(&mut self, update_hashes_only: bool) {
        if !update_hashes_only {
            println!("{}", "Diffing rooms".magenta().bold());
        } else {
            println!("{}", "Updating all rooms".magenta().bold())
        }

        let force = self.force;
        self.rooms
            .par_iter_mut()
            .for_each(|room| room.should_build(force));

        if !update_hashes_only {
            let mut is_before = false;
            let mut is_run_para = false;
            let mut is_run_sync = false;
            let mut is_after = false;

            let diff_names: Vec<_> = self
                .rooms
                .iter()
                .filter_map(|room| {
                    if room.hooks.before.is_some() {
                        is_before = true;
                    }

                    if room.hooks.run_parallel.is_some() {
                        is_run_para = true;
                    }

                    if room.hooks.run_synchronously.is_some() {
                        is_run_sync = true;
                    }

                    if room.hooks.after.is_some() {
                        is_after = true;
                    }

                    if room.should_build {
                        Some(format!("{} {}", "==>".bold(), &room.name))
                    } else {
                        None
                    }
                })
                .collect();

            if diff_names.is_empty() {
                println!("All rooms appear to be up to date!");
                return;
            }
            println!("The following rooms have changed:");
            println!("{}", diff_names.join("\n"));

            if is_before {
                println!("{}", "\nExecuting Before".magenta().bold());
                self.rooms
                    .par_iter()
                    .for_each(|room| exec_cmd(room, &room.hooks.before));
            }

            if is_run_para {
                println!("{}", "\nExecuting Run Parallel".magenta().bold());
                self.rooms
                    .par_iter()
                    .for_each(|room| exec_cmd(room, &room.hooks.run_parallel));
            }

            if is_run_sync {
                println!("{}", "\nExecuting Run Synchronously".magenta().bold());
                self.rooms
                    .iter()
                    .for_each(|room| exec_cmd(room, &room.hooks.run_synchronously));
            }

            if is_after {
                println!("{}", "\nExecuting After".magenta().bold());
                self.rooms
                    .par_iter()
                    .for_each(|room| exec_cmd(room, &room.hooks.after));
            }
        }

        for room in &self.rooms {
            room.write_hash();
        }
    }
}

fn exec_cmd(room: &RoomBuilder, cmd: &Option<String>) {
    use subprocess::{Exec, ExitStatus::Exited, Redirection};

    let should_build = room.should_build.to_owned();
    let cwd = room.path.to_owned();

    if should_build {
        match cmd {
            Some(cmd) => {
                println!("{} {} {}", "==>".bold(), "[Starting]".cyan(), room.name);
                match Exec::shell(cmd)
                    .cwd(cwd)
                    .stdout(Redirection::Pipe)
                    .stderr(Redirection::Pipe)
                    .capture()
                {
                    Ok(capture_data) => match capture_data.exit_status {
                        Exited(0) => {
                            println!("{} {} {}", "==>".bold(), "[Completed]".green(), room.name)
                        }
                        _ => {
                            println!("{} {} {}", "==>".bold(), "[Error]".red(), room.name);
                            println!("{}", capture_data.stderr_str());
                        }
                    },
                    _ => panic!("Unexpected error in exec_cmd"),
                }
            }
            None => (),
        }
    }
}
