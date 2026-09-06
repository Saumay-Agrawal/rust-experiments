use std::{f64};

fn main() {

    // create a struct variable
    let mut circle1 = Circle {
        x: 10.0,
        radius: 10.0
    };
    
    // print radius and variable x
    println!("x: {}, radius: {}", circle1.x, circle1.radius);
    println!("Radius : {}", get_radius(&circle1));

}

// define your custom user data type
struct Circle {
    x: f64,
    radius: f64,
}
// get radius function
fn get_radius(c1: &Circle) -> f64{
    c1.radius
}