use glob::{glob_with, MatchOptions};
use rayon::prelude::*;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct RoomBuilder {
    pub name: String,
    pub path: String,
    pub include: String,
    pub before: Option<String>,
    should_build: bool,
}

impl RoomBuilder {
    pub fn new(path: &str, before: Option<&str>, include: &str) -> RoomBuilder {
        RoomBuilder {
            name: "NAME_PLACEHOLER".to_string(),
            path: path.to_string(),
            before: before.map(|b| b.to_string()),
            include: include.to_string(),
            should_build: true,
        }
    }
    pub fn should_build(&mut self) {
        self.should_build = true;

        let globpath =
            Path::new(&self.path).join(Path::new(&self.include).strip_prefix("./").unwrap());

        let options: MatchOptions = Default::default();
        let source_files: Vec<_> = glob_with(&globpath.to_str().unwrap(), &options)
            .unwrap()
            .filter_map(|x| x.ok())
            .collect();

        let files: Vec<_> = source_files
            .par_iter()
            .map(read_file)
            .filter_map(|x| match x {
                Some(x) => {
                    use sha2::{Digest, Sha256};
                    let mut hasher = Sha256::new();
                    hasher.input(x);
                    Some(hasher.result())
                }
                None => None,
            }).collect();

        println!("The number if files is {:?}", files.len());

        // hash(self, files)
    }
}

fn read_file(path: &PathBuf) -> Option<String> {
    use std::fs::metadata;
    match metadata(&path).unwrap().is_dir() {
        true => None,
        false => {
            let mut f = File::open(&path).expect("file not found");

            let mut contents = String::new();
            f.read_to_string(&mut contents).expect("err reading file");

            Some(contents)
        }
    }
}

fn hash(room: &mut RoomBuilder, files: Vec<String>) {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.input(&room.name);
    hasher.input(&room.path);
    hasher.input(&files.join("\n"));
    println!("{:?}", hasher.result());
}
