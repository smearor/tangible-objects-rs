use std::fmt::Display;

use tangible_objects_model::FrameCount;

use super::Accuracy;
use super::LogisticBaseline;
use crate::Recording;

/// Gradient-descent epochs for the baseline — fixed for determinism.
const EPOCHS: usize = 500;
/// Learning rate of the baseline.
const LEARNING_RATE: f64 = 0.5;
/// Every Nth labeled candidate goes to the held-out set (deterministic).
const TEST_EVERY: usize = 5;

/// Baseline ML evaluation over a recording (TRAINING.md 3.3).
///
/// Trains a [`LogisticBaseline`] on labeled candidates — features are the
/// normalized deviations (`dev[i] / tolerance[i]`), the label is whether the
/// candidate's pattern is the frame's ground-truth label — and compares its
/// held-out accuracy against the binary score rule (`score <= 1.0`).
#[derive(Debug)]
pub struct MlReport {
    /// Labeled candidates used (train + held-out).
    pub samples: FrameCount,
    /// Held-out candidates evaluated.
    pub test_samples: FrameCount,
    /// Rule baseline (`score <= 1.0`) accuracy on the held-out set.
    pub rule_accuracy: Option<Accuracy>,
    /// Logistic-regression accuracy on the held-out set; `None` when too few
    /// samples or a single-class label set made training impossible.
    pub ml_accuracy: Option<Accuracy>,
    /// Classifier feature weights (inspectable, one per deviation index).
    pub weights: Vec<f64>,
}

impl MlReport {
    /// Trains and evaluates the baseline on labeled candidates.
    ///
    /// On recordings without labels the report is empty — the evaluation
    /// requires ground truth.
    pub fn analyze(recording: &Recording) -> Self {
        // Feature rows: normalized deviations; target: candidate pattern == label.
        let mut features = Vec::new();
        let mut targets = Vec::new();
        for frame in recording.frames() {
            let Some(label) = &frame.label else { continue };
            for candidate in &frame.candidates {
                if candidate.deviations.is_empty() {
                    continue;
                }
                features.push(
                    candidate
                        .deviations
                        .iter()
                        .enumerate()
                        .map(|(i, dev)| dev.as_f64() / candidate.tolerance_at(i).as_f64())
                        .collect::<Vec<f64>>(),
                );
                targets.push(candidate.matches_label(label));
            }
        }

        // Deterministic hold-out: every TEST_EVERY-th labeled candidate.
        let mut train_x = Vec::new();
        let mut train_y = Vec::new();
        let mut test_x = Vec::new();
        let mut test_y = Vec::new();
        for (index, (x, y)) in features.into_iter().zip(targets).enumerate() {
            if index % TEST_EVERY == TEST_EVERY - 1 {
                test_x.push(x);
                test_y.push(y);
            } else {
                train_x.push(x);
                train_y.push(y);
            }
        }

        let model = LogisticBaseline::train(&train_x, &train_y, EPOCHS, LEARNING_RATE);

        let accuracy = |predict: &dyn Fn(&[f64]) -> bool| -> Option<Accuracy> {
            let correct = test_x
                .iter()
                .zip(test_y.iter())
                .filter(|(x, y)| predict(x) == **y)
                .count();
            Accuracy::of(correct, test_x.len())
        };

        Self {
            samples: FrameCount::from(train_x.len() + test_x.len()),
            test_samples: FrameCount::from(test_x.len()),
            rule_accuracy: accuracy(&|x| x.iter().copied().reduce(f64::max).unwrap_or(0.0) <= 1.0),
            ml_accuracy: model.as_ref().and_then(|m| accuracy(&|x| m.predicts(x))),
            weights: model.map(|m| m.weights).unwrap_or_default(),
        }
    }
}

impl Display for MlReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "== ML baseline == ({} labeled candidates)", self.samples)?;
        if self.samples.as_u64() == 0 {
            writeln!(f, "  (no labeled frames — record with --record-label)")?;
            return Ok(());
        }
        if let Some(rule) = self.rule_accuracy {
            writeln!(
                f,
                "  rule (score<=1.0): {} accuracy on {} held-out samples",
                rule, self.test_samples
            )?;
        }
        match self.ml_accuracy {
            Some(acc) => writeln!(
                f,
                "  logistic baseline: {} (weights: {:?})",
                acc, self.weights
            )?,
            None => writeln!(
                f,
                "  logistic baseline: not trainable (single class / no data)"
            )?,
        }
        Ok(())
    }
}
