//! Dynamic Creative Optimization engine — assembles and scores ad creatives
//! from templates using Thompson Sampling and user-segment affinity.

pub mod assembler;
pub mod asset_ops;
pub mod brand;
pub mod engine;
pub mod scorer;
pub mod studio;
pub mod types;

pub use assembler::CreativeAssembler;
pub use asset_ops::{AssetIngestionEngine, AssetWorkflowEngine, RenditionEngine, RightsManager};
pub use brand::{AssetLibrary, BrandGuidelinesEngine};
pub use engine::DcoEngine;
pub use studio::{
    CreativeApprovalEngine, CreativePerformanceTracker, DcoStudio, PlacementRegistry,
};
