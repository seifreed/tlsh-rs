//! Pure Rust implementation of TLSH profiles and tooling.
//!
//! Supported digest profiles:
//! - `128-1` canonical `T1`
//! - `128-3`
//! - `256-1`
//! - `256-3`
//!
//! The crate exposes a pure-Rust hashing core plus a CLI module with text,
//! JSON and SARIF output helpers.

mod builder;
pub mod cli;
mod digest;
mod error;
mod internal;
mod profile;

pub use builder::{TlshBuilder, TlshOptions, hash_bytes, hash_bytes_with_profile};
pub use digest::TlshDigest;
pub use error::TlshError;
pub use profile::{BucketKind, ChecksumKind, TlshProfile};
