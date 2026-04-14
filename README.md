# dioxus-mosaic

[![Crates.io](https://img.shields.io/crates/v/dioxus-mosaic.svg)](https://crates.io/crates/dioxus-mosaic)
[![Documentation](https://docs.rs/dioxus-mosaic/badge.svg)](https://docs.rs/dioxus-mosaic)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A React-Mosaic-style tiling window manager library for Dioxus applications.

## Demo

![Demo](assets/demo.gif)

## Features

- ⚡ **HashMap-based architecture** - O(1) lookups for smooth 60 FPS performance
- 📐 **Binary splits** - Simple, proven pattern (like VSCode, Sublime)
- 🎯 **Resizable dividers** - Drag to resize panes smoothly
- ✂️ **Dynamic splitting** - Split any tile horizontally or vertically
- 🎮 **Panel controls** - Close tiles, collapse/expand
- 💾 **Serializable layout** - Save/restore via serde (e.g. LocalStorage)
- 🏗️ **Clean builder API** - Easy-to-use tree-like configuration
- 🎨 **Drag-and-drop** - Reorder tiles by dragging

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
dioxus-mosaic = "0.1.0"
```

### Basic Example

```rust
use dioxus::prelude::*;
use dioxus_mosaic::{Mosaic, MosaicBuilder, tile};

fn App() -> Element {
    let mut layout = use_signal(|| {
        MosaicBuilder::horizontal()
            .left(tile("sidebar"))
            .right(
                MosaicBuilder::vertical()
                    .top(tile("editor"))
                    .bottom(tile("terminal"))
                    .split(70.0)  // 70% top, 30% bottom
                    .build_tree()  // nested builders need build_tree()
            )
            .split(25.0)  // 25% left, 75% right
            .build()
    });

    let render_tile = use_signal(|| {
        Box::new(move |tile_id: String| {
            match tile_id.as_str() {
                "sidebar" => Some(rsx! {
                    div { class: "panel",
                        h2 { "Sidebar" }
                        p { "Navigation content here" }
                    }
                }),
                "editor" => Some(rsx! {
                    div { class: "panel",
                        h2 { "Editor" }
                        textarea { "Your code here..." }
                    }
                }),
                "terminal" => Some(rsx! {
                    div { class: "panel",
                        h2 { "Terminal" }
                        pre { "$ cargo run" }
                    }
                }),
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
        Mosaic {
            layout: layout,
            render_tile: render_tile,
            render_title: render_title,
        }
    }
}
```

Run the example:

```bash
dx serve --example basic
```

## Architecture

### Why HashMap?

**Performance matters:** When you drag a divider, hundreds of events per second need O(1) lookups. The layout is stored as a flat HashMap; rendering traverses the tree in O(n) to compute absolute positions, but individual tile lookups and split-percentage updates remain O(1).

| Operation | Tree (React-Mosaic) | HashMap (dioxus-mosaic) |
|-----------|---------------------|------------------------|
| Find tile | O(n) | **O(1)** |
| Update split % | O(n) | **O(1)** |
| Split tile | O(n) | **O(1)** |
| Compute layout | O(n) | O(n) |

### Binary Splits

We use binary splits (2 children per split) like VSCode, Sublime Text, and Emacs:
- ✅ Simpler resize UX (one divider affects exactly 2 panes)
- ✅ More flexible (nested splits can create any layout)
- ✅ Easier to implement and reason about

Want 3+ panes side-by-side? Just nest splits:

```rust
MosaicBuilder::horizontal()
    .left(tile("A"))
    .right(
        MosaicBuilder::horizontal()
            .left(tile("B"))
            .right(tile("C"))
            .split(50.0)
            .build()
    )
    .split(33.3)
    .build()
```

Result: `[A | B | C]` ✓

## Advanced Usage

### Complex Layouts

```rust
let layout = MosaicBuilder::horizontal()
    .left(
        MosaicBuilder::vertical()
            .top(tile("header"))
            .bottom(
                MosaicBuilder::horizontal()
                    .left(tile("sidebar"))
                    .right(tile("editor"))
                    .split(25.0)
                    .build()
            )
            .split(20.0)
            .build()
    )
    .right(
        MosaicBuilder::vertical()
            .top(tile("preview"))
            .bottom(tile("console"))
            .split(70.0)
            .build()
    )
    .split(75.0)
    .build();
```

### Programmatic Control

```rust
// Access layout state
let mut layout = use_signal(|| MosaicBuilder::horizontal()...build());

// Split a tile programmatically
let layout_clone = layout.clone();
let split_editor = move |_| {
    layout_clone.write().split_tile(
        "editor",
        SplitDirection::Vertical,
        "new_panel"
    );
};

// Close a tile
let close_panel = move |_| {
    layout.write().remove_tile("sidebar");
};
```

### Persistence

The layout is serializable via serde. You can persist it to LocalStorage, a backend, or any other store:

```rust
// Save layout
let json = serde_json::to_string(&layout.read().to_tree())?;
// Store in LocalStorage, your backend, file, etc.

// Restore layout
let tree: MosaicNode = serde_json::from_str(&json)?;
layout.set(MosaicLayout::from_tree(tree));
```

## Examples

- **`basic.rs`** - Simple 4-panel layout (sidebar, editor, interactive, terminal)
- **`advanced.rs`** - Complex multi-panel layout with all features

Run examples:

```bash
# Simple example
dx serve --example basic

# Advanced example
dx serve --example advanced
```

## Features Roadmap

### v0.1.0 (Current) ✅
- [x] HashMap-based layout with O(1) lookups
- [x] Binary splits (horizontal/vertical)
- [x] Resizable dividers
- [x] Dynamic splitting and closing
- [x] Serializable layout (serde)
- [x] Drag-and-drop tile reordering
- [x] Clean builder API

### v0.2.0 (Planned)
- [ ] Undo/Redo with keyboard shortcuts
- [ ] Themes and custom styling
- [ ] Layout templates
- [ ] Comprehensive documentation

### Future
- [ ] Floating panels (detach from grid)
- [ ] Tab groups (multiple tiles in one pane)
- [ ] Custom tile widgets (progress bars, badges)

## Performance

Optimized for real-time interaction:
- **< 16ms** frame time (60 FPS) even during drag operations
- **O(1)** HashMap lookups for tile access and split updates
- **O(n)** layout pass to compute absolute tile positions
- **Zero-cost** abstractions with Rust

## API Documentation

Full API documentation available at [docs.rs/dioxus-mosaic](https://docs.rs/dioxus-mosaic).

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT - See [LICENSE](LICENSE) for details.

## Real-World Usage

*Using dioxus-mosaic in your project? Open a PR to add it here!*

## Acknowledgments

Inspired by:
- [react-mosaic-component](https://github.com/nomcopter/react-mosaic-component) - The original React implementation
- VSCode's split view system
- Sublime Text's pane management

Built with ❤️ using [Dioxus](https://dioxuslabs.com/)
