extern crate nix;
extern crate sheller;

// use nix::sys::wait::*;
// use nix::unistd::*;
// use std::ffi::CString;
// use sheller::parser::*;
// use sheller::lexer::*;

fn main() {
    // match fork().expect("fork failed") {
    //     ForkResult::Parent{ child } => {
    //         // sleep(5);
    //         // kill(child, SIGKILL).expect("kill failed");
    //         match waitpid(child, None) {
    //             Ok(_) => println!("Process exited without error"),
    //             Err(err) => println!("Error: {:?}", err),
    //         };
    //     }
    //     ForkResult::Child => {
    //         match execvp(&CString::new("bash").unwrap(), [].as_ref()) {
    //             Ok(_) => (),
    //             Err(err) => println!("Error: {:?}", err),
    //         };  // until killed
    //     }
    // }
    println!("Blah");
}
