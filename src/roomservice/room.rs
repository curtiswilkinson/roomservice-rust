use rayon::prelude::*;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

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
        use glob::{glob_with, MatchOptions};

        let globpath =
            Path::new(&self.path).join(Path::new(&self.include).strip_prefix("./").unwrap());

        let options: MatchOptions = Default::default();
        let source_files: Vec<_> = glob_with(&globpath.to_str().unwrap(), &options)
            .unwrap()
            .filter_map(|x| match x {
                Ok(path) => {
                    use std::fs::metadata;
                    match metadata(&path) {
                        Ok(meta) => {
                            if meta.is_file() {
                                Some(path)
                            } else {
                                None
                            }
                        }
                        Err(_) => None,
                    }
                }
                Err(_) => None,
            })
            .collect();

        let hashed_files: Vec<_> = source_files
            .par_iter()
            .map(|path| hash_file(&path, BLAKE2))
            .collect();

        hashed_files.join("\n")
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
