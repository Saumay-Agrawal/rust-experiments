fn main() {

    // Arithmetic Operations
    println!("5 + 4 = {}", 5 + 4);
    println!("5 - 4 = {}", 5 - 4);
    println!("5 * 4 = {}", 5 * 4);
    println!("5 / 4 = {}", 5 / 4);
    println!("5 % 4 = {}", 5 % 4);
    
    // Assigning data types and mathematical Operations
    let neg_4 = -4i32;
    println!("abs(-4) = {}", neg_4.abs());
    println!("abs(-4) = {}", neg_4.pow(2));
    println!("round(1.2345) = {}", 1.2354f64.round());
    println!("ceil(1.2345) = {}", 1.2345f64.ceil());
    println!("sin 3.14 = {}", 3.14f64.sin());

}