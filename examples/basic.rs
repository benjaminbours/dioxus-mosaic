use dioxus::prelude::*;
use dioxus_mosaic::{Mosaic, MosaicBuilder, tile};

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // Create a simple 3-panel layout: sidebar | (editor / terminal)
    let mut layout = use_signal(|| {
        MosaicBuilder::horizontal()
            .left(tile("sidebar"))
            .right(
                MosaicBuilder::vertical()
                    .top(tile("editor"))
                    .bottom(tile("terminal"))
                    .split(70.0)  // 70% editor, 30% terminal
                    .build_tree()
            )
            .split(25.0)  // 25% sidebar, 75% main area
            .build()
    });

    // Render functions need to be boxed and wrapped in signals
    let render_tile = use_signal(|| {
        Box::new(move |tile_id: String| {
            match tile_id.as_str() {
                "sidebar" => Some(rsx! { SidebarPanel {} }),
                "editor" => Some(rsx! { EditorPanel {} }),
                "terminal" => Some(rsx! { TerminalPanel {} }),
                _ => None
            }
        }) as Box<dyn Fn(String) -> Option<Element>>
    });

    let render_title = use_signal(|| {
        Box::new(move |tile_id: String| {
            rsx! {
                span {
                    match tile_id.as_str() {
                        "sidebar" => "Files",
                        "editor" => "Editor",
                        "terminal" => "Terminal",
                        _ => "Unknown"
                    }
                }
            }
        }) as Box<dyn Fn(String) -> Element>
    });

    rsx! {
        style { {include_str!("styles.css")} }

        div { class: "app",
            h1 { class: "title", "dioxus-mosaic - Basic Example" }

            div { class: "mosaic-container",
                Mosaic {
                    layout: layout,
                    render_tile: render_tile,
                    render_title: render_title,
                }
            }
        }
    }
}

#[component]
fn SidebarPanel() -> Element {
    rsx! {
        div { class: "panel sidebar",
            h2 { "Files" }
            ul { class: "file-list",
                li { "📄 main.rs" }
                li { "📄 lib.rs" }
                li { "📁 components/" }
                li { "  📄 button.rs" }
                li { "  📄 input.rs" }
                li { "📁 views/" }
                li { "  📄 home.rs" }
                li { "📄 Cargo.toml" }
            }
        }
    }
}

#[component]
fn EditorPanel() -> Element {
    rsx! {
        div { class: "panel editor",
            div { class: "editor-header",
                span { "main.rs" }
            }
            pre { class: "code",
                code {
                    "use dioxus::prelude::*;\n"
                    "use dioxus_mosaic::{{Mosaic, MosaicBuilder, tile}};\n"
                    "\n"
                    "fn main() {{\n"
                    "    dioxus::launch(App);\n"
                    "}}\n"
                    "\n"
                    "#[component]\n"
                    "fn App() -> Element {{\n"
                    "    let mut layout = use_signal(|| {{\n"
                    "        MosaicBuilder::horizontal()\n"
                    "            .left(tile(\"sidebar\"))\n"
                    "            .right(tile(\"editor\"))\n"
                    "            .build()\n"
                    "    }});\n"
                    "\n"
                    "    rsx! {{\n"
                    "        Mosaic {{\n"
                    "            layout: layout,\n"
                    "            render_tile: |id| match id {{\n"
                    "                \"sidebar\" => rsx! {{ div {{ \"Sidebar\" }} }},\n"
                    "                \"editor\" => rsx! {{ div {{ \"Editor\" }} }},\n"
                    "                _ => None\n"
                    "            }}\n"
                    "        }}\n"
                    "    }}\n"
                    "}}\n"
                }
            }
        }
    }
}

#[component]
fn TerminalPanel() -> Element {
    rsx! {
        div { class: "panel terminal",
            div { class: "terminal-header",
                span { "Terminal" }
            }
            pre { class: "terminal-output",
                "$ dx serve --example basic\n"
                "🚀 Starting development server...\n"
                "📦 Compiling dioxus-mosaic v0.1.0\n"
                "✅ Compiled successfully!\n"
                "🌐 Server running at http://localhost:8080\n"
                "\n"
                "Try:\n"
                "  • Drag the dividers to resize panels\n"
                "  • Click split buttons to add new panels\n"
                "  • Close panels with the X button\n"
                "  • Drag panel headers to reorder\n"
                "  • Refresh the page - layout persists!\n"
            }
        }
    }
}
