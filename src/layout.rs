use crate::node::Node;
use crate::types::{NodeId, SplitDirection, TileId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main layout structure using HashMap for O(1) operations
///
/// This is the core data structure that manages the mosaic layout.
/// Internally uses a HashMap for fast lookups, but provides a tree-like API for ease of use.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MosaicLayout {
    /// All nodes indexed by ID - O(1) access
    nodes: HashMap<NodeId, Node>,

    /// Root node ID (None when layout is empty)
    root: Option<NodeId>,

    /// Counter for generating unique node IDs
    next_id: usize,
}

impl MosaicLayout {
    /// Create a new empty layout with a single tile
    pub fn new(tile_id: TileId) -> Self {
        let mut nodes = HashMap::new();
        let root_id = "node_0".to_string();

        let root_node = Node::Tile {
            id: root_id.clone(),
            tile_id,
            parent: None,
            locked: false,
        };

        nodes.insert(root_id.clone(), root_node);

        Self {
            nodes,
            root: Some(root_id),
            next_id: 1,
        }
    }

    /// Create a completely empty layout with no tiles
    pub fn empty() -> Self {
        Self {
            nodes: HashMap::new(),
            root: None,
            next_id: 0,
        }
    }

    /// Check if the layout is empty (has no tiles)
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    /// Generate a new unique node ID
    pub(crate) fn gen_id(&mut self) -> NodeId {
        let id = format!("node_{}", self.next_id);
        self.next_id += 1;
        id
    }

    /// Get the root node ID
    pub fn root(&self) -> Option<&NodeId> {
        self.root.as_ref()
    }

    /// Set the root node ID (used internally by tree API)
    pub(crate) fn set_root(&mut self, new_root: NodeId) {
        self.root = Some(new_root);
    }

    /// Remove temporary tiles (used internally by tree API)
    pub(crate) fn remove_temp_tiles(&mut self) {
        self.nodes.retain(|_, node| match node {
            Node::Tile { tile_id, .. } => tile_id != "__temp__",
            _ => true,
        });
    }

    /// Insert a node into the layout (used internally by tree API)
    pub(crate) fn insert_node(&mut self, node_id: NodeId, node: Node) {
        self.nodes.insert(node_id, node);
    }

    /// Get a node by ID
    pub fn get_node(&self, node_id: &NodeId) -> Option<&Node> {
        self.nodes.get(node_id)
    }

    /// Get a mutable reference to a node by ID
    pub fn get_node_mut(&mut self, node_id: &NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(node_id)
    }

    /// Find a node by tile ID - O(n) but only used for user actions
    ///
    /// Returns the NodeId of the Tile node containing this tile_id
    pub fn find_tile(&self, tile_id: &TileId) -> Option<NodeId> {
        self.nodes.values().find_map(|node| match node {
            Node::Tile {
                id, tile_id: tid, ..
            } if tid == tile_id => Some(id.clone()),
            _ => None,
        })
    }

    /// Update split percentage - O(1)
    ///
    /// Clamps the percentage to the node's min/max values
    pub fn update_split(&mut self, node_id: &NodeId, percentage: f64) -> bool {
        if let Some(Node::Split {
            split_percentage,
            min_percentage,
            max_percentage,
            locked,
            ..
        }) = self.nodes.get_mut(node_id)
        {
            if *locked {
                return false;
            }
            *split_percentage = percentage.clamp(*min_percentage, *max_percentage);
            true
        } else {
            false
        }
    }

