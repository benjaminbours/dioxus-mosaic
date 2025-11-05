# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- Updated dioxus dependency from 0.6.3 to 0.7.0
- Simplified dioxus dependency configuration (removed explicit feature flags, now using defaults)

## [0.1.0] - 2024-10-31

### Added
- Initial release of dioxus-mosaic
- HashMap-based layout system with O(1) operations
- Binary split support (horizontal and vertical)
- Resizable dividers with drag functionality
- Dynamic panel splitting and closing
- LocalStorage persistence for layouts
- Drag-and-drop tile reordering
- Clean builder API (`MosaicBuilder`)
- Tree-like external API for easy serialization
- Comprehensive examples (basic and advanced)
- Full documentation and README

### Features
- **Performance**: O(1) tile lookups, splits, and updates
- **Flexibility**: Nested splits for any layout complexity
- **Persistence**: Automatic LocalStorage save/restore
- **UX**: Smooth 60 FPS interactions during resize/drag
- **API**: Intuitive builder pattern for layout definition

### Architecture
- HashMap-based internal storage for performance
- Tree-like external API for developer ergonomics
- Binary splits (2 children per split)
- Metadata support (collapsed state, locked tiles, custom titles)

[Unreleased]: https://github.com/benjaminbours/dioxus-mosaic/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/benjaminbours/dioxus-mosaic/releases/tag/v0.1.0
