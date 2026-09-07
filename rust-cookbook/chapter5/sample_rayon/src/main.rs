// Calling the rayon crate
extern crate rayon;
use rayon::prelude::*;

// Sum of squares function
fn sum_of_squares(input: &[i32]) -> i32 {
    input.par_iter()
        .map(|&i| i * i)
        .sum()
}

// Main execution of code
fn main() {

    // Declaring a random variable of 10
    let numbers = [1, 2, 3, 4, 5];

    // Calling the method to get sum_of_squares
    let sum_sq = sum_of_squares(&numbers);

    // Printing the result
    println!("Sum of squares of {:?} is {1}", numbers, sum_sq);
}