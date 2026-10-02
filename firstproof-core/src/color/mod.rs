//! Scene-referred Color Science Core for First Proof
//!
//! Re-exports canonical color science models, transfer functions, and reference pipeline
//! from `firstproof_engine::color`, augmented with Adobe DCP profile parsing and CIEDE2000 metrics.

pub use firstproof_engine::color::*;

pub mod ciede2000;
pub mod dcp;

pub use dcp::{DcpHueSatMap, DcpProfile};
