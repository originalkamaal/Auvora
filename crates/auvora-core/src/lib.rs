//! Auvora domain primitives shared by every crate.
//!
//! - [`id`]: prefixed, UUIDv7-based external identifiers ([ADR-0018]).
//! - [`time`]: the injectable [`time::Clock`] (ADR-0047 §10).
//!
//! [ADR-0018]: https://github.com/originalkamaal/Auvora/blob/main/docs/adr/0018-identifier-format-uuidv7-prefixed.md

pub mod id;
pub mod time;
