use std::fs::File;
use std::io::prelude::*;

#[derive(Debug)]
pub struct RoomBuilder {
    pub name: String,
    pub path: String,
    pub cache_dir: String,
    pub include: String,
    pub hooks: Hooks,
    pub should_build: bool,
    pub latest_hash: Option<String>,
    pub errored: bool,
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
    pub fn new(
        name: String,
        path: String,
        cache_dir: String,
        include: String,
        hooks: Hooks,
    ) -> RoomBuilder {
        RoomBuilder {
            name,
            path,
            cache_dir,
            include,
            hooks,
            errored: false,
            should_build: true,
            latest_hash: None,
        }
    }

    fn generate_hash(&self) -> String {
        use checksums::{hash_file, Algorithm::BLAKE2};

        let walker = globwalk::GlobWalkerBuilder::from_patterns(
            &self.path,
            &["*", "!target", "!.git", "!.roomservice", "!node_modules"],
        )
        .follow_links(false)
        .build()
        .unwrap()
        .filter_map(|result| match result {
            Ok(entry) => {
                if entry.file_type().is_file() {
                    Some(entry)
                } else {
                    None
                }
            }
            Err(_) => None,
        });

        let mut hash = String::new();

        for file in walker {
            hash.push_str(&hash_file(file.path(), BLAKE2));
            hash.push_str("\n");
        }

        hash
    }

    fn prev_hash(&self) -> Option<String> {
        let mut path = String::new();
        path.push_str(&self.cache_dir);
        path.push_str("/");
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

    pub fn set_errored(&mut self) {
        self.errored = true
    }

    pub fn write_hash(&self) {
        let mut path = String::new();
        path.push_str(&self.cache_dir);
        path.push_str("/");
        path.push_str(&self.name);
        let mut file = File::create(path).unwrap();
        match file.write_all(self.latest_hash.as_ref().unwrap().as_bytes()) {
            Ok(_) => (),
            Err(e) => panic!("Unable to write roomservice cache for room {}", e),
        }
    }

    pub fn should_build(&mut self, force: bool) {
        let prev = self.prev_hash();
        let curr = self.generate_hash();
        // println!("Current Hash is: {}, previous hash was: {:?}", curr, prev);
        if force {
            self.should_build = true
        } else {
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
        }

        self.latest_hash = Some(curr);
    }
}
