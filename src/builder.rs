use crate::layout::MosaicLayout;
use crate::tree_api::MosaicNode;
use crate::types::{SplitDirection, TileId};

/// Builder for creating mosaic layouts with a fluent API
///
/// This provides a clean, readable way to define layouts.
///
/// # Example
/// ```ignore
/// let layout = MosaicBuilder::horizontal()
///     .left(tile("sidebar"))
///     .right(
///         MosaicBuilder::vertical()
///             .top(tile("editor"))
///             .bottom(tile("terminal"))
///             .split(70.0)
///             .build()
///     )
///     .split(25.0)
///     .build();
/// ```
pub struct MosaicBuilder {
    direction: SplitDirection,
    first: Option<MosaicNode>,
    second: Option<MosaicNode>,
    split_percentage: f64,
}

impl MosaicBuilder {
    /// Create a horizontal split builder (left | right)
    pub fn horizontal() -> Self {
        Self {
            direction: SplitDirection::Horizontal,
            first: None,
            second: None,
            split_percentage: 50.0,
        }
    }

    /// Create a vertical split builder (top | bottom)
    pub fn vertical() -> Self {
        Self {
            direction: SplitDirection::Vertical,
            first: None,
            second: None,
            split_percentage: 50.0,
        }
    }

    /// Set the first child (left for horizontal, top for vertical)
    pub fn first(mut self, node: MosaicNode) -> Self {
        self.first = Some(node);
        self
    }

    /// Set the second child (right for horizontal, bottom for vertical)
    pub fn second(mut self, node: MosaicNode) -> Self {
        self.second = Some(node);
        self
    }

    /// Set the left child (alias for first in horizontal splits)
    pub fn left(self, node: MosaicNode) -> Self {
        self.first(node)
    }

    /// Set the right child (alias for second in horizontal splits)
    pub fn right(self, node: MosaicNode) -> Self {
        self.second(node)
    }

    /// Set the top child (alias for first in vertical splits)
    pub fn top(self, node: MosaicNode) -> Self {
        self.first(node)
    }

    /// Set the bottom child (alias for second in vertical splits)
    pub fn bottom(self, node: MosaicNode) -> Self {
        self.second(node)
    }

    /// Set the split percentage (0-100)
    pub fn split(mut self, percentage: f64) -> Self {
        self.split_percentage = percentage.clamp(0.0, 100.0);
        self
    }

    /// Build the MosaicNode tree
    pub fn build_tree(self) -> MosaicNode {
        let first = self.first.expect("First child not set");
        let second = self.second.expect("Second child not set");

        MosaicNode::Split {
            direction: self.direction,
            first: Box::new(first),
            second: Box::new(second),
            split_percentage: self.split_percentage,
        }
    }

    /// Build the MosaicLayout directly
    pub fn build(self) -> MosaicLayout {
        let tree = self.build_tree();
        MosaicLayout::from_tree(tree)
    }
}

/// Create a tile node (helper function)
pub fn tile(tile_id: impl Into<TileId>) -> MosaicNode {
    MosaicNode::Leaf(tile_id.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_horizontal() {
        let layout = MosaicBuilder::horizontal()
            .left(tile("a"))
            .right(tile("b"))
            .split(40.0)
            .build();

        assert_eq!(layout.get_all_tiles(), vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn test_builder_vertical() {
        let layout = MosaicBuilder::vertical()
            .top(tile("a"))
            .bottom(tile("b"))
            .split(60.0)
            .build();

        assert_eq!(layout.get_all_tiles(), vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn test_builder_nested() {
        let layout = MosaicBuilder::horizontal()
            .left(tile("sidebar"))
            .right(
                MosaicBuilder::vertical()
                    .top(tile("editor"))
                    .bottom(tile("terminal"))
                    .split(70.0)
                    .build_tree(),
            )
            .split(25.0)
            .build();

        assert_eq!(
            layout.get_all_tiles(),
            vec![
                "sidebar".to_string(),
                "editor".to_string(),
                "terminal".to_string()
            ]
        );
    }
}
