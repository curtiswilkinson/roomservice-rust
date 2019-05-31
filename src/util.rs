use colored::Colorize;

pub fn fail(message: &str) {
    println!("{} {}", "Error:".red().bold(), message);
    std::process::exit(1)
}

pub fn unwrap_fail<A>(target: Option<A>, message: &str) -> A {
    match target {
        Some(unwrapped) => unwrapped,
        None => {
            fail(message);
            unreachable!()
        }
    }
}
