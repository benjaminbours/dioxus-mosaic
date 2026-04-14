use crate::layout::MosaicLayout;
use crate::node::Node;
use crate::types::{NodeId, SplitDirection, TileId};

/// Pixel rectangle for a tile, computed from the layout tree.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TileRect {
    pub tile_id: TileId,
    pub node_id: NodeId,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub locked: bool,
}

/// Pixel rectangle for a resize divider between two split children.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DividerRect {
    pub split_node_id: NodeId,
    pub direction: SplitDirection,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Active resize state, held at the mosaic root while a divider is being dragged.
#[derive(Debug, Clone, Default)]
pub(crate) struct ActiveResize {
    pub split_node_id: Option<NodeId>,
    pub direction: Option<SplitDirection>,
    /// Pixel origin of the parent region along the split axis (used to compute new percentage).
    pub origin_px: f64,
    /// Pixel extent of the parent region along the split axis.
    pub extent_px: f64,
    pub min_pct: f64,
    pub max_pct: f64,
}

impl ActiveResize {
    pub fn is_active(&self) -> bool {
        self.split_node_id.is_some()
    }

    pub fn start(
        split_node_id: NodeId,
        direction: SplitDirection,
        origin_px: f64,
        extent_px: f64,
        min_pct: f64,
        max_pct: f64,
    ) -> Self {
        Self {
            split_node_id: Some(split_node_id),
            direction: Some(direction),
            origin_px,
            extent_px,
            min_pct,
            max_pct,
        }
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn compute_percentage(&self, mouse_pos: f64) -> f64 {
        if self.extent_px <= 0.0 {
            return 50.0;
        }
        let pct = ((mouse_pos - self.origin_px) / self.extent_px * 100.0)
            .clamp(self.min_pct, self.max_pct);
        pct
    }
}

/// Compute flat tile rects and divider rects from a layout tree.
///
/// `gap_px` is the fixed pixel gap on *each side* of a divider (total gap = divider_width + 2 * gap_px).
/// `divider_px` is the pixel width of the divider handle itself.
pub(crate) fn compute_rects(
    layout: &MosaicLayout,
    container_width: f64,
    container_height: f64,
    gap_px: f64,
    divider_px: f64,
) -> (Vec<TileRect>, Vec<DividerRect>) {
    let mut tiles = Vec::new();
    let mut dividers = Vec::new();

    if let Some(root_id) = layout.root() {
        compute_node(
            layout,
            root_id,
            0.0,
            0.0,
            container_width,
            container_height,
            gap_px,
            divider_px,
            &mut tiles,
            &mut dividers,
        );
    }

    (tiles, dividers)
}

fn compute_node(
    layout: &MosaicLayout,
    node_id: &NodeId,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    gap_px: f64,
    divider_px: f64,
    tiles: &mut Vec<TileRect>,
    dividers: &mut Vec<DividerRect>,
) {
    let node = match layout.get_node(node_id) {
        Some(n) => n.clone(),
        None => return,
    };

    match node {
        Node::Tile {
            id,
            tile_id,
            locked,
            ..
        } => {
            tiles.push(TileRect {
                tile_id,
                node_id: id,
                x,
                y,
                width: w,
                height: h,
                locked,
            });
        }
        Node::Split {
            id,
            direction,
            first,
            second,
            split_percentage,
            ..
        } => {
            // gutter = gap + divider + gap
            let gutter = gap_px + divider_px + gap_px;

            match direction {
                SplitDirection::Horizontal => {
                    let available = (w - gutter).max(0.0);
                    let first_w = available * split_percentage / 100.0;
                    let second_w = available - first_w;

                    let divider_x = x + first_w + gap_px;
                    dividers.push(DividerRect {
                        split_node_id: id,
                        direction,
                        x: divider_x,
                        y,
                        width: divider_px,
                        height: h,
                    });

                    compute_node(
                        layout, &first, x, y, first_w, h, gap_px, divider_px, tiles, dividers,
                    );
                    let second_x = divider_x + divider_px + gap_px;
                    compute_node(
                        layout, &second, second_x, y, second_w, h, gap_px, divider_px, tiles,
                        dividers,
                    );
                }
                SplitDirection::Vertical => {
                    let available = (h - gutter).max(0.0);
                    let first_h = available * split_percentage / 100.0;
                    let second_h = available - first_h;

                    let divider_y = y + first_h + gap_px;
                    dividers.push(DividerRect {
                        split_node_id: id,
                        direction,
                        x,
                        y: divider_y,
                        width: w,
                        height: divider_px,
                    });

                    compute_node(
                        layout, &first, x, y, w, first_h, gap_px, divider_px, tiles, dividers,
                    );
                    let second_y = divider_y + divider_px + gap_px;
                    compute_node(
                        layout, &second, x, second_y, w, second_h, gap_px, divider_px, tiles,
                        dividers,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_tile_fills_container() {
        let layout = MosaicLayout::new("only".into());
        let (tiles, dividers) = compute_rects(&layout, 800.0, 600.0, 3.0, 4.0);

        assert_eq!(tiles.len(), 1);
        assert_eq!(dividers.len(), 0);

        let t = &tiles[0];
        assert_eq!(t.tile_id, "only");
        assert!((t.x - 0.0).abs() < f64::EPSILON);
        assert!((t.y - 0.0).abs() < f64::EPSILON);
        assert!((t.width - 800.0).abs() < f64::EPSILON);
        assert!((t.height - 600.0).abs() < f64::EPSILON);
    }

    #[test]
    fn horizontal_split_produces_two_tiles_one_divider() {
        let mut layout = MosaicLayout::new("left".into());
        layout.split_tile(
            &"left".into(),
            SplitDirection::Horizontal,
            "right".into(),
            50.0,
        );

        let gap = 3.0;
        let div = 4.0;
        let (tiles, dividers) = compute_rects(&layout, 800.0, 600.0, gap, div);

        assert_eq!(tiles.len(), 2);
        assert_eq!(dividers.len(), 1);

        let gutter = gap + div + gap; // 10px total
        let available = 800.0 - gutter;
        let left_w = available * 0.5;
        let right_w = available - left_w;

        let left_tile = tiles.iter().find(|t| t.tile_id == "left").unwrap();
        assert!((left_tile.x - 0.0).abs() < 0.01);
        assert!((left_tile.width - left_w).abs() < 0.01);

        let right_tile = tiles.iter().find(|t| t.tile_id == "right").unwrap();
        let expected_right_x = left_w + gutter;
        assert!((right_tile.x - expected_right_x).abs() < 0.01);
        assert!((right_tile.width - right_w).abs() < 0.01);

        let d = &dividers[0];
        assert_eq!(d.direction, SplitDirection::Horizontal);
        assert!((d.x - (left_w + gap)).abs() < 0.01);
        assert!((d.width - div).abs() < 0.01);
        assert!((d.height - 600.0).abs() < 0.01);
    }

    #[test]
    fn nested_split_produces_correct_rects() {
        let mut layout = MosaicLayout::new("a".into());
        layout.split_tile(&"a".into(), SplitDirection::Horizontal, "b".into(), 50.0);
        layout.split_tile(&"b".into(), SplitDirection::Vertical, "c".into(), 50.0);

        let (tiles, dividers) = compute_rects(&layout, 1000.0, 800.0, 3.0, 4.0);

        assert_eq!(tiles.len(), 3);
        assert_eq!(dividers.len(), 2);

        for t in &tiles {
            assert!(
                t.width > 0.0,
                "tile {} has non-positive width {}",
                t.tile_id,
                t.width
            );
            assert!(
                t.height > 0.0,
                "tile {} has non-positive height {}",
                t.tile_id,
                t.height
            );
        }
    }
}
