//! DSP integration layer — routes bids to demand-side platforms.

pub mod audience_proxy;
pub mod clients;
pub mod router;

pub use audience_proxy::AudienceProxyEngine;
pub use router::DspRouter;
