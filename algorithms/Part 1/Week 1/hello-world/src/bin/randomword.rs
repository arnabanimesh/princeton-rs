use rand::{
    distributions::{Bernoulli, Distribution},
    thread_rng,
};

fn main() {
    let mut out = String::new();
    std::env::args().enumerate().skip(1).for_each(|arg| {
        if Bernoulli::from_ratio(1, arg.0 as u32)
            .unwrap()
            .sample(&mut thread_rng())
        {
            out = arg.1;
        }
    });
    println!("{}", out);
}
