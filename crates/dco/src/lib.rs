//! Dynamic Creative Optimization engine — assembles and scores ad creatives
//! from templates using Thompson Sampling and user-segment affinity.

pub mod types;
pub mod engine;
pub mod assembler;
pub mod scorer;

pub use engine::DcoEngine;
pub use assembler::CreativeAssembler;
