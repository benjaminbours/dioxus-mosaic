use crate::drag_drop::{DragGhost, DragState, ResizeState, TileRect as DragTileRect, TileRefs};
use crate::flat_layout::{compute_rects, ActiveResize, DividerRect, TileRect};
use crate::layout::MosaicLayout;
use crate::node::Node;
use crate::tile_pane::TilePane;
use crate::types::{NodeId, SplitDirection, TileId};
use dioxus::prelude::*;
use std::collections::HashMap;

const GAP_PX: f64 = 6.0;
const DIVIDER_PX: f64 = 4.0;

#[derive(Clone, Default, PartialEq)]
struct ContainerRect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl ContainerRect {
    fn has_size(&self) -> bool {
        self.width > 0.0 && self.height > 0.0
    }
}

#[derive(Clone, Default, PartialEq)]
struct MeasureKey {
    root_id: Option<NodeId>,
    tile_count: usize,
}

#[derive(Clone, Copy)]
struct Bounds {
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
}

impl Bounds {
    fn from_tile_rect(rect: &TileRect) -> Self {
        Self {
            min_x: rect.x,
            min_y: rect.y,
            max_x: rect.x + rect.width,
            max_y: rect.y + rect.height,
        }
    }

    fn union(self, other: Self) -> Self {
        Self {
            min_x: self.min_x.min(other.min_x),
            min_y: self.min_y.min(other.min_y),
            max_x: self.max_x.max(other.max_x),
            max_y: self.max_y.max(other.max_y),
        }
    }

    fn axis_origin(&self, direction: SplitDirection) -> f64 {
        match direction {
            SplitDirection::Horizontal => self.min_x,
            SplitDirection::Vertical => self.min_y,
        }
    }

