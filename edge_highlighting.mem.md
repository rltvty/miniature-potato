# Edge Highlighting System Documentation

This document describes the edge highlighting system implemented in the physics branch that can be applied to the main geotiles-based branch.

## Overview

The edge highlighting system provides visual debugging capabilities for tiles using gizmos and keyboard controls. The system is toggled with the 'C' key and provides multiple visualization modes.

## Key Components

### 1. PhysicsDebugSettings Resource

Located in `src/physics_debug.rs`, this resource controls all debug visualization options:

```rust
#[derive(Resource, Default)]
pub struct PhysicsDebugSettings {
    pub show_edge_colors: bool,      // Color-coded gizmo lines
    pub show_tile_info: bool,        // Tile information overlay  
    pub show_normals: bool,          // Normal vectors
    pub edge_label_size: f32,        // Size of 3D text labels
}
```

### 2. Keyboard Handler System

The main keyboard handler in `src/physics_debug.rs` manages the 'C' key toggle:

```rust
pub fn handle_physics_debug_keys(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut debug_settings: ResMut<PhysicsDebugSettings>,
) {
    // C: Toggle edge highlighting visualization
    if keyboard_input.just_pressed(KeyCode::KeyC) {
        debug_settings.show_edge_colors = !debug_settings.show_edge_colors;
        println!("🎨 Edge colors: {}", if debug_settings.show_edge_colors { "ON" } else { "OFF" });
    }
    
    // H: Toggle tile info display  
    if keyboard_input.just_pressed(KeyCode::KeyH) {
        debug_settings.show_tile_info = !debug_settings.show_tile_info;
        println!("📊 Tile info: {}", if debug_settings.show_tile_info { "ON" } else { "OFF" });
    }
    
    // N: Toggle normals display
    if keyboard_input.just_pressed(KeyCode::KeyN) {
        debug_settings.show_normals = !debug_settings.show_normals;
        println!("📐 Normals: {}", if debug_settings.show_normals { "ON" } else { "OFF" });
    }
}
```

### 3. Edge Highlighting Visualization System

The main visualization system in `src/physics_debug.rs`:

```rust
pub fn physics_tile_visualization(
    mut gizmos: Gizmos,
    tile_query: Query<(&Transform, &PhysicsTile)>,
    debug_settings: Res<PhysicsDebugSettings>,
) {
    for (transform, tile) in tile_query.iter() {
        let tile_color = get_tile_color(tile.tile_type, tile.tile_id);
        
        // Draw tile center
        if debug_settings.show_edge_colors {
            gizmos.sphere(transform.translation, 0.03, tile_color);
        }
        
        // Draw normal vector
        if debug_settings.show_normals {
            let normal = transform.rotation * Vec3::Z;
            let normal_end = transform.translation + normal * 0.3;
            gizmos.line(transform.translation, normal_end, tile_color);
        }
        
        // Draw tile outline/edges
        if debug_settings.show_tile_info {
            let radius = tile.radius;
            let edge_count = match tile.tile_type {
                TileType::Pentagon => 5,
                TileType::Hexagon => 6,
            };
            
            // Calculate edge vertices in tile's local coordinate system
            let normal = transform.rotation * Vec3::Z;
            let up = if normal.dot(Vec3::Y).abs() < 0.9 { Vec3::Y } else { Vec3::X };
            let tangent1 = transform.rotation * (normal.cross(up).normalize());
            let tangent2 = transform.rotation * (normal.cross(tangent1).normalize());
            
            let mut points = Vec::new();
            for i in 0..edge_count {
                let angle = (i as f32) * 2.0 * std::f32::consts::PI / (edge_count as f32);
                let local_pos = tangent1 * (angle.cos() * radius) + 
                               tangent2 * (angle.sin() * radius);
                let vertex_pos = transform.translation + local_pos;
                points.push(vertex_pos);
            }
            
            // Draw edge lines
            for i in 0..edge_count {
                let next_i = (i + 1) % edge_count;
                gizmos.line(points[i], points[next_i], tile_color);
            }
        }
    }
}
```

### 4. Color Scheme

The color system uses tile type and ID for differentiation:

```rust
pub fn get_tile_color(tile_type: TileType, tile_id: TileId) -> Color {
    match tile_type {
        TileType::Pentagon => Color::srgb(1.0, 0.3, 0.3), // Red
        TileType::Hexagon => {
            if tile_id.0 == 2 {
                Color::srgb(0.3, 1.0, 0.3) // Green for special hexagon
            } else {
                Color::srgb(0.3, 0.3, 1.0) // Blue for regular hexagons
            }
        }
    }
}
```

## Plugin Integration

The edge highlighting system is integrated through the physics debug plugin:

```rust
pub struct PhysicsDebugPlugin;

impl Plugin for PhysicsDebugPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(PhysicsDebugSettings::default())
            .add_systems(Update, (
                handle_physics_debug_keys,
                physics_tile_visualization,
            ));
    }
}
```

## Adaptation for Geotiles Branch

To implement this in the main geotiles branch:

1. **Create similar debug settings resource** - Replace `PhysicsDebugSettings` with `GeotilesDebugSettings`
2. **Adapt tile query** - Change from `Query<(&Transform, &PhysicsTile)>` to work with geotiles components
3. **Update color scheme** - Adapt `get_tile_color()` to work with geotiles tile identification
4. **Integrate with sphere rotation** - Apply sphere rotation to gizmo coordinates like existing selection highlighting
5. **Add to plugin system** - Include in main app builder with existing systems

## Key Features

- **'C' key toggle** - Primary control for edge highlighting
- **Color-coded tiles** - Different colors for pentagons/hexagons
- **Multiple visualization modes** - Centers, edges, normals
- **Console feedback** - Status messages when toggling features
- **Gizmo-based rendering** - Uses Bevy's gizmo system for efficient debug rendering

This system provides comprehensive visual debugging capabilities for understanding tile positioning, orientation, and relationships in the tessellated sphere.