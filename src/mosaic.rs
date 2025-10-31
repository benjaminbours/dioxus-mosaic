use crate::drag_drop::{DragGhost, DragState};
use crate::layout::MosaicLayout;
use crate::node::Node;
use crate::split_pane::SplitPane;
use crate::tile_pane::TilePane;
use crate::types::{NodeId, TileId};
use dioxus::prelude::*;

/// Props for the Mosaic component
#[derive(PartialEq, Clone, Props)]
pub struct MosaicProps {
    /// Signal containing the MosaicLayout
    pub layout: Signal<MosaicLayout>,

    /// Function to render each tile's content
    /// Takes a TileId and returns an optional Element
    pub render_tile: Signal<Box<dyn Fn(TileId) -> Option<Element>>>,

    /// Function to render each tile's title
    /// Takes a TileId and returns an Element for the title
    pub render_title: Signal<Box<dyn Fn(TileId) -> Element>>,

    /// Optional function to render empty state when no tiles are open
    /// If not provided, a default message will be shown
    #[props(default = None)]
    pub render_empty_state: Option<Signal<Box<dyn Fn() -> Element>>>,
}

/// Main mosaic component
///
/// Renders a tiling window manager with resizable splits and dynamic tiles.
///
/// # Example
/// ```ignore
/// let render_fn = use_signal(|| Box::new(move |tile_id: String| {
///     match tile_id.as_str() {
///         "editor" => Some(rsx! { EditorPanel {} }),
///         "sidebar" => Some(rsx! { SidebarPanel {} }),
///         _ => None
///     }
/// }) as Box<dyn Fn(String) -> Option<Element>>);
///
/// Mosaic {
///     layout: layout_signal,
///     render_tile: render_fn,
/// }
/// ```
pub fn Mosaic(props: MosaicProps) -> Element {
    let layout = props.layout;

    // Initialize drag state
    let drag_state = use_signal(DragState::new);

    // Provide layout signal, drag state, and render functions to all child components via context
    use_context_provider(|| layout);
    use_context_provider(|| drag_state);
    use_context_provider(|| props.render_tile);
    use_context_provider(|| props.render_title);

    let root_id = layout.read().root().cloned();

    rsx! {
        div {
            class: "mosaic-container",
            style: "width: 100%; height: 100%; position: relative;",

            // Render content based on whether layout is empty
            if let Some(root) = root_id {
                // Recursively render from root
                MosaicNode {
                    node_id: root,
                }
            } else {
                // Render empty state
                if let Some(render_empty) = props.render_empty_state {
                    {(render_empty.read())()}
                } else {
                    // Default empty state
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

            // Render drag ghost when dragging
            if drag_state.read().is_dragging() {
                DragGhost {
                    drag_state: drag_state,
                    render_title: props.render_title,
                }
            }
        }
    }
}

/// Internal component for rendering a single node (recursively)
#[component]
fn MosaicNode(node_id: NodeId) -> Element {
    let mut layout = use_context::<Signal<MosaicLayout>>();
    let render_tile = use_context::<Signal<Box<dyn Fn(TileId) -> Option<Element>>>>();
    let render_title = use_context::<Signal<Box<dyn Fn(TileId) -> Element>>>();
    let node = layout.read().get_node(&node_id).cloned();

    match node {
        Some(Node::Tile {
            tile_id,
            locked,
            ..
        }) => {
            // Clone tile_id for use in multiple closures
            let tile_id_for_horizontal = tile_id.clone();
            let tile_id_for_vertical = tile_id.clone();
            let tile_id_for_close = tile_id.clone();

            // Render title and content
            let title = (render_title.read())(tile_id.clone());
            let content = (render_tile.read())(tile_id.clone());

            rsx! {
                TilePane {
                    tile_id: tile_id.clone(),
                    title_component: title,
                    locked: locked,
                    on_split_horizontal: move |_| {
                        let new_tile_id = format!("{}_new", tile_id_for_horizontal);
                        layout.write().split_tile(
                            &tile_id_for_horizontal,
                            crate::types::SplitDirection::Horizontal,
                            new_tile_id,
                            50.0
                        );
                    },
                    on_split_vertical: move |_| {
                        let new_tile_id = format!("{}_new", tile_id_for_vertical);
                        layout.write().split_tile(
                            &tile_id_for_vertical,
                            crate::types::SplitDirection::Vertical,
                            new_tile_id,
                            50.0
                        );
                    },
                    on_close: move |_| {
                        layout.write().close_tile(&tile_id_for_close);
                    },

                    {content}
                }
            }
        }

        Some(Node::Split {
            direction,
            first,
            second,
            split_percentage,
            ..
        }) => {
            // Render a split with two children
            let node_id_for_resize = node_id.clone();

            rsx! {
                SplitPane {
                    direction: direction,
                    initial_size: split_percentage,
                    min_size: 20.0,
                    max_size: 80.0,
                    on_resize: Some(EventHandler::new(move |new_pos: f64| {
                        layout.write().update_split(&node_id_for_resize, new_pos);
                    })),

                    first_pane: rsx! {
                        MosaicNode {
                            node_id: first.clone(),
                        }
                    },

                    second_pane: rsx! {
                        MosaicNode {
                            node_id: second.clone(),
                        }
                    },
                }
            }
        }

        None => {
            // Node not found (shouldn't happen)
            rsx! {
                div {
                    style: "color: red; padding: 1rem;",
                    "Error: Node not found"
                }
            }
        }
    }
}
