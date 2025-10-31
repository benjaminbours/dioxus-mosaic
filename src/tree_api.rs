use crate::layout::MosaicLayout;
use crate::node::Node;
use crate::types::{NodeId, SplitDirection, TileId};
use serde::{Deserialize, Serialize};

/// Tree representation for external API
///
/// This provides a simple, tree-like structure for defining layouts,
/// which is then converted to the internal HashMap representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MosaicNode {
    /// A split containing two child nodes
    Split {
        direction: SplitDirection,
        first: Box<MosaicNode>,
        second: Box<MosaicNode>,
        split_percentage: f64,
    },
    /// A leaf node containing a tile
    Leaf(TileId),
}

impl MosaicNode {
    /// Create a horizontal split
    pub fn horizontal(first: MosaicNode, second: MosaicNode, split_percentage: f64) -> Self {
        MosaicNode::Split {
            direction: SplitDirection::Horizontal,
            first: Box::new(first),
            second: Box::new(second),
            split_percentage,
        }
    }

    /// Create a vertical split
    pub fn vertical(first: MosaicNode, second: MosaicNode, split_percentage: f64) -> Self {
        MosaicNode::Split {
            direction: SplitDirection::Vertical,
            first: Box::new(first),
            second: Box::new(second),
            split_percentage,
        }
    }

    /// Create a leaf tile
    pub fn tile(tile_id: impl Into<TileId>) -> Self {
        MosaicNode::Leaf(tile_id.into())
    }
}

/// Helper function for creating leaf tiles
pub fn tile(tile_id: impl Into<TileId>) -> MosaicNode {
    MosaicNode::tile(tile_id)
}

impl MosaicLayout {
    /// Create a layout from a tree representation
    pub fn from_tree(tree: MosaicNode) -> Self {
        let mut layout = match &tree {
            MosaicNode::Leaf(tile_id) => MosaicLayout::new(tile_id.clone()),
            MosaicNode::Split { .. } => {
                // Start with a dummy tile, we'll replace it
                MosaicLayout::new("__temp__".to_string())
            }
        };

        if let MosaicNode::Split { .. } = tree {
            let new_root = layout.insert_tree_recursive(&tree, None);
            layout.set_root(new_root);
            // Remove the temp tile
            layout.remove_temp_tiles();
        }

        layout
    }

    /// Recursively insert a tree node and return its ID
    fn insert_tree_recursive(
        &mut self,
        tree_node: &MosaicNode,
        parent_id: Option<String>,
    ) -> String {
        match tree_node {
            MosaicNode::Leaf(tile_id) => {
                let node_id = self.gen_id();
                let node = Node::Tile {
                    id: node_id.clone(),
                    tile_id: tile_id.clone(),
                    parent: parent_id,
                    locked: false,
                };
                self.insert_node(node_id.clone(), node);
                node_id
            }
            MosaicNode::Split {
                direction,
                first,
                second,
                split_percentage,
            } => {
                let node_id = self.gen_id();

                // Recursively insert children first
                let first_id = self.insert_tree_recursive(first, Some(node_id.clone()));
                let second_id = self.insert_tree_recursive(second, Some(node_id.clone()));

                let node = Node::Split {
                    id: node_id.clone(),
                    direction: *direction,
                    first: first_id,
                    second: second_id,
                    split_percentage: *split_percentage,
                    parent: parent_id,
                    locked: false,
                    min_percentage: 20.0,
                    max_percentage: 80.0,
                };
                self.insert_node(node_id.clone(), node);
                node_id
            }
        }
    }

    /// Convert layout to tree representation
    /// Returns None if the layout is empty
    pub fn to_tree(&self) -> Option<MosaicNode> {
        self.root().map(|root_id| self.node_to_tree(root_id))
    }

    /// Recursively convert a node to tree representation
    fn node_to_tree(&self, node_id: &NodeId) -> MosaicNode {
        match self.get_node(node_id) {
            Some(Node::Tile { tile_id, .. }) => MosaicNode::Leaf(tile_id.clone()),
            Some(Node::Split {
                direction,
                first,
                second,
                split_percentage,
                ..
            }) => {
                let first_tree = self.node_to_tree(&first);
                let second_tree = self.node_to_tree(&second);
                MosaicNode::Split {
                    direction: direction.clone(),
                    first: Box::new(first_tree),
                    second: Box::new(second_tree),
                    split_percentage: split_percentage.clone(),
                }
            }
            None => MosaicNode::Leaf("error".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_to_layout_simple() {
        let tree = MosaicNode::tile("tile1");
        let layout = MosaicLayout::from_tree(tree);
        assert_eq!(layout.get_all_tiles(), vec!["tile1".to_string()]);
    }

    #[test]
    fn test_tree_to_layout_split() {
        let tree = MosaicNode::horizontal(
            MosaicNode::tile("tile1"),
            MosaicNode::tile("tile2"),
            50.0,
        );
        let layout = MosaicLayout::from_tree(tree);
        assert_eq!(
            layout.get_all_tiles(),
            vec!["tile1".to_string(), "tile2".to_string()]
        );
    }

    #[test]
    fn test_tree_to_layout_nested() {
        let tree = MosaicNode::horizontal(
            MosaicNode::tile("tile1"),
            MosaicNode::vertical(
                MosaicNode::tile("tile2"),
                MosaicNode::tile("tile3"),
                60.0,
            ),
            40.0,
        );
        let layout = MosaicLayout::from_tree(tree);
        assert_eq!(
            layout.get_all_tiles(),
            vec![
                "tile1".to_string(),
                "tile2".to_string(),
                "tile3".to_string()
            ]
        );
    }

    #[test]
    fn test_layout_to_tree() {
        let tree = MosaicNode::horizontal(
            MosaicNode::tile("tile1"),
            MosaicNode::tile("tile2"),
            50.0,
        );
        let layout = MosaicLayout::from_tree(tree.clone());
        let tree2 = layout.to_tree().expect("Layout should not be empty");

        // Both layouts should have the same tiles
        let layout2 = MosaicLayout::from_tree(tree2);
        assert_eq!(layout.get_all_tiles(), layout2.get_all_tiles());
    }

    #[test]
    fn test_empty_layout_to_tree() {
        let layout = MosaicLayout::empty();
        let tree = layout.to_tree();
        assert!(tree.is_none());
    }
}
