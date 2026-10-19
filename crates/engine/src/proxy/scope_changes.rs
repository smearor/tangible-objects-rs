use tangible_objects_model::StageName;

/// Scope diff applied by [`super::Pipeline::sync_scopes`]
/// (EVENT_PIPELINE.md phase 6) - the caller reports each side as
/// lifecycle events (`StageRegistered`/`StageUnregistered`).
#[derive(Debug, Default, Clone)]
pub struct ScopeChanges {
    /// Scope names that got a new tangible stage.
    pub registered: Vec<StageName>,
    /// Scope names whose tangible stage was removed - its suppression
    /// state died with it.
    pub unregistered: Vec<StageName>,
}
