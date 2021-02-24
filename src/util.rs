use colored::Colorize;
use std::fmt::Display;

pub fn fail<T: Display>(message: T) {
    println!("{} {}", "Error:".red().bold(), message);
    std::process::exit(1)
}

pub trait Failable<T> {
    fn unwrap_fail(self, message: &str) -> T;
}

impl<T> Failable<T> for Option<T> {
    fn unwrap_fail(self, message: &str) -> T {
        match self {
            Some(unwrapped) => unwrapped,
            None => {
                fail(message);
                unreachable!()
            }
        }
    }
}

impl<T, E> Failable<T> for Result<T, E> {
    fn unwrap_fail(self, message: &str) -> T {
        match self {
            Ok(unwrapped) => unwrapped,
            Err(_) => {
                fail(message);
                unreachable!()
            }
        }
    }
}

#[macro_export]
macro_rules! para_hook {
    ($x: ident, $y: ident) => {
        if $y.iter().any(|(_, room)| room.$x.is_some()) {
            println!(
                "{}",
                format!("\nExecuting {}", stringify!($x)).magenta().bold()
            );
        }

        $y.par_iter().for_each(|(name, room)| {
            if let Some(hook) = &room.$x {
                exec::exec_cmd(&room.path, &hook, name).unwrap();
            }
        });
    };
}

#[macro_export]
macro_rules! sync_hook {
    ($x: ident, $y: ident) => {
        let mut ran = false;
        for (name, room) in &$y {
            if let Some(hook) = &room.$x {
                if !ran {
                    println!(
                        "{}",
                        format!("\nExecuting {}", stringify!($x)).magenta().bold()
                    );
                    ran = true;
                }
                exec::exec_cmd(&room.path, &hook, name).unwrap();
            }
        }
    };
}
