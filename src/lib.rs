//! # dioxus-mosaic
//!
//! A React-Mosaic-style tiling window manager library for Dioxus applications.
//!
//! ## Features
//!
//! - **HashMap-based architecture** - O(1) operations for smooth 60 FPS performance
//! - **Binary splits** - Simple, proven pattern (like VSCode, Sublime)
//! - **Resizable dividers** - Drag to resize panes
//! - **Dynamic splitting** - Split any tile horizontally or vertically
//! - **Panel controls** - Close tiles, collapse/expand
//! - **LocalStorage persistence** - Layout survives page reloads
//! - **Clean builder API** - Easy-to-use tree-like configuration
//!
//! ## Quick Start
//!
//! ```ignore
//! use dioxus::prelude::*;
//! use dioxus_mosaic::{Mosaic, MosaicBuilder, tile};
//!
//! #[component]
//! fn App() -> Element {
//!     let mut layout = use_signal(|| {
//!         MosaicBuilder::horizontal()
//!             .left(tile("sidebar"))
//!             .right(tile("editor"))
//!             .split(25.0)
//!             .build()
//!     });
//!
//!     rsx! {
//!         Mosaic {
//!             layout: layout,
//!             render_tile: move |tile_id| {
//!                 match tile_id.as_str() {
//!                     "sidebar" => rsx! { div { "Sidebar" } },
//!                     "editor" => rsx! { div { "Editor" } },
//!                     _ => None
//!                 }
//!             },
//!         }
//!     }
//! }
//! ```

mod builder;
mod drag_drop;
mod layout;
mod mosaic;
mod node;
mod split_pane;
mod tile_pane;
mod tree_api;
mod types;

// Re-export public API
pub use builder::{tile, MosaicBuilder};
pub use drag_drop::{DragGhost, DragState, DropZone};
pub use layout::MosaicLayout;
pub use mosaic::Mosaic;
pub use split_pane::SplitPane;
pub use tile_pane::TilePane;
pub use tree_api::MosaicNode;
pub use types::{NodeId, SplitDirection, TileId};
