// Calling the standard libraries
use std::process::Command;

// Main execution starts here
fn main() {

    // Creating a child process
    let mut child = Command::new("sleep").arg("5").spawn().unwrap();

    // Waiting for the child process to complete
    let _result = child.wait().unwrap();
    // printing the status of child process
    print!("Status if child process {} \n", _result);
    // Marking the end of the main function
    println!("reached end of main");
    
}