    /// Split a tile into two panes - O(1)
    ///
    /// Replaces the tile node with a split node containing the original tile and a new tile
    pub fn split_tile(
        &mut self,
        tile_id: &TileId,
        direction: SplitDirection,
        new_tile_id: TileId,
        split_percentage: f64,
    ) -> bool {
        // Find the tile node
        let tile_node_id = match self.find_tile(tile_id) {
            Some(id) => id,
            None => return false,
        };

        // Get the parent ID before we modify anything
        let parent_id = self
            .nodes
            .get(&tile_node_id)
            .and_then(|n| n.parent().cloned());

        // Create new tile node
        let new_tile_node_id = self.gen_id();
        let new_tile_node = Node::Tile {
            id: new_tile_node_id.clone(),
            tile_id: new_tile_id,
            parent: None, // Will be set below
            locked: false,
        };

        // Create split node
        let split_node_id = self.gen_id();
        let split_node = Node::Split {
            id: split_node_id.clone(),
            direction,
            first: tile_node_id.clone(),
            second: new_tile_node_id.clone(),
            split_percentage: split_percentage.clamp(20.0, 80.0),
            parent: parent_id.clone(),
            locked: false,
            min_percentage: 20.0,
            max_percentage: 80.0,
        };

        // Insert new nodes
        self.nodes.insert(new_tile_node_id.clone(), new_tile_node);
        self.nodes.insert(split_node_id.clone(), split_node);

        // Update the original tile's parent to point to the new split
        if let Some(tile_node) = self.nodes.get_mut(&tile_node_id) {
            tile_node.set_parent(Some(split_node_id.clone()));
        }

        // Update the new tile's parent to point to the new split
        if let Some(new_tile_node) = self.nodes.get_mut(&new_tile_node_id) {
            new_tile_node.set_parent(Some(split_node_id.clone()));
        }

        // Update parent's child pointer or root
        if let Some(parent_id) = parent_id {
            self.replace_child(&parent_id, &tile_node_id, &split_node_id);
        } else {
            // This was the root
            self.root = Some(split_node_id);
        }

        true
    }

    /// Close a tile - O(1)
    ///
    /// Removes the tile and its parent split, promoting the sibling.
    /// If this is the last tile, the layout becomes empty.
    pub fn close_tile(&mut self, tile_id: &TileId) -> bool {
        // Find the tile node
        let tile_node_id = match self.find_tile(tile_id) {
            Some(id) => id,
            None => return false,
        };

        // Check if locked
        if let Some(Node::Tile { locked: true, .. }) = self.nodes.get(&tile_node_id) {
            return false;
        }

        // Get parent (the split node)
        let parent_id = match self.nodes.get(&tile_node_id).and_then(|n| n.parent()) {
            Some(id) => id.clone(),
            None => {
                // This is the root and only tile - close it to create empty layout
                self.nodes.remove(&tile_node_id);
                self.root = None;
                return true;
            }
        };

        // Get the sibling node ID
        let sibling_id = match self.nodes.get(&parent_id) {
            Some(Node::Split { first, second, .. }) => {
                if first == &tile_node_id {
                    second.clone()
                } else {
                    first.clone()
                }
            }
            _ => return false,
        };

        // Get the grandparent ID
        let grandparent_id = self.nodes.get(&parent_id).and_then(|n| n.parent().cloned());

        // Update sibling's parent to grandparent
        if let Some(sibling_node) = self.nodes.get_mut(&sibling_id) {
            sibling_node.set_parent(grandparent_id.clone());
        }

        // Update grandparent's child pointer or root
        if let Some(gp_id) = grandparent_id {
            self.replace_child(&gp_id, &parent_id, &sibling_id);
        } else {
            // Parent was root, sibling becomes new root
            self.root = Some(sibling_id);
        }

        // Remove tile and parent split nodes
        self.nodes.remove(&tile_node_id);
        self.nodes.remove(&parent_id);

        true
    }

    /// Replace a child node in a split
    fn replace_child(&mut self, parent_id: &NodeId, old_child: &NodeId, new_child: &NodeId) {
        if let Some(Node::Split { first, second, .. }) = self.nodes.get_mut(parent_id) {
            if first == old_child {
                *first = new_child.clone();
            } else if second == old_child {
                *second = new_child.clone();
            }
        }
    }

