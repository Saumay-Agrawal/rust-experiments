use std::thread;

fn main() {

    thread::spawn(move || {
        println!("Hello from spawned thread");
    });
    
    let join_handle = thread::spawn(move || {
        println!("Hello from second spawned thread");
        20
    });
    
    println!("Hello from the main thread");
    
    match join_handle.join() {
        Ok(x) => println!("Second spawned thread returned {}", x),
        Err(_) => println!("Second spawned thread panicked")
    }
}