//! Transparent WASM guest logger for GoldSrc.rs plugins.

pub mod guest;
pub use guest::init_guest_logger;
