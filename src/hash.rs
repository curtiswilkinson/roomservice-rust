use digest::Digest;
use ignore::Walk;
use meowhash::{MeowHash, MeowHasher};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub fn hash_dir(path: &Path) -> MeowHash {
    let mut hasher = MeowHasher::new();

    for maybe_file in Walk::new(&path) {
        let file = maybe_file.unwrap();
        match file.file_type() {
            Some(entry) => {
                if entry.is_file() {
                    let file = File::open(file.into_path()).unwrap();
                    let mut reader = BufReader::new(file);
                    loop {
                        let length = {
                            let buffer = reader.fill_buf().unwrap();
                            hasher.update(buffer);
                            buffer.len()
                        };
                        if length == 0 {
                            break;
                        }

                        reader.consume(length);
                    }
                }
            }
            None => (),
        }
    }

    hasher.finalise()
}

mod tests {
    use super::*;
    use test::Bencher;
    #[bench]
    fn bench_hash_dir(b: &mut Bencher) {
        b.iter(|| hash_dir(Path::new("../unity")))
    }
}
