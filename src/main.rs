// extern crate nix;
extern crate sheller;

// use nix::sys::wait::*;
// use nix::unistd::*;
// use std::ffi::CString;
use sheller::lexer::*;
use sheller::executor::*;

fn main() {
    /*
    match fork().expect("fork failed") {
        ForkResult::Parent { child } => {
            // sleep(5);
            // kill(child, SIGKILL).expect("kill failed");
            match waitpid(child, None) {
                Ok(_) => println!("Process exited without error"),
                Err(err) => println!("Error: {:?}", err),
            };
        }
        ForkResult::Child => {
            match execvp(
                &CString::new("ls").unwrap(),
                [CString::new("ls").unwrap(), CString::new("-l").unwrap()].as_ref(),
            ) {
                Ok(_) => (),
                Err(err) => println!("Error here: {:?}", err),
            }; // until killed
        }
    }
     */
    // println!("Blah");
    let string = "sleep 30; sleep 60";
    let functions = get_function_from_string(&string).unwrap();
    execute_all_functions(functions);
    // let tokens = tokenize_string(&string).unwrap();
    // let commands = Command::new(tokens).unwrap();
    // let commands = convert_tokens(tokens);
    // println!("{:?}", commands);
    // command.execute();
}
