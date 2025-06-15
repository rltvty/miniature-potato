# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust project using Bevy to create an interactive spherical world with geodesic polyhedron tiling. The project generates a sphere covered with hexagonal and pentagonal tiles using the geotiles crate, providing a clean, modern implementation with comprehensive UI and interaction features.

## Commands

### Development & Testing
```bash
cargo run                    # Run the interactive application 
cargo run -- --screenshot   # Take a screenshot and exit (for testing)
cargo check                  # Quick compile check
cargo build                  # Build debug version
cargo build --release       # Build optimized release version
```

### Code Quality
```bash
cargo clippy                 # Run linter
cargo fmt                    # Format code
cargo test                   # Run tests (if any)
```

## Current Implementation Status

### ✅ Complete Features
- **Geodesic Sphere**: 92 tiles (12 pentagons, 80 hexagons) with mathematically accurate Goldberg polyhedron
- **Interactive Camera**: Mouse rotates sphere, WASD moves camera freely, mouse wheel zooms
- **Real-time UI**: Persistent on-screen displays showing sphere info and tile interactions
- **Tile Interaction**: 
  - Hover detection with real-time feedback
  - Click selection with visual highlighting
  - Detailed tile information display
- **Visual Modes**: 
  - Solid tile rendering with proper colors (green hexagons, magenta pentagons)
  - Wireframe toggle showing tile geometry
  - Normal vector visualization for debugging
- **Selection System**: White wireframe highlighting that follows sphere rotation

### 🖥️ User Interface Layout
- **Top-left**: Controls help text
- **Top-right**: Sphere statistics (total tiles, counts, radius, size variation)
- **Bottom-left**: Hovered tile info (updates continuously)
- **Bottom-right**: Selected tile info (updates on click)

### 🎛️ Controls
- **Right mouse drag**: Rotate sphere around its center
- **Mouse wheel**: Zoom camera forward/backward
- **WASD**: Move camera up/down/left/right
- **Left click**: Select/deselect tiles
- **Space**: Toggle wireframe mode
- **N**: Toggle normal vector visualization
- **I**: Print detailed hexasphere info to console
- **Esc**: Quit application

## Architecture

### Core Files (Clean, Minimal Structure)
```
src/
├── main.rs              # Main entry point with UI systems
├── camera.rs            # Camera controls and sphere rotation
├── geotiles_bevy.rs     # Bevy integration for geotiles crate
└── lib.rs               # Module declarations
```

### Key Dependencies
- **Bevy 0.16.1** - Game engine, rendering, and UI
- **geotiles** - Geodesic polyhedron generation (from GitHub: rltvty/geotiles)

### Configuration
Located in `src/geotiles_bevy.rs`:
```rust
const SPHERE_RADIUS: f64 = 5.0;      // Size of the sphere
const SUBDIVISIONS: usize = 10;      // Complexity level (higher = more tiles)
const TILE_SIZE: f64 = 0.99;         // Size factor for tiles
const TILE_THICKNESS: f64 = 0.2;     // Thickness for 3D extrusion
const USE_UNIFORM_TILES: bool = false; // Toggle between thick tiles and uniform approximations
```

## Technical Implementation

### Geodesic Generation
- Uses the `geotiles` crate for mathematically accurate Goldberg polyhedron generation
- Creates thick extruded tiles with proper flat surfaces (not pyramids)
- Maintains excellent uniformity with minimal size variation
- Supports both thick tile geometry and uniform hexagon approximations

### Rendering System
- Individual material instances per tile prevent visual artifacts
- Proper normal vector calculation for correct lighting
- Back-face culling enabled for performance
- Wireframe mode using Bevy's built-in wireframe plugin

### Interaction & UI
- **Ray-casting**: Accurate mouse hover detection accounting for sphere rotation
- **Real-time updates**: UI text updates every frame with current state
- **Persistent display**: No need to press keys for information - everything visible on screen
- **Visual feedback**: Material color changes and wireframe highlighting for selection

### Camera System
- **Sphere rotation**: Mouse input rotates the sphere while camera stays oriented forward
- **Free movement**: WASD moves camera in screen-relative directions
- **Independent zoom**: Mouse wheel moves camera along its forward direction
- **Proper constraints**: Distance limits prevent getting too close/far from sphere

## Development Notes

### Screenshot Testing
Run with `--screenshot` flag to automatically:
1. Initialize application
2. Wait 2 seconds for setup
3. Capture screenshot to timestamped PNG file
4. Exit cleanly

### UI Features
- **Font**: Uses Bevy's default font (no external dependencies)
- **Layout**: Absolute positioning in all four corners
- **Colors**: Different colors for different types of information
- **Real-time**: All displays update automatically without user input

### Performance Optimizations
- Efficient tile hover detection with rotation compensation
- Individual materials prevent flashing during hover/selection
- Proper culling and lighting setup
- Minimal overdraw with back-face culling

### Code Quality
- Clean, focused codebase with no legacy modules
- Well-documented functions and systems
- Consistent error handling and resource management
- Modern Bevy patterns and best practices

This represents a complete, polished implementation of an interactive geodesic sphere visualization with comprehensive UI and smooth interaction systems.