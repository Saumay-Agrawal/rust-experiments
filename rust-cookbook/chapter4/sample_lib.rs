pub fn public_function() {
    println!("called sample_lib::public_function()");
}

fn private_function() {
    println!("called sample_lib::private_function()");
}

pub fn indirect_access() {
    print!("called sample_lib::indirect_access(), that \n > ");
    private_function();
}