use crate::types::{NodeId, SplitDirection, TileId};
use serde::{Deserialize, Serialize};

/// A node in the mosaic layout
///
/// Each node is either a Split (containing two child nodes) or a Tile (leaf node with content).
/// Nodes are stored in a HashMap for O(1) access.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Node {
    /// A split node containing two children
    Split {
        /// Unique identifier for this node
        id: NodeId,

        /// Direction of the split (horizontal or vertical)
        direction: SplitDirection,

        /// ID of the first child node
        first: NodeId,

        /// ID of the second child node
        second: NodeId,

        /// Split position as percentage (0.0 - 100.0) for the first pane
        split_percentage: f64,

        /// Parent node ID (None for root)
        parent: Option<NodeId>,

        // Metadata
        /// Whether this split is locked (prevents resizing)
        locked: bool,

        /// Minimum percentage for the first pane
        min_percentage: f64,

        /// Maximum percentage for the first pane
        max_percentage: f64,
    },

    /// A leaf node containing a tile
    Tile {
        /// Unique identifier for this node
        id: NodeId,

        /// ID of the tile content (used to render)
        tile_id: TileId,

        /// Parent node ID (None for root)
        parent: Option<NodeId>,

        // Metadata
        /// Whether this tile is locked (prevents closing)
        locked: bool,
    },
}

impl Node {
    /// Get the node's ID
    pub fn id(&self) -> &NodeId {
        match self {
            Node::Split { id, .. } => id,
            Node::Tile { id, .. } => id,
        }
    }

    /// Get the node's parent ID
    pub fn parent(&self) -> Option<&NodeId> {
        match self {
            Node::Split { parent, .. } => parent.as_ref(),
            Node::Tile { parent, .. } => parent.as_ref(),
        }
    }

    /// Set the node's parent ID
    pub fn set_parent(&mut self, new_parent: Option<NodeId>) {
        match self {
            Node::Split { parent, .. } => *parent = new_parent,
            Node::Tile { parent, .. } => *parent = new_parent,
        }
    }

    /// Check if this is a split node
    pub fn is_split(&self) -> bool {
        matches!(self, Node::Split { .. })
    }

    /// Check if this is a tile node
    pub fn is_tile(&self) -> bool {
        matches!(self, Node::Tile { .. })
    }

    /// Get child node IDs (for Split nodes only)
    pub fn children(&self) -> Option<(&NodeId, &NodeId)> {
        match self {
            Node::Split { first, second, .. } => Some((first, second)),
            Node::Tile { .. } => None,
        }
    }
}
