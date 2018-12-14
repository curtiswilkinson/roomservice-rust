use colored::Colorize;
use rayon::prelude::*;

pub mod config;
pub mod room;
use std::path::Path;

#[derive(Debug)]
pub struct RoomserviceBuilder {
    rooms: Vec<room::RoomBuilder>,
    project: String,
}

impl RoomserviceBuilder {
    // @CleanUp, this can be &str
    pub fn new(project: String) -> RoomserviceBuilder {
        match std::fs::create_dir(".roomservice") {
            Ok(_) => (),
            Err(e) => match e.kind() {
                std::io::ErrorKind::AlreadyExists => (),
                _ => panic!("Unable to create `.roomservice` directory in project"),
            },
        };

        RoomserviceBuilder {
            project,
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

    pub fn exec(&mut self) {
        println!("{}", "Diffing rooms".magenta().bold());
        self.rooms
            .par_iter_mut()
            .for_each(|room| room.should_build());

        let diff_names: Vec<_> = self
            .rooms
            .iter()
            .filter_map(|room| {
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

        println!("{}", "\nExecuting Before".magenta().bold());
        self.rooms.par_iter().for_each(|room| {
            exec_cmd(
                &room.name,
                room.should_build,
                &room.path,
                &room.hooks.before,
            )
        });

        println!("{}", "\nExecuting Run Parallel".magenta().bold());
        self.rooms.par_iter().for_each(|room| {
            exec_cmd(
                &room.name,
                room.should_build,
                &room.path,
                &room.hooks.run_parallel,
            )
        });

        println!("{}", "\nExecuting Run Synchronously".magenta().bold());
        self.rooms.iter().for_each(|room| {
            exec_cmd(
                &room.name,
                room.should_build,
                &room.path,
                &room.hooks.run_synchronously,
            )
        });

        println!("{}", "\nExecuting After".magenta().bold());
        self.rooms.par_iter().for_each(|room| {
            exec_cmd(&room.name, room.should_build, &room.path, &room.hooks.after)
        });

        for room in &self.rooms {
            if !room.errored {
                room.write_hash();
            }
        }
    }
}

fn exec_cmd(name: &str, should_build: bool, cwd: &str, cmd: &Option<String>) {
    use subprocess::{Exec, ExitStatus::Exited, Redirection};

    if should_build.to_owned() {
        match cmd {
            Some(cmd) => {
                println!("{} {} {}", "==>".bold(), "[Starting]".cyan(), name);
                match Exec::shell(cmd)
                    .cwd(cwd)
                    .stdout(Redirection::Pipe)
                    .popen()
                    .unwrap()
                    .wait()
                {
                    Ok(e) => match e {
                        Exited(0) => {
                            println!("{} {} {}", "==>".bold(), "[Completed]".green(), name)
                        }
                        e => {
                            println!("{} {} {}", "==>".bold(), "[Error]".red(), name);
                            println!("{:?}", e)
                        }
                    },
                    _ => panic!("Unexpected stuff"),
                }
            }
            None => (),
        }
    }
}
