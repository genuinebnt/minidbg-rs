mod breakpoint;
mod debugger;

use std::{env, num::ParseIntError, ptr::null};

use nix::{
    libc::execl,
    sys::{
        personality::{self, Persona},
        ptrace,
    },
    unistd::{fork, ForkResult},
};

use crate::debugger::Debugger;

#[derive(Debug)]
pub enum HostError {
    ProcessNotFound(String),
    NixError(nix::errno::Errno),
    ParseError(ParseIntError),
}

impl std::error::Error for HostError {}

impl std::fmt::Display for HostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HostError::ProcessNotFound(e) => write!(f, "Process {} not found", e),
            HostError::NixError(e) => write!(f, "Nix error {}", e),
            HostError::ParseError(e) => write!(f, "Parse Error {}", e),
        }
    }
}

impl From<nix::errno::Errno> for HostError {
    fn from(value: nix::errno::Errno) -> Self {
        HostError::NixError(value)
    }
}

pub type HostResult<T> = std::result::Result<T, HostError>;

fn main() -> HostResult<()> {
    pretty_env_logger::init();

    let args: Vec<String> = env::args().collect();

    // first argument is the program to debug
    if args.len() < 2 {
        eprintln!("Usage: {} <your_argument>", args[0]);
        std::process::exit(1);
    }

    log::info!("My Pid is {}", std::process::id());
    let process_name = &args[1];

    // fork create 2 processes
    match unsafe { fork() } {
        Ok(ForkResult::Child) => {
            // disable address space randomization
            personality::set(Persona::ADDR_NO_RANDOMIZE)?;
            // enable child process to be traced
            ptrace::traceme()?;
            // execute child process
            unsafe { execl(process_name.as_ptr() as *const i8, null()) };
        }
        Ok(ForkResult::Parent { child, .. }) => {
            println!("Started debugging process with Pid {}", child);
            let mut debugger = Debugger::new(process_name.to_string(), child);
            debugger.run()?;
        }
        Err(e) => return Err(HostError::NixError(e)),
    };

    Ok(())
}
