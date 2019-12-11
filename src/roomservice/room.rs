use checksums::{hash_file, Algorithm::BLAKE2};
use ignore::Walk;
use std::fs::{self, File};
use std::io::prelude::*;
use util::fail;
use util::Failable;

#[derive(Debug, Clone, Copy)]
pub struct RoomBuilder<'a> {
    pub name: &'a str,
    pub path: &'a str,
    pub cache_dir: &'a str,
    pub include: &'a str,
    pub hooks: Hooks<'a>,
    pub should_build: bool,
    pub latest_hash: Option<&'a str>,
    pub errored: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct Hooks<'a> {
    pub before: Option<&'a str>,
    pub run_synchronously: Option<&'a str>,
    pub run_parallel: Option<&'a str>,
    pub after: Option<&'a str>,
    pub finally: Option<&'a str>,
}

impl RoomBuilder<'_> {
    pub fn new<'a>(
        name: &'a str,
        path: &'a str,
        cache_dir: &'a str,
        include: &'a str,
        hooks: Hooks<'a>,
    ) -> RoomBuilder<'a> {
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

    fn generate_hash(&self, dump_scope: bool) -> String {
        let mut hash = String::with_capacity(256);
        let mut scope = String::new();

        for maybe_file in Walk::new(&self.path) {
            let file = maybe_file.unwrap();
            if let Some(entry) = file.file_type() {
                if entry.is_file() {
                    if dump_scope {
                        scope.push_str(file.path().to_str().unwrap());
                        scope.push_str("\n")
                    }

                    hash.push_str(&hash_file(file.path(), BLAKE2));
                    hash.push_str("\n");
                }
            }
        }

        if dump_scope {
            fs::write(&self.name, scope).unwrap_fail("unable to dump file-scope");
        }

        hash
    }

    fn prev_hash(&self) -> Option<String> {
        let mut path = String::new();
        path.push_str(&self.cache_dir);
        path.push_str("/");
        path.push_str(&self.name);

        fs::read_to_string(path).ok()
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
        if file
            .write_all(self.latest_hash.as_ref().unwrap().as_bytes())
            .is_err()
        {
            fail("Unable to write roomservice cache for room {}")
        }
    }

    pub fn should_build(&mut self, force: bool, dump_scope: bool) {
        let prev = self.prev_hash();
        let curr = self.generate_hash(dump_scope);
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

        self.latest_hash = Some(&curr);
    }
}
