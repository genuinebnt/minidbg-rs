use std::io::{self, Write};

use nix::{sys::ptrace, sys::wait::waitpid, unistd::Pid};

use crate::HostError;

#[derive(Debug)]
pub struct Debugger {
    prog_name: String,
    pid: Pid,
}

impl Debugger {
    pub fn new(prog_name: String, pid: Pid) -> Self {
        Debugger { prog_name, pid }
    }

    pub fn run(&self) -> Result<(), HostError> {
        waitpid(self.pid, None)?;

        loop {
            print!("minidbg> ");
            std::io::stdout().flush().expect("Failed to flush stdout");

            let mut command = String::new();
            io::stdin()
                .read_line(&mut command)
                .expect("Failed to read line");

            let command = command.trim();
            if command == "exit" {
                break;
            }

            self.handle_command(command);
        }

        Ok(())
    }

    fn handle_command(&self, command: &str) {
        let mut args = command.split(' ');
        let command = args.next().unwrap();

        if command == "continue" {
            self.continue_execution().unwrap();
        }
    }

    fn continue_execution(&self) -> Result<(), HostError> {
        ptrace::cont(self.pid, None)?;

        waitpid(self.pid, None)?;

        Ok(())
    }
}
