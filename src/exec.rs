use crate::util::fail;

use colored::Colorize;

pub fn exec_cmd(cwd: &str, cmd: &str, name: &str) -> Result<(), ()> {
    use subprocess::{Exec, ExitStatus::Exited, Redirection};

    println!("{} {} {}", "==>".bold(), "[Starting]".cyan(), name);
    match Exec::shell(cmd)
        .cwd(cwd)
        .stdout(Redirection::Pipe)
        .stderr(Redirection::Pipe)
        .capture()
    {
        Ok(capture_data) => match capture_data.exit_status {
            Exited(0) => {
                println!("{} {} {}", "==>".bold(), "[Completed]".green(), name);
                Ok(())
            }
            _ => {
                println!("{} {} {}", "==>".bold(), "[Error]".red(), name);

                println!(
                    "{}\n{}",
                    capture_data.stdout_str(),
                    capture_data.stderr_str()
                );
                Err(())
            }
        },
        _ => Err(fail("Unexpected error in exec_cmd")),
    }
}
