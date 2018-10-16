use rayon::prelude::*;

pub mod room;
use std::path::Path;

#[derive(Debug)]
pub struct RoomserviceBuilder {
    rooms: Vec<room::RoomBuilder>,
    project: String,
}

impl RoomserviceBuilder {
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
        self.rooms
            .par_iter_mut()
            .for_each(|room| room.should_build());

        // Check should builds
    }
}