    fn axis_extent(&self, direction: SplitDirection) -> f64 {
        match direction {
            SplitDirection::Horizontal => self.max_x - self.min_x,
            SplitDirection::Vertical => self.max_y - self.min_y,
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct MosaicProps {
    pub layout: Signal<MosaicLayout>,
    pub render_tile: Signal<Box<dyn Fn(TileId) -> Option<Element>>>,
    pub render_title: Signal<Box<dyn Fn(TileId) -> Element>>,
    #[props(default = None)]
    pub render_empty_state: Option<Signal<Box<dyn Fn() -> Element>>>,
}

pub fn Mosaic(props: MosaicProps) -> Element {
    let mut layout = props.layout;
    let mut drag_state = use_signal(DragState::new);
    let tile_refs = use_signal(TileRefs::new);
    let mut resize_state = use_signal(ResizeState::default);
    let mut active_resize = use_signal(ActiveResize::default);
    let mut container_ref = use_signal(|| None::<MountedEvent>);
    let container_rect = use_signal(ContainerRect::default);
    let mut is_measuring = use_signal(|| false);
    let mut last_measure_key = use_signal(MeasureKey::default);

    use_context_provider(|| layout);
    use_context_provider(|| drag_state);
    use_context_provider(|| tile_refs);
    use_context_provider(|| resize_state);
    use_context_provider(|| active_resize);
    use_context_provider(|| props.render_tile);
    use_context_provider(|| props.render_title);

    let layout_snapshot = layout.read().clone();
    let root_id = layout_snapshot.root().cloned();
    let measure_key = MeasureKey {
        root_id: root_id.clone(),
        tile_count: layout_snapshot.get_all_tiles().len(),
    };

    if let Some(mounted) = container_ref.read().clone() {
        let needs_measure =
            !container_rect.read().has_size() || *last_measure_key.read() != measure_key;
        if needs_measure && !is_measuring() {
            is_measuring.set(true);
            last_measure_key.set(measure_key.clone());
            let mut container_rect_signal = container_rect;
            let mut is_measuring_signal = is_measuring;
            spawn(async move {
                if let Ok(rect) = mounted.get_client_rect().await {
                    container_rect_signal.set(ContainerRect {
                        x: rect.origin.x,
                        y: rect.origin.y,
                        width: rect.size.width,
                        height: rect.size.height,
                    });
                }
                is_measuring_signal.set(false);
            });
        }
    }

    let container_snapshot = container_rect.read().clone();
    let (tile_rects, divider_rects) = if root_id.is_some() && container_snapshot.has_size() {
        compute_rects(
            &layout_snapshot,
            container_snapshot.width,
            container_snapshot.height,
            GAP_PX,
            DIVIDER_PX,
        )
    } else {
        (Vec::new(), Vec::new())
    };

    let split_bounds = build_split_bounds(&layout_snapshot, root_id.as_ref(), &tile_rects);
    let divider_infos: Vec<(DividerRect, f64, f64)> = divider_rects
        .iter()
        .cloned()
        .filter_map(|divider_rect| {
            split_bounds.get(&divider_rect.split_node_id).map(|bounds| {
                let page_origin = match divider_rect.direction {
                    SplitDirection::Horizontal => container_snapshot.x,
                    SplitDirection::Vertical => container_snapshot.y,
                };
                (
                    divider_rect.clone(),
                    page_origin + bounds.axis_origin(divider_rect.direction),
                    bounds.axis_extent(divider_rect.direction),
                )
            })
        })
        .collect();

    let handle_mouse_move = move |evt: Event<MouseData>| {
        let resize = active_resize.read().clone();
        if resize.is_active() {
            let mouse_pos = match resize.direction {
                Some(SplitDirection::Horizontal) => evt.page_coordinates().x as f64,
                Some(SplitDirection::Vertical) => evt.page_coordinates().y as f64,
                None => return,
            };

            if let Some(ref split_node_id) = resize.split_node_id {
                layout
                    .write()
                    .update_split(split_node_id, resize.compute_percentage(mouse_pos));
            }
            return;
        }

        if !drag_state.read().is_dragging() {
            return;
        }

        let mouse_x = evt.page_coordinates().x as f64;
        let mouse_y = evt.page_coordinates().y as f64;

        if drag_state.read().cached_rects.is_empty() && !drag_state.read().rects_fetching {
            drag_state.write().rects_fetching = true;

            let dragging_tile_id = drag_state.read().dragging_tile_id.clone();
            let refs_snapshot: Vec<_> = tile_refs
                .read()
                .refs
                .iter()
                .filter(|(tid, _)| Some(*tid) != dragging_tile_id.as_ref())
                .map(|(tid, mounted)| (tid.clone(), mounted.clone()))
                .collect();

            spawn(async move {
                let mut rects = HashMap::new();
                for (tid, mounted) in refs_snapshot {
                    if let Ok(rect) = mounted.get_client_rect().await {
                        rects.insert(
                            tid,
                            DragTileRect {
                                x: rect.origin.x,
                                y: rect.origin.y,
                                width: rect.size.width,
                                height: rect.size.height,
                            },
                        );
                    }
                }
                let mut state = drag_state.write();
                state.cached_rects = rects;
                state.rects_fetching = false;
                state.update_hover_from_cache();
            });

            drag_state.write().update_position(mouse_x, mouse_y);
            return;
        }

        let mut state = drag_state.write();
        state.update_position(mouse_x, mouse_y);
        state.update_hover_from_cache();
    };

    let handle_mouse_up = move |_evt: Event<MouseData>| {
        if active_resize.read().is_active() {
            active_resize.write().clear();
            resize_state.write().is_resizing = false;
            return;
        }

        if !drag_state.read().is_dragging() {
            return;
        }

        let dragged_tile = match drag_state.read().dragging_tile_id.clone() {
            Some(tid) => tid,
            None => return,
        };

        if let Some((target_tile, zone)) = drag_state.read().hover_target.clone() {
            if dragged_tile != target_tile {
                layout
                    .write()
                    .insert_tile_with_split(&dragged_tile, &target_tile, zone);
            }
        }

        drag_state.write().end_drag();
    };

    rsx! {
        div {
            class: "mosaic-container",
            onmounted: move |evt| {
                container_ref.set(Some(evt.clone()));
                last_measure_key.set(MeasureKey::default());
            },
            style: {
                let is_dragging = drag_state.read().is_dragging();
                let is_resizing = active_resize.read().is_active() || resize_state.read().is_resizing;
                let user_select = if is_dragging || is_resizing {
                    "user-select: none; -webkit-user-select: none;"
                } else {
                    ""
                };
                format!(
                    "width: 100%; height: 100%; position: relative; overflow: hidden; {user_select}"
                )
            },
            onmousemove: handle_mouse_move,
            onmouseup: handle_mouse_up,

            if root_id.is_some() && container_snapshot.has_size() {
                for tile_rect in tile_rects.iter().cloned() {
                    FlatTile {
                        key: "{tile_rect.tile_id}",
                        tile_rect: tile_rect.clone(),
                    }
                }

                for (divider_rect, origin_px, extent_px) in divider_infos.iter().cloned() {
                    FlatDivider {
                        divider_rect: divider_rect,
                        origin_px: origin_px,
                        extent_px: extent_px,
                    }
                }
            } else {
                if let Some(render_empty) = props.render_empty_state {
                    {(render_empty.read())()}
                } else if root_id.is_none() {
                    div {
                        style: "
                            display: flex;
                            justify-content: center;
                            align-items: center;
                            height: 100%;
                            color: #888;
                            font-size: 1rem;
                        ",
                        "No panels open"
                    }
                }
            }

            if drag_state.read().is_dragging() {
                DragGhost {
                    drag_state: drag_state,
                    render_title: props.render_title,
                }
            }
        }
    }
}

#[component]
fn FlatTile(tile_rect: TileRect) -> Element {
    let layout = use_context::<Signal<MosaicLayout>>();
    let render_tile = use_context::<Signal<Box<dyn Fn(TileId) -> Option<Element>>>>();
    let render_title = use_context::<Signal<Box<dyn Fn(TileId) -> Element>>>();

    let tile_id = tile_rect.tile_id.clone();
    let tile_id_for_horizontal = tile_id.clone();
    let tile_id_for_vertical = tile_id.clone();
    let tile_id_for_close = tile_id.clone();
    let title_component = (render_title.read())(tile_id.clone());
    let content = (render_tile.read())(tile_id.clone());

    let mut layout_write = layout;

    rsx! {
        div {
            style: format!(
                "position: absolute; left: {}px; top: {}px; width: {}px; height: {}px; min-width: 0; min-height: 0;",
                tile_rect.x,
                tile_rect.y,
                tile_rect.width,
                tile_rect.height,
            ),

            TilePane {
                tile_id: tile_id.clone(),
                title_component: title_component,
                locked: tile_rect.locked,
                on_split_horizontal: move |_| {
                    let new_tile_id = format!("{}_new", tile_id_for_horizontal);
                    layout_write.write().split_tile(
                        &tile_id_for_horizontal,
                        SplitDirection::Horizontal,
                        new_tile_id,
                        50.0,
                    );
                },
                on_split_vertical: move |_| {
                    let new_tile_id = format!("{}_new", tile_id_for_vertical);
                    layout_write.write().split_tile(
                        &tile_id_for_vertical,
                        SplitDirection::Vertical,
                        new_tile_id,
                        50.0,
                    );
                },
                on_close: move |_| {
                    layout_write.write().close_tile(&tile_id_for_close);
                },

                {content}
            }
        }
    }
}

#[component]
fn FlatDivider(divider_rect: DividerRect, origin_px: f64, extent_px: f64) -> Element {
    let layout = use_context::<Signal<MosaicLayout>>();
    let mut resize_state = use_context::<Signal<ResizeState>>();
    let mut active_resize = use_context::<Signal<ActiveResize>>();

    let split_node = layout.read().get_node(&divider_rect.split_node_id).cloned();
    let Some(Node::Split {
        min_percentage,
        max_percentage,
        locked,
        ..
    }) = split_node
    else {
        return rsx! { div { style: "display: none;" } };
    };

    // TODO(v0.2.0): extract hardcoded divider colors into configurable theme props
    let (cursor, indicator_style, bg_color) = match divider_rect.direction {
        SplitDirection::Horizontal => (
            "col-resize",
            "position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%); width: 2px; height: 30px; background-color: #555; border-radius: 2px;",
            "#2a2f3a",
        ),
        SplitDirection::Vertical => (
            "row-resize",
            "position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%); width: 30px; height: 2px; background-color: #555; border-radius: 2px;",
            "#2a2f3a",
        ),
    };

    rsx! {
        div {
            key: "divider-{divider_rect.split_node_id}",
            class: "mosaic-divider",
            style: format!(
                "position: absolute; left: {}px; top: {}px; width: {}px; height: {}px; background-color: {}; cursor: {}; border-radius: 3px; z-index: 20; transition: background-color 0.2s ease;",
                divider_rect.x,
                divider_rect.y,
                divider_rect.width,
                divider_rect.height,
                bg_color,
                cursor,
            ),
            onmousedown: move |evt: Event<MouseData>| {
                evt.prevent_default();
                if locked {
                    return;
                }

                active_resize.set(ActiveResize::start(
                    divider_rect.split_node_id.clone(),
                    divider_rect.direction,
                    origin_px,
                    extent_px,
                    min_percentage,
                    max_percentage,
                ));
                resize_state.write().is_resizing = true;
            },

            div { style: indicator_style }
        }
    }
}

fn build_split_bounds(
    layout: &MosaicLayout,
    root_id: Option<&NodeId>,
    tile_rects: &[TileRect],
) -> HashMap<NodeId, Bounds> {
    let tile_rect_map: HashMap<TileId, TileRect> = tile_rects
        .iter()
        .cloned()
        .map(|rect| (rect.tile_id.clone(), rect))
        .collect();
    let mut split_bounds = HashMap::new();

    if let Some(root_id) = root_id {
        collect_bounds(layout, root_id, &tile_rect_map, &mut split_bounds);
    }

    split_bounds
}

fn collect_bounds(
    layout: &MosaicLayout,
    node_id: &NodeId,
    tile_rect_map: &HashMap<TileId, TileRect>,
    split_bounds: &mut HashMap<NodeId, Bounds>,
) -> Option<Bounds> {
    match layout.get_node(node_id)? {
        Node::Tile { tile_id, .. } => tile_rect_map.get(tile_id).map(Bounds::from_tile_rect),
        Node::Split {
            id, first, second, ..
        } => {
            let first_bounds = collect_bounds(layout, first, tile_rect_map, split_bounds)?;
            let second_bounds = collect_bounds(layout, second, tile_rect_map, split_bounds)?;
            let bounds = first_bounds.union(second_bounds);
            split_bounds.insert(id.clone(), bounds);
            Some(bounds)
        }
    }
}
