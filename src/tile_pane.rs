use crate::drag_drop::{get_drop_zone_style, DragState, DropZone, TileRefs};
use crate::types::TileId;
use dioxus::prelude::*;

#[component]
pub fn TilePane(
    tile_id: TileId,
    title_component: Element,
    locked: bool,
    on_split_horizontal: EventHandler<()>,
    on_split_vertical: EventHandler<()>,
    on_close: EventHandler<()>,
    children: Element,
) -> Element {
    let mut drag_state = use_context::<Signal<DragState>>();
    let mut tile_refs = use_context::<Signal<TileRefs>>();

    let is_being_dragged = drag_state.read().dragging_tile_id.as_ref() == Some(&tile_id);
    let is_drag_active = drag_state.read().is_dragging();
    let tile_opacity = if is_being_dragged { "0.4" } else { "1.0" };
    let header_cursor = if is_drag_active { "grabbing" } else { "grab" };

    let current_drop_zone: Option<DropZone> = drag_state
        .read()
        .hover_target
        .as_ref()
        .filter(|(tid, _)| tid == &tile_id)
        .map(|(_, zone)| *zone);

    let tile_id_for_mousedown = tile_id.clone();

    rsx! {
        div {
            class: "mosaic-tile-pane",
            onmounted: {
                let tile_id = tile_id.clone();
                move |evt: MountedEvent| {
                    tile_refs.write().register(tile_id.clone(), evt.clone());
                }
            },
            style: "
                background-color: #1a1d24;
                border: 1px solid #2a2f3a;
                border-radius: 8px;
                overflow: hidden;
                display: flex;
                flex-direction: column;
                height: 100%;
                position: relative;
                opacity: {tile_opacity};
                transition: opacity 0.2s ease;
            ",

            div {
                class: "mosaic-tile-header",
                onmousedown: move |evt: Event<MouseData>| {
                    evt.prevent_default();
                    let mouse_x = evt.page_coordinates().x as f64;
                    let mouse_y = evt.page_coordinates().y as f64;
                    drag_state.write().start_drag(tile_id_for_mousedown.clone(), mouse_x, mouse_y);
                },
                style: "
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    padding: 0.5rem 0.75rem;
                    border-bottom: 1px solid #2a2f3a;
                    background-color: #14161c;
                    flex-shrink: 0;
                    cursor: {header_cursor};
                    user-select: none;
                ",

                div {
                    style: "
                        font-size: 0.875rem;
                        font-weight: 600;
                        color: #ffffff;
                        margin: 0;
                        flex: 1;
                        pointer-events: none;
                    ",
                    {title_component}
                }

                div {
                    class: "mosaic-tile-controls",
                    style: "display: flex; gap: 0.25rem; align-items: center;",

                    if !locked {
                        button {
                            onclick: move |_| on_close.call(()),
                            title: "Close",
                            style: "
                                background: none;
                                border: 1px solid #3a4050;
                                color: #d66;
                                cursor: pointer;
                                font-size: 0.75rem;
                                padding: 0.25rem 0.5rem;
                                border-radius: 3px;
                                transition: all 0.2s ease;
                            ",
                            "✕"
                        }
                    }
                }
            }

            div {
                class: "mosaic-tile-content",
                style: "
                    flex: 1;
                    overflow: auto;
                    min-height: 0;
                ",
                {children}
            }

            if is_drag_active && !is_being_dragged {
                div {
                    class: "drop-zone drop-zone-top",
                    style: get_drop_zone_style(
                        DropZone::Top,
                        current_drop_zone == Some(DropZone::Top)
                    ),
                }
                div {
                    class: "drop-zone drop-zone-bottom",
                    style: get_drop_zone_style(
                        DropZone::Bottom,
                        current_drop_zone == Some(DropZone::Bottom)
                    ),
                }
                div {
                    class: "drop-zone drop-zone-left",
                    style: get_drop_zone_style(
                        DropZone::Left,
                        current_drop_zone == Some(DropZone::Left)
                    ),
                }
                div {
                    class: "drop-zone drop-zone-right",
                    style: get_drop_zone_style(
                        DropZone::Right,
                        current_drop_zone == Some(DropZone::Right)
                    ),
                }
            }
        }
    }
}
