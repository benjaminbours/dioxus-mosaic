use crate::types::SplitDirection;
use dioxus::prelude::*;
use dioxus::signals::*;
use wasm_bindgen::JsCast;

/// A resizable split pane component
///
/// Allows users to drag a divider to resize two child panels.
/// Supports both horizontal (left/right) and vertical (top/bottom) splits.
#[component]
pub fn SplitPane(
    direction: SplitDirection,
    initial_size: f64,                    // Percentage (0.0 - 100.0) for first pane
    min_size: f64,                        // Minimum percentage
    max_size: f64,                        // Maximum percentage
    on_resize: Option<EventHandler<f64>>, // Called when user finishes dragging
    first_pane: Element,
    second_pane: Element,
) -> Element {
    // State for current split position (percentage)
    let mut split_pos = use_signal(|| initial_size);
    let mut is_dragging = use_signal(|| false);
    let mut is_hovering = use_signal(|| false);
    let mut container_ref = use_signal(|| None::<web_sys::HtmlElement>);

    // Mouse move handler for dragging
    let handle_mouse_move = move |evt: Event<MouseData>| {
        if !is_dragging() {
            return;
        }

        if let Some(container) = container_ref() {
            let rect = container.get_bounding_client_rect();

            let new_pos = match direction {
                SplitDirection::Horizontal => {
                    let x = evt.page_coordinates().x as f64;
                    let container_x = rect.left();
                    let container_width = rect.width();
                    ((x - container_x) / container_width * 100.0).clamp(min_size, max_size)
                }
                SplitDirection::Vertical => {
                    let y = evt.page_coordinates().y as f64;
                    let container_y = rect.top();
                    let container_height = rect.height();
                    ((y - container_y) / container_height * 100.0).clamp(min_size, max_size)
                }
            };

            split_pos.set(new_pos);
        }
    };

    // Mouse up handler - stop dragging and save position
    let handle_mouse_up = move |_evt: Event<MouseData>| {
        if is_dragging() {
            let current_pos = split_pos();
            is_dragging.set(false);
            // Notify parent of new position
            if let Some(handler) = &on_resize {
                handler.call(current_pos);
            }
        }
    };

    let cursor = match direction {
        SplitDirection::Horizontal => "col-resize",
        SplitDirection::Vertical => "row-resize",
    };

    rsx! {
        div {
            class: "split-pane",
                // style: "margin: 0.5rem;",
            onmounted: move |evt| {
                spawn(async move {
                    if let Some(element) = evt.data().downcast::<web_sys::Element>() {
                        if let Ok(html_element) = element.clone().dyn_into::<web_sys::HtmlElement>() {
                            container_ref.set(Some(html_element));
                        }
                    }
                });
            },
            onmousemove: handle_mouse_move,
            onmouseup: handle_mouse_up,
            style: {
                let current_split = split_pos();
                let divider_width = 4.0; // Slightly wider divider for better UX
                let gap = 6.0; // Gap between panels and divider in pixels

                let grid_template = match direction {
                    SplitDirection::Horizontal =>
                        format!("{current_split}% {gap}px {divider_width}px {gap}px calc(100% - {current_split}% - {divider_width}px - {gap}px * 2)"),
                    SplitDirection::Vertical =>
                        format!("{current_split}% {gap}px {divider_width}px {gap}px calc(100% - {current_split}% - {divider_width}px - {gap}px * 2)"),
                };
                let grid_direction = match direction {
                    SplitDirection::Horizontal => "grid-template-columns",
                    SplitDirection::Vertical => "grid-template-rows",
                };
                format!("
                    display: grid;
                    {}: {};
                    width: 100%;
                    height: 100%;
                    {}
                ", grid_direction, grid_template, if is_dragging() { "user-select: none;" } else { "" })
            },

            // First pane
            div {
                class: "split-pane-first",
                style: "min-width: 0; min-height: 0;",
                {first_pane}
            }

            // Gap before divider (also draggable, triggers hover)
            div {
                class: "split-gap-before",
                onmousedown: move |_evt| {
                    is_dragging.set(true);
                },
                onmouseenter: move |_evt| {
                    is_hovering.set(true);
                },
                onmouseleave: move |_evt| {
                    is_hovering.set(false);
                },
                style: "
                    background-color: transparent;
                    cursor: {cursor};
                ",
            }

            // Divider (drag handle)
            div {
                class: "split-divider",
                onmousedown: move |_evt| {
                    is_dragging.set(true);
                },
                onmouseenter: move |_evt| {
                    is_hovering.set(true);
                },
                onmouseleave: move |_evt| {
                    is_hovering.set(false);
                },
                style: {
                    let bg_color = if is_hovering() { "#3a4050" } else { "#2a2f3a" };
                    format!("
                        background-color: {bg_color};
                        cursor: {cursor};
                        transition: background-color 0.2s ease;
                        position: relative;
                        border-radius: 3px;
                    ")
                },

                // Visual indicator line
                div {
                    style: match direction {
                        SplitDirection::Horizontal => {
                            let indicator_color = if is_hovering() { "#888" } else { "#555" };
                            format!("
                                position: absolute;
                                top: 50%;
                                left: 50%;
                                transform: translate(-50%, -50%);
                                width: 2px;
                                height: 30px;
                                background-color: {indicator_color};
                                border-radius: 2px;
                                transition: background-color 0.2s ease;
                            ")
                        },
                        SplitDirection::Vertical => {
                            let indicator_color = if is_hovering() { "#888" } else { "#555" };
                            format!("
                                position: absolute;
                                top: 50%;
                                left: 50%;
                                transform: translate(-50%, -50%);
                                width: 30px;
                                height: 2px;
                                background-color: {indicator_color};
                                border-radius: 2px;
                                transition: background-color 0.2s ease;
                            ")
                        },
                    }
                }
            }

            // Gap after divider (also draggable, triggers hover)
            div {
                class: "split-gap-after",
                onmousedown: move |_evt| {
                    is_dragging.set(true);
                },
                onmouseenter: move |_evt| {
                    is_hovering.set(true);
                },
                onmouseleave: move |_evt| {
                    is_hovering.set(false);
                },
                style: "
                    background-color: transparent;
                    cursor: {cursor};
                ",
            }

            // Second pane
            div {
                class: "split-pane-second",
                style: "min-width: 0; min-height: 0;",
                {second_pane}
            }
        }
    }
}
