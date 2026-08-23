//! Declarative, multi-page Menu System for GoldSrc.rs.

pub mod builder;
pub mod types;

pub use builder::{Menu, MenuBuilder, RenderedMenuPage, SlotAction};
pub use types::{
    Condition, DenyAction, DenyPolicy, ExitBehavior, ItemKind, ItemTitle, MenuContext, MenuItem,
    MenuRendererKind, MenuStyle, VisualDeny,
};