    /// Insert a tile by splitting a target tile
    ///
    /// This is used for drag-and-drop operations. It removes the dragged tile from its
    /// current position and inserts it by splitting the target tile in the specified direction.
    ///
    /// Returns true if the operation succeeded, false if either tile wasn't found or
    /// if trying to drop a tile onto itself.
    pub fn insert_tile_with_split(
        &mut self,
        dragged_tile_id: &TileId,
        target_tile_id: &TileId,
        drop_zone: crate::drag_drop::DropZone,
    ) -> bool {
        // Don't allow dropping on itself
        if dragged_tile_id == target_tile_id {
            return false;
        }

        // Find both tile nodes
        let dragged_node_id = match self.find_tile(dragged_tile_id) {
            Some(id) => id,
            None => return false,
        };

        let target_node_id = match self.find_tile(target_tile_id) {
            Some(id) => id,
            None => return false,
        };

        // Check if target is locked
        if let Some(Node::Tile { locked: true, .. }) = self.nodes.get(&target_node_id) {
            return false;
        }

        // Step 1: Remove dragged tile from its current position (but keep the node)
        // We need to preserve the dragged tile node and its data
        let dragged_tile_node = match self.nodes.get(&dragged_node_id).cloned() {
            Some(Node::Tile { tile_id, locked, .. }) => Node::Tile {
                id: dragged_node_id.clone(),
                tile_id,
                parent: None, // Will be updated below
                locked,
            },
            _ => return false,
        };

        // Get parent of dragged tile (before removing it)
        let dragged_parent_id = self
            .nodes
            .get(&dragged_node_id)
            .and_then(|n| n.parent().cloned());

        // If dragged tile has a parent (it's in a split), remove it properly
        if let Some(parent_id) = dragged_parent_id {
            // Get sibling node ID
            let sibling_id = match self.nodes.get(&parent_id) {
                Some(Node::Split { first, second, .. }) => {
                    if first == &dragged_node_id {
                        second.clone()
                    } else {
                        first.clone()
                    }
                }
                _ => return false,
            };

            // Get grandparent ID
            let grandparent_id = self.nodes.get(&parent_id).and_then(|n| n.parent().cloned());

            // Update sibling's parent to grandparent
            if let Some(sibling_node) = self.nodes.get_mut(&sibling_id) {
                sibling_node.set_parent(grandparent_id.clone());
            }

            // Update grandparent's child pointer or root
            if let Some(gp_id) = grandparent_id {
                self.replace_child(&gp_id, &parent_id, &sibling_id);
            } else {
                // Parent was root, sibling becomes new root
                self.root = Some(sibling_id);
            }

            // Remove parent split node and dragged tile node
            self.nodes.remove(&parent_id);
            self.nodes.remove(&dragged_node_id);
        } else {
            // Dragged tile was root - just remove it for now
            self.nodes.remove(&dragged_node_id);
            self.root = None; // Will be updated below
        }

        // Step 2: Split the target tile and insert the dragged tile
        let target_parent_id = self
            .nodes
            .get(&target_node_id)
            .and_then(|n| n.parent().cloned());

        // Create new split node
        let split_node_id = self.gen_id();
        let direction = drop_zone.split_direction();
        let dragged_is_first = drop_zone.dragged_is_first();

        let (first, second) = if dragged_is_first {
            (dragged_node_id.clone(), target_node_id.clone())
        } else {
            (target_node_id.clone(), dragged_node_id.clone())
        };

        let split_node = Node::Split {
            id: split_node_id.clone(),
            direction,
            first,
            second,
            split_percentage: 50.0, // Default 50/50 split
            parent: target_parent_id.clone(),
            locked: false,
            min_percentage: 20.0,
            max_percentage: 80.0,
        };

        // Insert split node
        self.nodes.insert(split_node_id.clone(), split_node);

        // Update dragged tile node with new parent
        let mut updated_dragged_node = dragged_tile_node;
        updated_dragged_node.set_parent(Some(split_node_id.clone()));
        self.nodes.insert(dragged_node_id.clone(), updated_dragged_node);

        // Update target tile's parent
        if let Some(target_node) = self.nodes.get_mut(&target_node_id) {
            target_node.set_parent(Some(split_node_id.clone()));
        }

        // Update parent's child pointer or root
        if let Some(parent_id) = target_parent_id {
            self.replace_child(&parent_id, &target_node_id, &split_node_id);
        } else {
            // Target was root, split becomes new root
            self.root = Some(split_node_id);
        }

        true
    }

    /// Get all tile IDs in the layout (in traversal order)
    pub fn get_all_tiles(&self) -> Vec<TileId> {
        let mut tiles = Vec::new();
        if let Some(root_id) = &self.root {
            self.collect_tiles(root_id, &mut tiles);
        }
        tiles
    }

