//! Operating mode implementations.
//!
//! Each operating mode has distinct initialization, runtime behavior,
//! and safety constraints.

pub mod replay;
pub mod prospective;
pub mod mock;
