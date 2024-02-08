use scanner::tokenize;

fn main() {
    println!("Hello, world!");
    let tokens = tokenize("../scanner/run_tests.py");
    println!("{:?}", tokens);
}
