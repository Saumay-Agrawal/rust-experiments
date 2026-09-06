use std::{i8,i16,i32,i64,u8,u16,u32,u64,f32,f64,isize,usize};
use std::io::stdin;

fn main() {

    println!("Understanding assignment");
    // Compiler will automatically figure out the data type if not mentioned
    // Cannot change the value
    let num = 10;
    println!("Num is {}", num);

    let age: i32 = 40;
    println!("Age is {}", age);
    // Prints the max and min value of 32bit integer
    println!("Max i32 {}", i32::MAX);
    println!("Max i32 {}", i32::MIN);

    // Another way of variable assigning
    let (f_name, l_name) = ("viki", "d");
    println!("First name {0} and last name {1}", f_name, l_name);

}
