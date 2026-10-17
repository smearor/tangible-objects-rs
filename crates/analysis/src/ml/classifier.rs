/// Dependency-free logistic-regression baseline over normalized deviation
/// features (TRAINING.md 3.3).
///
/// Deliberately minimal: plain gradient descent, fixed epochs - a baseline
/// to decide whether learned matching beats the score threshold, not a
/// production classifier.
#[derive(Debug, Clone)]
pub struct LogisticBaseline {
    /// Feature weights.
    pub weights: Vec<f64>,
    /// Bias term.
    pub bias: f64,
}

impl LogisticBaseline {
    /// Trains on `(features, is_match)` samples via batch gradient descent.
    ///
    /// Returns `None` when the sample set is empty or one-dimensional labels
    /// are all identical - nothing to learn.
    pub fn train(samples: &[Vec<f64>], labels: &[bool], epochs: usize, rate: f64) -> Option<Self> {
        if samples.is_empty() {
            return None;
        }
        let width = samples.first()?.len();
        if labels.iter().all(|label| *label) || labels.iter().all(|label| !*label) {
            return None;
        }

        let mut model = Self {
            weights: vec![0.0; width],
            bias: 0.0,
        };
        let n = samples.len() as f64;
        for _ in 0..epochs {
            let mut grad_w = vec![0.0; width];
            let mut grad_b = 0.0;
            for (features, &label) in samples.iter().zip(labels.iter()) {
                let error = model.probability(features) - label as u8 as f64;
                for (weight, feature) in grad_w.iter_mut().zip(features.iter()) {
                    *weight += error * feature;
                }
                grad_b += error;
            }
            for (weight, grad) in model.weights.iter_mut().zip(grad_w.iter()) {
                *weight -= rate * grad / n;
            }
            model.bias -= rate * grad_b / n;
        }
        Some(model)
    }

    /// Sigmoid probability that `features` belong to a matching candidate.
    pub fn probability(&self, features: &[f64]) -> f64 {
        let z: f64 = self
            .weights
            .iter()
            .zip(features.iter())
            .map(|(w, x)| w * x)
            .sum::<f64>()
            + self.bias;
        1.0 / (1.0 + (-z).exp())
    }

    /// Binary prediction at threshold 0.5.
    pub fn predicts(&self, features: &[f64]) -> bool {
        self.probability(features) > 0.5
    }
}

#[cfg(test)]
mod tests {
    use super::LogisticBaseline;

    #[test]
    fn separates_obvious_classes() {
        // Class 1: all-normalized deviations small; class 0: all large.
        let mut samples = Vec::new();
        let mut labels = Vec::new();
        for i in 0..40 {
            let jitter = (i % 5) as f64 * 0.01;
            samples.push(vec![0.2 + jitter, 0.3 + jitter]);
            labels.push(true);
            samples.push(vec![2.0 + jitter, 2.5 + jitter]);
            labels.push(false);
        }
        let model = LogisticBaseline::train(&samples, &labels, 500, 0.5).expect("model");
        assert!(model.predicts(&[0.2, 0.3]));
        assert!(!model.predicts(&[2.0, 2.5]));
    }

    #[test]
    fn refuses_single_class_samples() {
        let samples = vec![vec![1.0], vec![2.0]];
        let labels = vec![true, true];
        assert!(LogisticBaseline::train(&samples, &labels, 10, 0.5).is_none());
    }
}
