mod debugger;

use std::ptr::null;

use nix::{
    libc::execl,
    sys::ptrace,
    unistd::{fork, ForkResult},
};

use crate::debugger::Debugger;

#[derive(Debug)]
pub enum HostError {
    ProcessNotFound(String),
    NixError(nix::errno::Errno),
}

impl std::error::Error for HostError {}

impl std::fmt::Display for HostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HostError::ProcessNotFound(e) => write!(f, "Process {} not found", e),
            HostError::NixError(e) => write!(f, "Nix error {}", e),
        }
    }
}

impl From<nix::errno::Errno> for HostError {
    fn from(value: nix::errno::Errno) -> Self {
        HostError::NixError(value)
    }
}

fn main() -> Result<(), HostError> {
    pretty_env_logger::init();

    log::info!("My Pid is {}", std::process::id());
    let process_name = "victim";

    match unsafe { fork() } {
        Ok(ForkResult::Child) => {
            ptrace::traceme()?;
            unsafe { execl(process_name.as_ptr() as *const i8, null()) };
        }
        Ok(ForkResult::Parent { child, .. }) => {
            log::info!("Started debugging process {}", child);
            let debugger = Debugger::new(process_name.to_string(), child);
            debugger.run()?;
        }
        Err(e) => return Err(HostError::NixError(e)),
    };

    Ok(())
}
