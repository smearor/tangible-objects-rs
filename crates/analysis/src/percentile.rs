/// Nearest-rank summary statistics of a measurement sample.
///
/// `None`-safe via [`Percentiles::of`] returning `Option` - empty samples
/// produce no report line rather than a degenerate row of zeros.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub struct Percentiles {
    /// Number of samples.
    pub count: usize,
    /// Arithmetic mean.
    pub mean: f64,
    /// Median.
    pub p50: f64,
    /// 95th percentile.
    pub p95: f64,
    /// 99th percentile.
    pub p99: f64,
    /// Maximum sample value.
    pub max: f64,
    /// Minimum sample value.
    pub min: f64,
}

impl Percentiles {
    /// Computes percentiles over `samples`; `None` for an empty sample.
    ///
    /// Accepts any `Into<f64>` item - newtyped measurements (`Millimeters`,
    /// `MatchScore`) feed directly (`samples.iter().copied()`), the unit
    /// crosses only at this computation boundary where stats are
    /// dimensionless.
    pub fn of<T: Into<f64>>(samples: impl Iterator<Item = T>) -> Option<Self> {
        let mut sorted: Vec<f64> = samples.map(Into::into).collect();
        if sorted.is_empty() {
            return None;
        }
        sorted.sort_by(f64::total_cmp);
        let quantile = |q: f64| sorted[(q * (sorted.len() - 1) as f64).round() as usize];
        Some(Self {
            count: sorted.len(),
            mean: sorted.iter().sum::<f64>() / sorted.len() as f64,
            p50: quantile(0.50),
            p95: quantile(0.95),
            p99: quantile(0.99),
            max: *sorted.last().unwrap_or(&0.0),
            min: sorted[0],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Percentiles;

    #[test]
    fn empty_sample_is_none() {
        assert!(Percentiles::of(std::iter::empty::<f64>()).is_none());
    }

    #[test]
    fn percentiles_of_sorted_sample() {
        let stats = Percentiles::of([1.0, 2.0, 3.0, 4.0, 100.0].into_iter()).expect("stats");
        assert_eq!(stats.count, 5);
        assert_eq!(stats.p50, 3.0);
        assert_eq!(stats.p99, 100.0);
        assert_eq!(stats.max, 100.0);
        assert!((stats.mean - 22.0).abs() < 1e-9);
    }
}
