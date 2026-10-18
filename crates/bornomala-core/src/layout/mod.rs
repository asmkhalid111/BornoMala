//! Fixed Keyboard Layouts Module for Borno Core.
//!
//! Provides schema definitions, built-in layout tables (National, Probhat, Bijoy),
//! and the deterministic fixed layout composition engine.

pub mod schema;
pub mod builtin;
pub mod engine;

pub use schema::{
    KeyMapping, LayoutDefinition, LayoutError, LayoutMetadata, LayoutRules, ModifierLayer,
    Modifiers, PhysicalKey,
};
pub use builtin::{
    build_bijoy_layout, build_national_layout, build_probhat_layout, get_bijoy_layout,
    get_layout_by_id, get_national_layout, get_probhat_layout, FixedLayoutId,
};
pub use engine::{CompositionState, FixedLayoutEngine};
