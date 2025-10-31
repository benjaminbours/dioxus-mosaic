use crate::types::TileId;
use dioxus::prelude::*;
use dioxus::signals::*;
use serde::{Deserialize, Serialize};

/// Drop zone position when hovering over a tile
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DropZone {
    /// Top 25% of tile (creates vertical split with dragged tile on top)
    Top,
    /// Bottom 25% of tile (creates vertical split with dragged tile on bottom)
    Bottom,
    /// Left 25% of tile (creates horizontal split with dragged tile on left)
    Left,
    /// Right 25% of tile (creates horizontal split with dragged tile on right)
    Right,
}

impl DropZone {
    /// Get the split direction for this drop zone
    pub fn split_direction(&self) -> crate::types::SplitDirection {
        match self {
            DropZone::Top | DropZone::Bottom => crate::types::SplitDirection::Vertical,
            DropZone::Left | DropZone::Right => crate::types::SplitDirection::Horizontal,
        }
    }

    /// Whether the dragged tile should be the first child in the split
    pub fn dragged_is_first(&self) -> bool {
        match self {
            DropZone::Top | DropZone::Left => true,
            DropZone::Bottom | DropZone::Right => false,
        }
    }
}

/// Global drag state
#[derive(Clone, Default, PartialEq)]
pub struct DragState {
    /// ID of the tile currently being dragged
    pub dragging_tile_id: Option<TileId>,

    /// Current mouse/cursor position during drag
    pub drag_position: (f64, f64),

    /// Currently hovered target tile and drop zone
    pub hover_target: Option<(TileId, DropZone)>,
}

impl DragState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_dragging(&self) -> bool {
        self.dragging_tile_id.is_some()
    }

    pub fn start_drag(&mut self, tile_id: TileId, x: f64, y: f64) {
        self.dragging_tile_id = Some(tile_id);
        self.drag_position = (x, y);
        self.hover_target = None;
    }

    pub fn update_position(&mut self, x: f64, y: f64) {
        self.drag_position = (x, y);
    }

    pub fn update_hover(&mut self, tile_id: TileId, zone: DropZone) {
        self.hover_target = Some((tile_id, zone));
    }

    pub fn clear_hover(&mut self) {
        self.hover_target = None;
    }

    pub fn end_drag(&mut self) {
        self.dragging_tile_id = None;
        self.drag_position = (0.0, 0.0);
        self.hover_target = None;
    }
}

/// Calculate which drop zone the cursor is in based on position within target element
///
/// Returns None if in the center (no-drop zone)
pub fn calculate_drop_zone(
    mouse_x: f64,
    mouse_y: f64,
    rect_x: f64,
    rect_y: f64,
    rect_width: f64,
    rect_height: f64,
) -> Option<DropZone> {
    // Calculate relative position (0.0 to 1.0)
    let rel_x = ((mouse_x - rect_x) / rect_width).clamp(0.0, 1.0);
    let rel_y = ((mouse_y - rect_y) / rect_height).clamp(0.0, 1.0);

    // Define drop zone margins (30% from each edge for better UX)
    const MARGIN: f64 = 0.3;

    // Check edge zones (priority: edges over center)
    if rel_y < MARGIN {
        Some(DropZone::Top)
    } else if rel_y > 1.0 - MARGIN {
        Some(DropZone::Bottom)
    } else if rel_x < MARGIN {
        Some(DropZone::Left)
    } else if rel_x > 1.0 - MARGIN {
        Some(DropZone::Right)
    } else {
        // Center zone - no drop
        None
    }
}

/// Get the CSS style for a drop zone overlay
pub fn get_drop_zone_style(
    zone: DropZone,
    is_active: bool,
) -> String {
    let (position_props, size_props) = match zone {
        DropZone::Top => ("top: 0; left: 0; right: 0;", "height: 30%;"),
        DropZone::Bottom => ("bottom: 0; left: 0; right: 0;", "height: 30%;"),
        DropZone::Left => ("top: 0; bottom: 0; left: 0;", "width: 30%;"),
        DropZone::Right => ("top: 0; bottom: 0; right: 0;", "width: 30%;"),
    };

    let bg_color = if is_active {
        "rgba(59, 130, 246, 0.3)" // blue-500 with opacity
    } else {
        "rgba(59, 130, 246, 0.15)"
    };

    let border_color = if is_active {
        "rgba(59, 130, 246, 0.8)"
    } else {
        "rgba(59, 130, 246, 0.4)"
    };

    format!(
        "position: absolute; {}; {}; background-color: {}; border: 2px dashed {}; pointer-events: none; transition: all 0.15s ease; z-index: 10; border-radius: 4px;",
        position_props, size_props, bg_color, border_color
    )
}

/// Drag ghost component that follows the cursor
#[component]
pub fn DragGhost(drag_state: Signal<DragState>, render_title: Signal<Box<dyn Fn(TileId) -> Element>>) -> Element {
    let state = drag_state.read();

    // If not dragging, don't render anything
    if state.dragging_tile_id.is_none() {
        return rsx! { div { style: "display: none;" } };
    }

    let dragging_tile = state.dragging_tile_id.as_ref().unwrap().clone();

    let (x, y) = state.drag_position;

    // Offset so ghost appears slightly below and to the right of cursor
    let offset_x = x + 10.0;
    let offset_y = y + 10.0;

    let title = (render_title.read())(dragging_tile.clone());

    rsx! {
        div {
            class: "drag-ghost",
            style: "
                position: fixed;
                left: {offset_x}px;
                top: {offset_y}px;
                width: 200px;
                height: 120px;
                background-color: #1a1d24;
                border: 2px solid #3b82f6;
                border-radius: 8px;
                box-shadow: 0 8px 16px rgba(0, 0, 0, 0.3);
                opacity: 0.85;
                pointer-events: none;
                z-index: 9999;
                display: flex;
                flex-direction: column;
                overflow: hidden;
            ",

            // Ghost header
            div {
                style: "
                    padding: 0.5rem 0.75rem;
                    background-color: #14161c;
                    border-bottom: 1px solid #2a2f3a;
                    font-size: 0.875rem;
                    font-weight: 600;
                    color: #ffffff;
                ",
                {title}
            }

            // Ghost content placeholder
            div {
                style: "
                    flex: 1;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    color: #888;
                    font-size: 0.75rem;
                ",
                "Dragging..."
            }
        }
    }
}
