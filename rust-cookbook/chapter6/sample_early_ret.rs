type Result<T> = std::result::Result<T, String>;

fn double_first(vec: Vec<&str>) -> Result<i32> {
    let first = match vec.first() {
        Some(first) => first,
        None => return Err("Please use a vector with at least one element.".to_owned())
    };
    match first.parse::<i32>() {
        Ok(i) => Ok(2 * i),
        Err(e) => Err(e.to_string()),
    }
}

fn print(result: Result<i32>) {
    match result {
        Ok(n) => println!("The first doubled is {}", n),
        Err(e) => println!("Error: {}", e),
    }
}

fn main() {
    let empty = vec!["1","2","3"];
    let strings = vec!["tofu", "93", "18"];
    print(double_first(empty));
    print(double_first(strings));
}