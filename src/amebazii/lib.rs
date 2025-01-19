#![cfg_attr(
    feature = "documentation",
    doc = "See the [CLI documentation](./doc/cli/index.html)."
)]

pub mod error;
pub mod util;
pub mod types;
pub mod keys;
pub mod map;
pub mod conf;

#[cfg(feature = "documentation")]
/// Documentation
pub mod doc;