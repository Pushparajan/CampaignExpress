//! DSP integration layer — routes bids to demand-side platforms.

pub mod clients;
pub mod router;

pub use router::DspRouter;
