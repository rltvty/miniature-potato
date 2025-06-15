# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust game project called "miniature-potato" - a simplified mini world to roam around and explore. It creates a spherical 3D world using the geotiles crate for geodesic polyhedron generation with hexagonal and pentagonal tiling (like a soccer ball pattern).

## Development Commands

**Running the game:**
```bash
cargo run
```

**Building:**
```bash
cargo build              # Debug build with optimizations
cargo build --release    # Release build
```

**Testing:**
```bash
cargo test
```

**Checking code:**
```bash
cargo check             # Quick syntax/type check
cargo clippy            # Linting
cargo fmt               # Format code
```

## Architecture Overview

The project uses Bevy 0.16.1 game engine with an ECS (Entity Component System) architecture and the geotiles crate for geodesic polyhedron generation.

### Key Modules
- `main.rs`: Entry point, sets up Bevy app with geotiles-based world
- `geotiles_bevy.rs`: Bevy integration for the geotiles crate
- `camera.rs`: Orbit camera with mouse drag rotation, wheel zoom, and WASD panning

### Legacy Modules (kept for reference)
- `goldberg_polyhedron.rs`: Original custom implementation
- `game_tiles.rs`: Original tile system
- `icosphere.rs`: Original icosphere generation
- `polygon_mesh.rs`: Original mesh generation
- `ray_casting.rs`: Original ray casting system
- `world.rs`: Original world setup

## Current Implementation

The game world uses the geotiles crate to generate a mathematically accurate Goldberg polyhedron with:
- **Exactly 12 pentagons** (topological requirement for spherical tiling)
- **80 hexagons** at subdivision level 3
- **Individual tile materials** for proper hover/selection effects
- **Uniform hexagon approximation** with 3.9% size variation

Each tile is an entity with:
- TileComponent (index, is_pentagon flag)
- Individual StandardMaterial for color changes
- Proper orientation using geotiles TileOrientation
- Mesh3d with pre-computed regular polygon mesh

## Game Controls

- **Mouse drag**: Orbit camera around sphere
- **Mouse wheel**: Zoom in/out  
- **Left click**: Select/deselect tiles
- **Space**: Toggle wireframe mode to see triangle mesh
- **I**: Print complete hexasphere statistics
- **H**: Print detailed info about currently hovered tile
- **Esc**: Quit application

## Debug Features

The game includes comprehensive debug features:

### Tile Information (I key)
- Total tile count and type breakdown
- Sphere radius and uniform hexagon radius
- Hexagon statistics (min/max/average radius, size variation)
- Standard deviation calculations

### Hovered Tile Info (H key)
- Tile index and type (pentagon/hexagon)
- Boundary vertex count and neighbor count
- 3D center coordinates
- Latitude/longitude coordinates
- List of neighbor tile indices

### Visual Debug
- Wireframe toggle shows underlying triangle structure
- Hover highlighting with yellow color and emissive effect
- Tile selection feedback in console

## Geotiles Integration

The project uses the geotiles crate (https://github.com/rltvty/geotiles) which provides:
- Mathematically correct Goldberg polyhedron generation
- Built-in hexagon uniformity analysis
- Proper tile orientations for 3D placement
- Neighbor relationship calculations
- Export capabilities (OBJ, JSON)

### Configuration
- **Sphere radius**: 5.0
- **Subdivision level**: 3 (provides good detail without performance issues)
- **Tile size factor**: 0.9 (90% size to show gaps between tiles)

## Performance Notes

- Dynamic linking is enabled for Bevy to speed up development builds
- Dev profile uses optimization level 1 for code, level 3 for dependencies
- Each tile has individual material instance to prevent color flashing
- Pre-computed regular polygon meshes shared between tiles of same type
- Efficient hover detection using sphere ray intersection