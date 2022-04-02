use percolation::*;
use rand::{
    distributions::{Distribution, Uniform},
    thread_rng,
};

const CONFIDENCE_95: f64 = 1.96;

pub struct PercolationStats {
    threshold: Vec<f64>,
}

impl PercolationStats {
    pub fn new(n: usize, trials: usize) -> Self {
        if n == 0 || trials == 0 {
            panic!("n and trials should both be positive");
        }
        let size = n * n;
        let mut threshold: Vec<f64> = Vec::with_capacity(trials);
        let dist = Uniform::new_inclusive(1, n);
        let mut rng = thread_rng();
        for _ in 0..trials {
            let mut p = Percolation::new(n);
            while !p.percolates() {
                p.open(dist.sample(&mut rng), dist.sample(&mut rng));
            }
            threshold.push(p.number_of_open_sites() as f64 / size as f64);
        }
        PercolationStats { threshold }
    }
    pub fn mean(&self) -> f64 {
        self.threshold.iter().sum::<f64>() / self.threshold.len() as f64
    }
    pub fn stddev(&self) -> f64 {
        if self.threshold.len() == 1 {
            return 0.;
        }
        let mean = self.mean();
        (self
            .threshold
            .iter()
            .map(|&val| {
                let diff = mean - val;
                diff * diff
            })
            .sum::<f64>()
            / (self.threshold.len() - 1) as f64)
            .sqrt()
    }
    pub fn confidence_lo(&self) -> f64 {
        self.mean() - CONFIDENCE_95 * self.stddev() / (self.threshold.len() as f64).sqrt()
    }
    pub fn confidence_hi(&self) -> f64 {
        self.mean() + CONFIDENCE_95 * self.stddev() / (self.threshold.len() as f64).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use crate::PercolationStats;
    #[test]
    #[should_panic]
    fn n_zero_panic() {
        let ps = PercolationStats::new(0, 60);
        assert_ne!(ps.stddev(), 0.);
    }
    #[test]
    #[should_panic]
    fn trials_zero_panic() {
        let ps = PercolationStats::new(100, 0);
        assert_ne!(ps.stddev(), 0.);
    }
}
