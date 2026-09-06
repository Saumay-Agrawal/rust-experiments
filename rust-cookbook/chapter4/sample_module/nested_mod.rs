// sample_mod/nested.rs
pub fn sample_function() {
    println!("called `sample_module::nested::sample_function()`");
}

#[allow(dead_code)]
fn private_function() {
    println!("called `sample_module::nested::private_function()`");
}

