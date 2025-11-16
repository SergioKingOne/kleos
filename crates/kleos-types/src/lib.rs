//! Domain types for Kleos streaming architecture
//!
//! This crate implements type-driven development principles:
//! - Newtype pattern for semantic type safety
//! - Construction-time validation
//! - Making illegal states unrepresentable
//! - Exhaustive pattern matching for state handling

pub mod validated;
pub mod ids;
pub mod events;

pub use validated::*;
pub use ids::*;
pub use events::*;
