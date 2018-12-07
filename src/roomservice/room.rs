use std::fs::File;
use std::io::prelude::*;

#[derive(Debug)]
pub struct RoomBuilder {
    pub name: String,
    pub path: String,
    pub include: String,
    pub hooks: Hooks,
    pub should_build: bool,
}

#[derive(Debug)]
pub struct Hooks {
    pub before: Option<String>,
    pub run_synchronously: Option<String>,
    pub run_parallel: Option<String>,
    pub after: Option<String>,
    pub finally: Option<String>,
}

impl RoomBuilder {
    pub fn new(name: String, path: String, include: String, hooks: Hooks) -> RoomBuilder {
        RoomBuilder {
            name,
            path,
            include,
            hooks,
            should_build: true,
        }
    }

    fn generate_hash(&self) -> String {
        use checksums::{hash_file, Algorithm::BLAKE2};

        let walker = globwalk::GlobWalkerBuilder::from_patterns(
            &self.path,
            &["*", "!target", "!.git", "!.roomservice"],
        ).follow_links(false)
        .build()
        .unwrap()
        .into_iter()
        .filter_map(|result| match result {
            Ok(entry) => if entry.file_type().is_file() {
                Some(entry)
            } else {
                None
            },
            Err(_) => None,
        });

        let mut hash = String::new();

        for file in walker {
            hash.push_str(&hash_file(file.path(), BLAKE2));
            hash.push_str("\n");
        }

        return hash;
    }

    fn prev_hash(&self) -> Option<String> {
        let mut path = String::new();
        path.push_str("./roomservice/");
        path.push_str(&self.name);
        let file = File::open(path);
        match file {
            Ok(mut handle) => {
                let mut contents = String::new();
                handle
                    .read_to_string(&mut contents)
                    .expect("should never fail");

                Some(contents)
            }
            Err(_) => None,
        }
    }

    pub fn write_hash(&self, hash: String) {
        let mut path = String::new();
        path.push_str("./.roomservice/");
        path.push_str(&self.name);
        let mut file = File::create(path).unwrap();
        match file.write_all(hash.as_bytes()) {
            Ok(_) => (),
            Err(e) => panic!("Unable to write roomservice cache for room {}", e),
        }
    }

    pub fn should_build(&mut self) {
        let prev = self.prev_hash();
        let curr = self.generate_hash();
        // println!("Current Hash is: {}, previous hash was: {:?}", curr, prev);

        match prev {
            Some(old_hash) => {
                if old_hash == curr {
                    self.should_build = false
                } else {
                    self.should_build = true;
                }
            }
            None => self.should_build = true,
        }

        if self.should_build {
            self.write_hash(curr)
        }
    }
}
