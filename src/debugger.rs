use nix::{sys::ptrace, sys::wait::waitpid, unistd::Pid};
use std::{
    collections::HashMap,
    io::{self, Write},
};

use crate::{breakpoint::BreakPoint, HostError, HostResult};

#[derive(Debug)]
pub struct Debugger {
    prog_name: String,
    pid: Pid,
    breakpoints: HashMap<u64, BreakPoint>,
}

impl Debugger {
    pub fn new(prog_name: String, pid: Pid) -> Self {
        Debugger {
            prog_name,
            pid,
            breakpoints: HashMap::new(),
        }
    }

    pub fn run(&mut self) -> HostResult<()> {
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

            self.handle_command(command)?;
        }

        Ok(())
    }

    fn handle_command(&mut self, command: &str) -> HostResult<()> {
        let mut args = command.split(' ');
        let command = args.next().unwrap();

        match command {
            "continue" => self.continue_execution().unwrap(),
            "break" => {
                // user input is in hex format. Ex: break 0x114c
                let address = args.next().unwrap().strip_prefix("0x").unwrap();
                let address =
                    i64::from_str_radix(address, 16).map_err(|e| HostError::ParseError(e))?;
                self.set_breakpoint_at_address(address.try_into().unwrap())?;
            }
            _ => log::error!("Unknown command"),
        }

        Ok(())
    }

    fn continue_execution(&self) -> HostResult<()> {
        // continue execution
        ptrace::cont(self.pid, None)?;

        //wait for signal
        waitpid(self.pid, None)?;

        Ok(())
    }

    fn set_breakpoint_at_address(&mut self, address: u64) -> HostResult<()> {
        log::info!("Set breakpoint at address 0x{:x}", address);
        let mut bp = BreakPoint::new(self.pid, address);

        match bp.enable() {
            Ok(_) => {
                println!("Breakpoint set: {:}", bp);
            }
            Err(err) => {
                println!("{}", err);
            }
        }
        self.breakpoints.insert(address, bp);

        Ok(())
    }
}
