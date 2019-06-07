use colored::Colorize;
use rayon::prelude::*;

pub mod config;
pub mod room;
use roomservice::room::RoomBuilder;
use std::path::Path;
use util::fail;

#[derive(Debug)]
pub struct RoomserviceBuilder {
    pub before_all: Option<String>,
    pub rooms: Vec<room::RoomBuilder>,
    pub after_all: Option<String>,
    project: String,
    cache_dir: String,
    force: bool,
}

impl RoomserviceBuilder {
    pub fn new(project: String, cache_dir: String, force: bool) -> RoomserviceBuilder {
        match std::fs::create_dir(&cache_dir) {
            Ok(_) => (),
            Err(e) => match e.kind() {
                std::io::ErrorKind::AlreadyExists => (),
                _ => fail("Unable to create `.roomservice` directory in project"),
            },
        };

        RoomserviceBuilder {
            project,
            force,
            cache_dir: cache_dir,
            rooms: Vec::new(),
            before_all: None,
            after_all: None,
        }
    }

    pub fn add_before_all(&mut self, before_all: String) {
        self.before_all = Some(before_all)
    }

    pub fn add_after_all(&mut self, after_all: String) {
        self.after_all = Some(after_all)
    }

    pub fn add_room(&mut self, mut room: room::RoomBuilder) {
        let room_path = Path::new(&self.project).join(&room.path);

        if room_path.exists() {
            room.path = room_path
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();

            self.rooms.push(room);
        } else {
            fail(
                format!(
                    "Path does not exist for room \"{}\" at \"{}\"",
                    room.name, room.path
                )
                .as_ref(),
            )
        }
    }

    pub fn exec(&mut self, update_hashes_only: bool, dry: bool, dump_scope: bool) {
        if !update_hashes_only {
            println!("{}", "Diffing rooms".magenta().bold());
        } else {
            println!("{}", "Updating all rooms".magenta().bold())
        }

        let force = self.force;
        self.rooms
            .par_iter_mut()
            .for_each(|room| room.should_build(force, dump_scope));

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
            if dry {
                return;
            }

            if self.before_all.is_some() {
                println!("{}", "\nExecuting Before All".magenta().bold());
                match exec_cmd(
                    "./".to_string(),
                    self.before_all.as_ref().unwrap().to_string(),
                    &"Before All".to_string(),
                ) {
                    Ok(_) => (),
                    Err(_) => fail("Error in Before All hook, aborting roomservice run"),
                }
            }

            if is_before {
                println!("{}", "\nExecuting Before".magenta().bold());
                self.rooms.par_iter_mut().for_each(|room| {
                    let hook = room.hooks.before.clone();
                    exec_room_cmd(room, hook);
                });
            }

            if is_run_para {
                println!("{}", "\nExecuting Run Parallel".magenta().bold());
                self.rooms.par_iter_mut().for_each(|room| {
                    let hook = room.hooks.run_parallel.clone();

                    exec_room_cmd(room, hook);
                });
            }

            if is_run_sync {
                println!("{}", "\nExecuting Run Synchronously".magenta().bold());
                self.rooms.iter_mut().for_each(|room| {
                    let hook = room.hooks.run_synchronously.clone();

                    exec_room_cmd(room, hook);
                });
            }
            if is_after {
                println!("{}", "\nExecuting After".magenta().bold());
                self.rooms.par_iter_mut().for_each(|room| {
                    let hook = room.hooks.after.clone();
                    exec_room_cmd(room, hook);
                });
            }

            if self.after_all.is_some() {
                println!("{}", "\nExecuting After All".magenta().bold());

                match exec_cmd(
                    "./".to_string(),
                    self.after_all.as_ref().unwrap().to_string(),
                    &"After All".to_string(),
                ) {
                    Ok(_) => (),
                    Err(_) => fail("Error in After All hook, aborting roomservice run"),
                }
            }
        }

        let mut was_error = false;
        for room in &self.rooms {
            if !room.errored {
                room.write_hash();
            } else {
                was_error = true
            }
        }

        if was_error {
            println!("\n{}", "Errors occured during roomservice".bold().red())
        }
    }
}

fn exec_room_cmd(room: &mut RoomBuilder, cmd: Option<String>) {
    let should_build = room.should_build.to_owned();
    let is_errored = room.errored;
    let cwd = room.path.to_owned();
    let name = &room.name;
    if should_build && !is_errored {
        match cmd {
            Some(cmd) => match exec_cmd(cwd, cmd, name) {
                Ok(_) => (),
                Err(_) => room.set_errored(),
            },
            None => (),
        }
    }
}

fn exec_cmd(cwd: String, cmd: String, name: &String) -> Result<(), ()> {
    use subprocess::{Exec, ExitStatus::Exited, Redirection};
    match Exec::shell(cmd)
        .cwd(cwd)
        .stdout(Redirection::Pipe)
        .stderr(Redirection::Pipe)
        .capture()
    {
        Ok(capture_data) => match capture_data.exit_status {
            Exited(0) => {
                println!("{} {} {}", "==>".bold(), "[Completed]".green(), name);
                Ok(())
            }
            _ => {
                println!("{} {} {}", "==>".bold(), "[Error]".red(), name);

                println!("{}", capture_data.stderr_str());
                Err(())
            }
        },
        _ => Err(fail("Unexpected error in exec_cmd")),
    }
}
