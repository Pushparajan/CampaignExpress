//! Dynamic Creative Optimization engine — assembles and scores ad creatives
//! from templates using Thompson Sampling and user-segment affinity.

pub mod assembler;
pub mod brand;
pub mod engine;
pub mod scorer;
pub mod types;

pub use assembler::CreativeAssembler;
pub use brand::{AssetLibrary, BrandGuidelinesEngine};
pub use engine::DcoEngine;
