//! Fundamental game data types, spatial mathematics, and engine descriptors.

pub mod edict;
pub mod liblist;
pub mod spatial;
pub mod vector;
pub mod vital;

pub use edict::{EDict, bump_map_generation, current_map_generation};
pub use liblist::{LIBLIST_FILENAME, LibList};
pub use spatial::{Angles, Origin, Velocity};
pub use vector::Vector3;
pub use vital::{Armor, Health};
