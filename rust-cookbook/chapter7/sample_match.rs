macro_rules! Check_Val {
    (x => $e:expr) => (println!("mode X: {}", $e));
    (y => $e:expr) => (println!("mode Y: {}", $e));
}

fn main() {
    Check_Val!(x => 5);
    Check_Val!(y => 3)
}