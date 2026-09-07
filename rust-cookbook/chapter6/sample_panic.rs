// function which checks if the strings are same or not
fn compare_stmt(stmt: &str) {
    // Check if the statements are same or not
    if stmt == "Another book" {
        panic!("Rust Cookbook is not selected!!!!");
    }
    println!("Statements is {}!!!!!", stmt);
}

// Execution starts here
fn main() {
    compare_stmt("Rust Cookbook");
    compare_stmt("Another book");
}