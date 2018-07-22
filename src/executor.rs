extern crate nix;

use executor::nix::sys::wait::waitpid;
use executor::nix::unistd::*;
use lexer::get_function_from_string;
use lexer::{Command, Function};

pub enum ToQuit {
    Quit,
    Continue,
}

pub fn execute_all_functions(functions: Vec<Function>) -> nix::Result<ToQuit> {
    let mut quit_after_execution = false;
    let mut children: Vec<Pid> = Vec::new();

    for function in &functions {
        match function {
            Function::Quit => quit_after_execution = true,
            Function::ShellCommand(command) => {
                let fork_result = fork()?;
                match fork_result {
                    ForkResult::Parent { child } => {
                        children.push(child);
                    }
                    ForkResult::Child => {
                        command.execute();
                    }
                }
            }
        }
    }
    for child in &children {
        waitpid(*child, None)?;
    }
    if quit_after_execution {
        Ok(ToQuit::Quit)
    } else {
        Ok(ToQuit::Continue)
    }
}
