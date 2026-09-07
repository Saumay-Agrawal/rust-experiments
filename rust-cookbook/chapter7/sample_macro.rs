// This is a simple macro named `say_hello`.
macro_rules! Welcome_RustBook {
    () => (
        // The macro will expand into the contents of this block.
        println!("Welcome to Rust Cookbook!")
    )
}

fn main() {
    // This call will expand into`println!("Hello");`
    Welcome_RustBook!();
}

