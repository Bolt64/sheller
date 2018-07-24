extern crate nix;
extern crate rustyline;

use executor::nix::sys::wait::waitpid;
use executor::nix::unistd::*;
use lexer::get_function_from_string;
use lexer::ParseError;
use lexer::Function;
use executor::rustyline::error::ReadlineError;
use executor::rustyline::Editor;


pub enum ToQuit {
    Quit,
    Continue,
}

fn execute_all_functions(functions: Vec<Function>) -> nix::Result<ToQuit> {
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
                        match command.execute() {
                            Ok(_) => (),
                            Err(err) => println!("Exectution Error {:?}", err),
                        };
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

fn run_string_input(string_input: &str) -> Result<nix::Result<ToQuit>, ParseError> {
    let functions = get_function_from_string(string_input)?;
    Ok(execute_all_functions(functions))
}

pub fn run_shell_mode(history_file: &str) {
    let mut prompt = Editor::<()>::new();
    if prompt.load_history(history_file).is_err() {
        println!("No history file");
    }
    loop {
        let line = prompt.readline("sheller>> ");
        match line {
            Ok(input_string) => {
                prompt.add_history_entry(input_string.as_ref());
                match run_string_input(&input_string) {
                    Err(parse_error) => println!("ParseError {:?}", parse_error),
                    Ok(inner_result) => match inner_result {
                        Err(nix_error) => println!("NixError {:?}", nix_error),
                        Ok(result) => match result {
                            ToQuit::Quit => {
                                prompt.save_history(history_file).unwrap();
                                break
                            },
                            ToQuit::Continue => (),
                        },
                    },
                }
            }
            Err(ReadlineError::Interrupted) => println!("KeyboardInterrupt"),
            Err(ReadlineError::Eof) => {
                println!("Ctrl-D received. Exiting.");
                prompt.save_history(history_file).unwrap();
                break;
            }
            Err(err) => {
                println!("Readline Error: {:?}", err);
                prompt.save_history(history_file).unwrap();
                break;
            }
        }
    }
}
