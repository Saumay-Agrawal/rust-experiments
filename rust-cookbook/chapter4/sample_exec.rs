// Imports all items under sample_lib
extern crate sample_lib;

fn main() {
    // Calling public_function
    sample_lib::public_function();
    // Calling indirect_access to private_function
    sample_lib::indirect_access();
}