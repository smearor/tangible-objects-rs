//! Opt-in per-candidate diagnostics for the recording pipeline.

use tangible_objects_model::CandidateReport;

/// Sink receiving one [`CandidateReport`] per tested point combination.
///
/// The report is built lazily via `make`: a sink that discards it - the
/// production path's [`NoDiagnostics`] - never constructs it, so the hot
/// matching loop allocates nothing when recording is off.
pub(crate) trait CandidateDiagnostics {
    /// Records one tested candidate; `make` builds the report on demand.
    fn report(&mut self, make: impl FnOnce() -> CandidateReport);
}

/// No-op diagnostics sink used on the production path.
///
/// Monomorphizes to an empty function; together with the lazy `make`
/// parameter the whole instrumentation compiles away.
pub(crate) struct NoDiagnostics;

impl CandidateDiagnostics for NoDiagnostics {
    #[inline(always)]
    fn report(&mut self, _make: impl FnOnce() -> CandidateReport) {}
}

impl CandidateDiagnostics for Vec<CandidateReport> {
    fn report(&mut self, make: impl FnOnce() -> CandidateReport) {
        self.push(make());
    }
}
