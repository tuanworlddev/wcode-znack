//! Честный ЗНАК (Znack) integration: GTIN sync from the National Catalog,
//! category+gender → GTIN mapping, KIZ purchase via СУЗ, automatic
//! introduction into circulation, and SGTIN printing support.
//!
//! Ported from the reference implementation in docs/znack_api/znack/.

pub mod auth;
pub mod client;
pub mod commands;
pub mod db;
pub mod introduction;
pub mod models;
pub mod pipeline;
pub mod sign;
pub mod sync;
