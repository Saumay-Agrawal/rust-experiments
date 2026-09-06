use std::{i8};

fn main() {

    // Declaring a tuple
    let rand_tuple = ("Rust", 2017);
    let rand_tuple2 : (&str, i8) = ("Viki", 4);
    
    // tuple operations
    println!(" Name : {}", rand_tuple2.0);
    println!(" Lucky no : {}", rand_tuple2.1);

}