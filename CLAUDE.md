# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust game project called "miniature-potato" - a simplified mini world (potato) to roam around and explore. It creates a spherical 3D world using a Goldberg polyhedron with hexagonal and pentagonal tiling (like a soccer ball pattern).

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

The project uses Bevy 0.16.1 game engine with an ECS (Entity Component System) architecture. Key modules:

- `main.rs`: Entry point, sets up Bevy app and registers systems
- `camera.rs`: Orbit camera with mouse drag rotation, wheel zoom, and WASD panning
- `game_tiles.rs`: Core tile logic, pentagon/hexagon management, and game state
- `goldberg_polyhedron.rs`: Pentagon-centric Goldberg polyhedron construction algorithm
- `icosphere.rs`: Icosphere generation with subdivision levels
- `polygon_mesh.rs`: Mesh generation for hexagons and pentagons
- `ray_casting.rs`: Mouse picking system for tile selection
- `world.rs`: World setup, tile spawning, and visualization

The game world is a spherical surface divided into hexagons and pentagons. Each tile is an entity with components for position, mesh, material, and game state. The ray casting system enables mouse interaction with tiles.

## Current Implementation Status

The project has a complete Goldberg polyhedron implementation with:
- Exactly 12 pentagons (topological requirement for any spherical tiling)
- Variable number of hexagons based on (h,k) parameters
- Vertex connectivity-based tile detection (5-connected = pentagon, 6-connected = hexagon)
- Wireframe debug mode (toggle with spacebar)
- Mouse hover tile selection with visual feedback

## Game Controls

- **Mouse drag**: Orbit camera around sphere
- **Mouse wheel**: Zoom in/out
- **Spacebar**: Toggle wireframe mode to see triangle mesh
- **Escape**: Quit application

## Performance Notes

- Dynamic linking is enabled for Bevy to speed up development builds
- Dev profile uses optimization level 1 for code, level 3 for dependencies
- The project is configured for fast iteration during development