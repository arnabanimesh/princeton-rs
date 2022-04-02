fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        panic!("Example usage: cargo run --example demo percolation_grid_side_length trial_count");
    }
    let ps =
        percolationstats::PercolationStats::new(args[1].parse().unwrap(), args[2].parse().unwrap());
    /* println!("mean                    = {}", ps.mean());
    println!("stddev                  = {}", ps.stddev());
    println!(
        "95% confidence interval = [{}, {}]",
        ps.confidence_lo(),
        ps.confidence_hi()
    ); */
    println!(
        "Percolation succeeded when {:.2}% boxes were opened on average with {:.3}% margin of error at 95% CI",
        ps.mean() * 100.,
        (ps.mean() - ps.confidence_lo()) * 100.
    )
}
