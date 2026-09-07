// Using the standard thread crate
use std::thread;

// static value NO_THREADS
static NO_THREADS: i32 = 10;

// Main thread starts here
fn main() {

    // Make a mutable vector named thread_holder to hold the threads spawned
    let mut thread_holder = vec![];

    for i in 0..NO_THREADS {
        // Spin up another thread
        thread_holder.push(thread::spawn(move || {
            println!("Thread number is {}", i);
            i
        }));
    }

    println!("***************************");
    
    for thread_elements in thread_holder {
        // Wait for the thread to finish. Returns a result.
        println!("Thread returned {:?}",
        thread_elements.join().unwrap());
    }
}