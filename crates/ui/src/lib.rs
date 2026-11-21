//! Shared view tokens: palette, type scale and icon catalogue.
//!
//! These are the parts of the UI that carry no application state, so they can
//! move out of the client first and be edited independently. Screen modules
//! follow as the `UiState`/`Act` boundary lands.

pub mod icons;
pub mod theme;
