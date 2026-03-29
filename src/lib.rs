//! Courier service lib
//!
//! # Module overview
//! - [`offers`]    — offer code definitions and eligibility checking.
//! - [`cost`]      — raw cost formula and per-package cost results.
//! - [`scheduler`] — vehicle scheduling, shipment selection, delivery times.

pub mod cost;
pub mod offers;
pub mod scheduler;
