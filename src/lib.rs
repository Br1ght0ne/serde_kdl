//! `serde_kdl` provides [Serde] serialization/deserialization support for the
//! [`kdl` crate], which implements [KDL] document language in Rust.
//!
//! ## Usage
//!
//! Just wrap your `KdlNode`s and `KdlValue`s
//! with `Node`s and `Value`s correspondingly.
//!
//! [Serde]: https://serde.rs
//! [`kdl` crate]: https://crates.io/crates/kdl
//! [KDL]: https://kdl.dev/

mod node;
mod value;

pub use node::Node;
pub use value::Value;
