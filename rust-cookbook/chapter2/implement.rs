use std::{f64};

fn main() {
    // create a struct variable
    let mut circle1 = Circle {
        x: 10.0,
        radius: 10.0
    };
    println!("x: {}, radius: {}", circle1.x, circle1.radius );
    println!("x: {}", circle1.get_x());
}

// define your custom user data type
struct Circle {
    x : f64,
    radius : f64,
}

// recommended way of creating structs
impl Circle {
    // pub makes this function public which makes it accessible outsite the scope {}
    pub fn get_x(&self) -> f64 {
        self.x
    }
}