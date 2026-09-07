use std::num::ParseIntError;

fn double_number(number_str: &str) -> Result<i32, ParseIntError> {
    match number_str.parse::<i32>() {
        Ok(n) => Ok(2 * n),
        Err(e) => Err(e),
    }
}

fn double_number_map(number_str: &str) -> Result<i32, ParseIntError> {
    number_str.parse::<i32>().map(|n| 2 * n)
}


fn print(result: Result<i32, ParseIntError>) {
    match result {
        Ok(n) => println!("n is {}", n),
        Err(e) => println!("Error: {}", e),
    }
}

fn main() {
    
    // This still presents a reasonable answer.
    let twenty = double_number("10");
    print(twenty);
    
    // The following now provides a much more helpful error message.
    let tt = double_number_map("t");
    print(tt);
}