#![feature(test)]
use colored::Colorize;
use meowhash::MeowHash;
use rayon::prelude::*;
use std::path::Path;
use std::path::PathBuf;
use std::time::Instant;
use structopt::StructOpt;

mod config;
mod exec;
mod hash;
mod util;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate meowhash;
extern crate rayon;
extern crate test;

#[derive(StructOpt, Debug)]
#[structopt(name = "roomservice")]
struct Options {
    #[structopt(short, long, parse(from_os_str))]
    project: Option<PathBuf>,

    #[structopt(short, long)]
    force: bool,
    #[structopt(short, long)]
    dry: bool,

    #[structopt(short, long)]
    only: Option<String>,
    #[structopt(short, long)]
    ignore: Option<String>,

    #[structopt(long)]
    before: bool,
    #[structopt(long = "no-before")]
    no_before: bool,
    #[structopt(long)]
    after: bool,
    #[structopt(long = "no-after")]
    no_after: bool,
    #[structopt(long)]
    finally: bool,
    #[structopt(long = "no-finally")]
    no_finally: bool,
}

fn main() {
    use std::fs;
    let start_time = Instant::now();
    let opt = Options::from_args();

    let project_root = config::find_project(opt.project.clone()).unwrap();

    let config = config::read(&project_root.join(config::CONFIG_NAME));

    let cache_dir = project_root.join(".roomservice");
    // println!("{:?}", cache_dir);

    let rooms: Vec<_> = config
        .rooms
        .par_iter()
        .filter(|entry| {
            let cache_path = cache_dir.join("/").join(entry.0);
            match fs::read_to_string(&cache_path) {
                Ok(cache_content) => match MeowHash::from_slice(cache_content.as_bytes()) {
                    Some(prev) => {
                        let curr = hash::hash_dir(Path::new(&entry.1.path));

                        println!("curr: {:?}", curr);
                        println!("prev: {:?}", prev);

                        let should_build = curr != prev;

                        if should_build {
                            fs::write(cache_path, curr.into_bytes()).unwrap();
                        }

                        should_build
                    }
                    None => true,
                },
                Err(_) => true,
            }
        })
        .collect();

    para_hook!(before, rooms);
    sync_hook!(before_synchronous, rooms);
    sync_hook!(run_synchronous, rooms);

    // println!("{:?}", project_root);
    // println!("{:?}", config);
    // println!("\n{:?}", rooms);
    // println!("\n{:?}", opt);
    println!("\nTime taken: {}s", start_time.elapsed().as_secs())
}
