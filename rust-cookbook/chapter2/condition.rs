use std::{i32};

fn main() {

    let age: i32 = 10;
    
    // If else statements
    if age <= 18 {
        println!("Go to School");
    } else if (age > 18) && (age <= 28) {
        println!("Go to college");
    } else {
        println!("Do something with your life");
    }
    
    // if/ else statement in one line
    let can_vote = if (age >= 18) {true} else {false};
    println!("Can vote {}",can_vote );
}