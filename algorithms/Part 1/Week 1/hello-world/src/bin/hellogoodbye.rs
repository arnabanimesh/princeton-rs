fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 3 {
        println!("Hello {} and {}.", &args[1], &args[2]);
        println!("Goodbye {} and {}.", &args[2], &args[1]);
    } else {
        println!("Usage: Enter two CLI arguments e.g. ./hellogoodbye Mike Jack")
    }
}
