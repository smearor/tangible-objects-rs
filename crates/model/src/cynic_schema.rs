//! Cynic schema bindings (`cynic` feature) - registers the exported
//! `schema.graphql` and maps model scalar newtypes onto their schema
//! scalars, mirroring the `async-graphql`/`zbus`/`mcp` feature-gate
//! pattern.

/// Schema types generated from `crates/graphql/schema.graphql` -
/// every `#[cynic(schema_path = ...)]` derive resolves against it
/// (re-exported as `crate::schema`, cynic's default lookup path).
pub mod schema {
    cynic::use_schema!("../graphql/schema.graphql");
}

cynic::impl_scalar!(crate::completion::AffinityFactor, schema::AffinityFactor);
cynic::impl_scalar!(crate::ContactCount, schema::ContactCount);
cynic::impl_scalar!(crate::EventCount, schema::EventCount);
cynic::impl_scalar!(crate::FieldName, schema::FieldName);
cynic::impl_scalar!(crate::FieldSetName, schema::FieldSetName);
cynic::impl_scalar!(crate::FrameCount, schema::FrameCount);
cynic::impl_scalar!(crate::Millimeters, schema::Millimeters);
cynic::impl_scalar!(crate::Owner, schema::Owner);
cynic::impl_scalar!(crate::PatternId, schema::PatternId);
cynic::impl_scalar!(crate::PatternName, schema::PatternName);
cynic::impl_scalar!(crate::PatternTolerance, schema::PatternTolerance);
cynic::impl_scalar!(crate::PinCount, schema::PinCount);
cynic::impl_scalar!(crate::RawAxisValue, schema::RawAxisValue);
cynic::impl_scalar!(crate::RecordLabel, schema::RecordLabel);
cynic::impl_scalar!(crate::RotationRad, schema::RotationRad);
cynic::impl_scalar!(crate::completion::SmoothingAlpha, schema::SmoothingAlpha);
cynic::impl_scalar!(crate::StageName, schema::StageName);
cynic::impl_scalar!(crate::TangibleId, schema::TangibleId);
cynic::impl_scalar!(crate::TangibleSetId, schema::TangibleSetId);
cynic::impl_scalar!(crate::TouchPointSlot, schema::TouchPointSlot);
cynic::impl_scalar!(crate::TrackingId, schema::TrackingId);
cynic::impl_scalar!(crate::UnitsPerMm, schema::UnitsPerMm);
// `JSON` passthrough fields (`metrics`, `analyze*`) select as
// `serde_json::Value` - the response still decodes into the typed
// reports through the model's `Deserialize`.
cynic::impl_scalar!(serde_json::Value, schema::JSON);
