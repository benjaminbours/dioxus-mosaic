use dioxus::prelude::*;
use dioxus_mosaic::{Mosaic, MosaicBuilder, tile};

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // Create a complex IDE-like layout
    let mut layout = use_signal(|| {
        MosaicBuilder::vertical()
            .top(
                // Top section: Header
                tile("header")
            )
            .bottom(
                // Main section: Sidebar | (Editor + Preview) | Inspector
                MosaicBuilder::horizontal()
                    .left(
                        // Left sidebar with file browser and outline
                        MosaicBuilder::vertical()
                            .top(tile("files"))
                            .bottom(tile("outline"))
                            .split(60.0)
                            .build()
                    )
                    .right(
                        // Main area: Editor + Preview + Console
                        MosaicBuilder::horizontal()
                            .left(
                                // Editor and console
                                MosaicBuilder::vertical()
                                    .top(tile("editor"))
                                    .bottom(tile("console"))
                                    .split(70.0)
                                    .build()
                            )
                            .right(
                                // Preview and inspector
                                MosaicBuilder::vertical()
                                    .top(tile("preview"))
                                    .bottom(tile("inspector"))
                                    .split(60.0)
                                    .build()
                            )
                            .split(60.0)
                            .build()
                    )
                    .split(20.0)
                    .build()
            )
            .split(8.0)
            .build()
    });

    rsx! {
        style { {include_str!("advanced_styles.css")} }

        div { class: "app",
            div { class: "mosaic-container",
                Mosaic {
                    layout: layout,
                    render_tile: move |tile_id| {
                        match tile_id.as_str() {
                            "header" => rsx! { HeaderPanel {} },
                            "files" => rsx! { FilesPanel {} },
                            "outline" => rsx! { OutlinePanel {} },
                            "editor" => rsx! { EditorPanel {} },
                            "console" => rsx! { ConsolePanel {} },
                            "preview" => rsx! { PreviewPanel {} },
                            "inspector" => rsx! { InspectorPanel {} },
                            _ => None
                        }
                    },
                }
            }
        }
    }
}

#[component]
fn HeaderPanel() -> Element {
    rsx! {
        div { class: "panel header-panel",
            div { class: "header-content",
                h1 { "🎨 Advanced Layout Demo" }
                div { class: "header-actions",
                    button { "File" }
                    button { "Edit" }
                    button { "View" }
                    button { "Help" }
                }
            }
        }
    }
}

#[component]
fn FilesPanel() -> Element {
    rsx! {
        div { class: "panel files-panel",
            h3 { "📁 Files" }
            ul { class: "tree",
                li { "📁 src" }
                li { class: "indent", "📄 main.rs" }
                li { class: "indent", "📄 lib.rs" }
                li { class: "indent", "📁 components" }
                li { class: "indent2", "📄 button.rs" }
                li { class: "indent2", "📄 layout.rs" }
                li { "📁 examples" }
                li { class: "indent", "📄 basic.rs" }
                li { class: "indent", "📄 advanced.rs" }
                li { "📄 Cargo.toml" }
                li { "📄 README.md" }
            }
        }
    }
}

#[component]
fn OutlinePanel() -> Element {
    rsx! {
        div { class: "panel outline-panel",
            h3 { "📋 Outline" }
            ul { class: "outline",
                li { "fn main()" }
                li { "#[component] App" }
                li { "#[component] HeaderPanel" }
                li { "#[component] FilesPanel" }
                li { "#[component] EditorPanel" }
            }
        }
    }
}

#[component]
fn EditorPanel() -> Element {
    rsx! {
        div { class: "panel editor-panel",
            div { class: "editor-tabs",
                div { class: "tab active", "main.rs" }
                div { class: "tab", "lib.rs" }
            }
            pre { class: "code",
                "use dioxus::prelude::*;\n"
                "use dioxus_mosaic::{{Mosaic, MosaicBuilder, tile}};\n"
                "\n"
                "/// Advanced multi-panel layout\n"
                "/// \n"
                "/// Features demonstrated:\n"
                "/// - Complex nested splits\n"
                "/// - Multiple levels of hierarchy\n"
                "/// - Resizable dividers\n"
                "/// - Drag and drop reordering\n"
                "/// - LocalStorage persistence\n"
                "\n"
                "#[component]\n"
                "fn App() -> Element {{\n"
                "    let mut layout = use_signal(|| {{\n"
                "        MosaicBuilder::vertical()\n"
                "            .top(tile(\"header\"))\n"
                "            .bottom(\n"
                "                MosaicBuilder::horizontal()\n"
                "                    .left(tile(\"sidebar\"))\n"
                "                    .right(tile(\"main\"))\n"
                "                    .build()\n"
                "            )\n"
                "            .build()\n"
                "    }});\n"
                "\n"
                "    rsx! {{ Mosaic {{ layout }} }}\n"
                "}}\n"
            }
        }
    }
}

#[component]
fn ConsolePanel() -> Element {
    rsx! {
        div { class: "panel console-panel",
            div { class: "console-tabs",
                div { class: "tab active", "Terminal" }
                div { class: "tab", "Debug" }
                div { class: "tab", "Problems" }
            }
            pre { class: "console-output",
                "$ dx serve --example advanced\n"
                "⚡ Hot reload enabled\n"
                "📦 Building WASM bundle...\n"
                "✨ Build complete in 2.3s\n"
                "🌐 http://localhost:8080\n"
            }
        }
    }
}

#[component]
fn PreviewPanel() -> Element {
    rsx! {
        div { class: "panel preview-panel",
            h3 { "👁️ Preview" }
            div { class: "preview-content",
                div { class: "preview-box",
                    "Live preview of your application would appear here"
                }
            }
        }
    }
}

#[component]
fn InspectorPanel() -> Element {
    rsx! {
        div { class: "panel inspector-panel",
            h3 { "🔍 Inspector" }
            div { class: "properties",
                div { class: "property",
                    span { class: "key", "Layout:" }
                    span { class: "value", "Mosaic" }
                }
                div { class: "property",
                    span { class: "key", "Tiles:" }
                    span { class: "value", "7 panels" }
                }
                div { class: "property",
                    span { class: "key", "Performance:" }
                    span { class: "value", "60 FPS" }
                }
                div { class: "property",
                    span { class: "key", "Operations:" }
                    span { class: "value", "O(1) HashMap" }
                }
            }
        }
    }
}
