//! DSP integration layer — routes bids to demand-side platforms.

pub mod router;
pub mod clients;

pub use router::DspRouter;