    /// Recursively collect tile IDs
    fn collect_tiles(&self, node_id: &NodeId, tiles: &mut Vec<TileId>) {
        if let Some(node) = self.nodes.get(node_id) {
            match node {
                Node::Tile { tile_id, .. } => {
                    tiles.push(tile_id.clone());
                }
                Node::Split { first, second, .. } => {
                    self.collect_tiles(first, tiles);
                    self.collect_tiles(second, tiles);
                }
            }
        }
    }
}

impl Default for MosaicLayout {
    fn default() -> Self {
        Self::new("default".to_string())
    }
}

impl MosaicLayout {
    /// Save layout to localStorage
    ///
    /// Serializes the layout to JSON and stores it in localStorage.
    /// Uses the provided key for storage.
    pub fn save_to_storage(&self, storage_key: &str) -> Result<(), String> {
        let json = serde_json::to_string(self).map_err(|e| e.to_string())?;

        web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .ok_or_else(|| "localStorage not available".to_string())?
            .set_item(storage_key, &json)
            .map_err(|_| "Failed to save to localStorage".to_string())
    }

    /// Load layout from localStorage
    ///
    /// Attempts to load and deserialize a layout from localStorage.
    /// Returns None if the key doesn't exist or deserialization fails.
    pub fn load_from_storage(storage_key: &str) -> Option<Self> {
        web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .and_then(|storage| storage.get_item(storage_key).ok().flatten())
            .and_then(|json_string| serde_json::from_str::<Self>(&json_string).ok())
    }

    /// Clear layout from localStorage
    pub fn clear_storage(storage_key: &str) -> Result<(), String> {
        web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .ok_or_else(|| "localStorage not available".to_string())?
            .remove_item(storage_key)
            .map_err(|_| "Failed to remove from localStorage".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_layout() {
        let layout = MosaicLayout::new("tile1".to_string());
        assert_eq!(layout.nodes.len(), 1);
        assert_eq!(layout.get_all_tiles(), vec!["tile1".to_string()]);
    }

    #[test]
    fn test_split_tile() {
        let mut layout = MosaicLayout::new("tile1".to_string());
        let success = layout.split_tile(
            &"tile1".to_string(),
            SplitDirection::Horizontal,
            "tile2".to_string(),
            50.0,
        );
        assert!(success);
        assert_eq!(layout.nodes.len(), 3); // original tile + new tile + split
        assert_eq!(
            layout.get_all_tiles(),
            vec!["tile1".to_string(), "tile2".to_string()]
        );
    }

    #[test]
    fn test_close_tile() {
        let mut layout = MosaicLayout::new("tile1".to_string());
        layout.split_tile(
            &"tile1".to_string(),
            SplitDirection::Horizontal,
            "tile2".to_string(),
            50.0,
        );

        let success = layout.close_tile(&"tile2".to_string());
        assert!(success);
        assert_eq!(layout.nodes.len(), 1); // only tile1 remains
        assert_eq!(layout.get_all_tiles(), vec!["tile1".to_string()]);
    }

    #[test]
    fn test_can_close_last_tile_creates_empty_layout() {
        let mut layout = MosaicLayout::new("tile1".to_string());
        let success = layout.close_tile(&"tile1".to_string());
        assert!(success);
        assert_eq!(layout.nodes.len(), 0);
        assert!(layout.is_empty());
        assert_eq!(layout.get_all_tiles(), Vec::<String>::new());
    }

    #[test]
    fn test_empty_layout() {
        let layout = MosaicLayout::empty();
        assert!(layout.is_empty());
        assert_eq!(layout.nodes.len(), 0);
        assert_eq!(layout.get_all_tiles(), Vec::<String>::new());
    }

    #[test]
    fn test_update_split() {
        let mut layout = MosaicLayout::new("tile1".to_string());
        layout.split_tile(
            &"tile1".to_string(),
            SplitDirection::Horizontal,
            "tile2".to_string(),
            50.0,
        );

        // Find the split node
        let split_id = layout.root().unwrap().clone();
        let success = layout.update_split(&split_id, 60.0);
        assert!(success);

        if let Some(Node::Split {
            split_percentage, ..
        }) = layout.get_node(&split_id)
        {
            assert_eq!(*split_percentage, 60.0);
        } else {
            panic!("Root should be a split node");
        }
    }
}
