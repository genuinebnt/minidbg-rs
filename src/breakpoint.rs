use std::{ffi::c_void, fmt::Display};

use nix::{sys::ptrace, unistd::Pid};

use crate::HostResult;

#[derive(Debug, Clone)]
pub struct BreakPoint {
    pid: Pid,
    address: u64,
    enabled: bool,
    original_instr: i64,
}

impl BreakPoint {
    pub fn new(pid: Pid, address: u64) -> Self {
        BreakPoint {
            pid,
            address,
            enabled: false,
            original_instr: i64::default(),
        }
    }

    pub fn enable(&mut self) -> HostResult<()> {
        // read calls ptrace with peekdata
        self.original_instr = ptrace::read(self.pid, self.address as *mut c_void)?;
        // save lower 8 bits so we can restore it later

        // overwrite lower 8 bits to 0xcc to set software breakpoint
        let data = self.original_instr & (i64::MAX ^ 0xFF) | 0xCC;

        //write calls ptrace with pokedata
        unsafe {
            ptrace::write(self.pid, self.address as *mut c_void, data as *mut c_void).unwrap()
        };

        self.enabled = true;

        eprintln!("{:}", self);

        Ok(())
    }

    pub fn disable(&mut self) -> HostResult<()> {
        unsafe {
            ptrace::write(
                self.pid,
                self.address as *mut c_void,
                self.original_instr as *mut c_void,
            )
            .unwrap();
        }

        self.enabled = false;

        Ok(())
    }
}

impl Display for BreakPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Bp Address 0x{:x}, PID: {}, Enabled: {}, Original: 0x{:x}",
            self.address, self.pid, self.enabled, self.original_instr
        )
    }
}
