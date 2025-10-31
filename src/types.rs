use serde::{Deserialize, Serialize};

/// Unique identifier for a node in the mosaic layout
pub type NodeId = String;

/// Unique identifier for a tile (user-defined content panel)
pub type TileId = String;

/// Direction of a split
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SplitDirection {
    /// Left | Right split
    Horizontal,
    /// Top | Bottom split
    Vertical,
}

impl SplitDirection {
    /// Returns the opposite direction
    pub fn opposite(&self) -> Self {
        match self {
            SplitDirection::Horizontal => SplitDirection::Vertical,
            SplitDirection::Vertical => SplitDirection::Horizontal,
        }
    }
}
