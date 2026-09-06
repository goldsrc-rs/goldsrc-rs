//! Fundamental game data types, spatial mathematics, and engine descriptors.

pub mod edict;
pub mod liblist;
pub mod vector;

pub use edict::{EDict, bump_map_generation, current_map_generation};
pub use liblist::{LIBLIST_FILENAME, LibList};
pub use vector::Vector3;
