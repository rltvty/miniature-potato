//! Game Tiles System for True Goldberg Polyhedron
//! 
//! Provides game-specific functionality on top of the mathematically correct
//! Goldberg polyhedron structure. Each tile corresponds to an actual pentagon
//! or hexagon face in the polyhedron.

use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use crate::goldberg_polyhedron::{GoldbergPolyhedron, FaceType};
use crate::polygon_mesh::{PolyhedronMesh, FaceMapping};

/// Game tile system built on true Goldberg polyhedron
#[derive(Resource)]
pub struct GameTileSystem {
    /// The underlying mathematical polyhedron
    pub polyhedron: GoldbergPolyhedron,
    /// Mesh data for rendering
    pub mesh_data: PolyhedronMesh,
    /// Game-specific tile data
    pub tiles: Vec<GameTile>,
    /// Fast lookup from triangle to tile
    pub triangle_to_tile: HashMap<usize, usize>,
    /// Currently hovered tile (if any)
    pub hovered_tile: Option<usize>,
    /// Currently selected tile (if any)
    pub selected_tile: Option<usize>,
}

/// A game tile corresponding to a polygon face
#[derive(Debug, Clone)]
pub struct GameTile {
    /// Type and index of the underlying polygon face
    pub face_type: FaceType,
    /// Current state of the tile
    pub state: TileState,
    /// Color override for this tile
    pub color_override: Option<Color>,
    /// Whether this tile is currently highlighted
    pub highlighted: bool,
    /// Custom data attached to this tile
    pub data: TileData,
}

/// Different states a tile can be in
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileState {
    Empty,
    Occupied,
    Selected,
    Blocked,
    Special,
}

/// Custom data that can be attached to tiles
#[derive(Debug, Clone, Default)]
pub struct TileData {
    /// Arbitrary key-value pairs
    pub properties: HashMap<String, TileProperty>,
}

/// Values that can be stored in tile properties
#[derive(Debug, Clone)]
pub enum TileProperty {
    Integer(i32),
    Float(f32),
    String(String),
    Boolean(bool),
    Color(Color),
}

impl GameTileSystem {
    /// Create a new game tile system from a Goldberg polyhedron
    pub fn new(polyhedron: GoldbergPolyhedron) -> Self {
        println!("🎮 Creating game tile system...");
        
        // Generate mesh for rendering
        let mesh_data = polyhedron.to_flat_mesh();
        
        // Create game tiles for each face
        let mut tiles = Vec::new();
        let mut triangle_to_tile = HashMap::new();
        
        // Create tiles for pentagons
        for pentagon_idx in 0..polyhedron.pentagons.len() {
            let tile_idx = tiles.len();
            tiles.push(GameTile {
                face_type: FaceType::Pentagon(pentagon_idx),
                state: TileState::Empty,
                color_override: None,
                highlighted: false,
                data: TileData::default(),
            });
            
            // Map triangles to this tile
            if let Some(triangle_indices) = mesh_data.face_mapping.get_pentagon_triangles(pentagon_idx) {
                for &triangle_idx in triangle_indices {
                    triangle_to_tile.insert(triangle_idx, tile_idx);
                }
            }
        }
        
        // Create tiles for hexagons
        for hexagon_idx in 0..polyhedron.hexagons.len() {
            let tile_idx = tiles.len();
            tiles.push(GameTile {
                face_type: FaceType::Hexagon(hexagon_idx),
                state: TileState::Empty,
                color_override: None,
                highlighted: false,
                data: TileData::default(),
            });
            
            // Map triangles to this tile
            if let Some(triangle_indices) = mesh_data.face_mapping.get_hexagon_triangles(hexagon_idx) {
                for &triangle_idx in triangle_indices {
                    triangle_to_tile.insert(triangle_idx, tile_idx);
                }
            }
        }
        
        println!("   Created {} tiles ({} pentagons, {} hexagons)", 
                 tiles.len(), polyhedron.pentagons.len(), polyhedron.hexagons.len());
        
        Self {
            polyhedron,
            mesh_data,
            tiles,
            triangle_to_tile,
            hovered_tile: None,
            selected_tile: None,
        }
    }
    
    /// Get a tile by index
    pub fn get_tile(&self, tile_idx: usize) -> Option<&GameTile> {
        self.tiles.get(tile_idx)
    }
    
    /// Get a mutable tile by index
    pub fn get_tile_mut(&mut self, tile_idx: usize) -> Option<&mut GameTile> {
        self.tiles.get_mut(tile_idx)
    }
    
    /// Get tile from triangle index (for mouse picking)
    pub fn get_tile_from_triangle(&self, triangle_idx: usize) -> Option<usize> {
        self.triangle_to_tile.get(&triangle_idx).copied()
    }
    
    /// Get the center position of a tile
    pub fn get_tile_center(&self, tile_idx: usize) -> Option<Vec3> {
        let tile = self.get_tile(tile_idx)?;
        match tile.face_type {
            FaceType::Pentagon(idx) => self.polyhedron.pentagons.get(idx).map(|p| p.center),
            FaceType::Hexagon(idx) => self.polyhedron.hexagons.get(idx).map(|h| h.center),
        }
    }
    
    /// Get the neighbors of a tile
    pub fn get_tile_neighbors(&self, tile_idx: usize) -> Vec<usize> {
        let tile = match self.get_tile(tile_idx) {
            Some(tile) => tile,
            None => return Vec::new(),
        };
        
        // TODO: Implement proper neighbor lookup
        // For now, return empty list
        Vec::new()
    }
    
    /// Set the state of a tile
    pub fn set_tile_state(&mut self, tile_idx: usize, state: TileState) -> bool {
        if let Some(tile) = self.get_tile_mut(tile_idx) {
            tile.state = state;
            true
        } else {
            false
        }
    }
    
    /// Set the color override for a tile
    pub fn set_tile_color(&mut self, tile_idx: usize, color: Option<Color>) -> bool {
        if let Some(tile) = self.get_tile_mut(tile_idx) {
            tile.color_override = color;
            true
        } else {
            false
        }
    }
    
    /// Highlight a tile
    pub fn highlight_tile(&mut self, tile_idx: usize, highlighted: bool) -> bool {
        if let Some(tile) = self.get_tile_mut(tile_idx) {
            tile.highlighted = highlighted;
            true
        } else {
            false
        }
    }
    
    /// Set the hovered tile
    pub fn set_hovered_tile(&mut self, tile_idx: Option<usize>) {
        // Clear previous hover
        if let Some(prev_hovered) = self.hovered_tile {
            self.highlight_tile(prev_hovered, false);
        }
        
        // Set new hover
        self.hovered_tile = tile_idx;
        if let Some(hovered) = tile_idx {
            self.highlight_tile(hovered, true);
        }
    }
    
    /// Set the selected tile
    pub fn set_selected_tile(&mut self, tile_idx: Option<usize>) {
        // Clear previous selection
        if let Some(prev_selected) = self.selected_tile {
            if let Some(tile) = self.get_tile_mut(prev_selected) {
                if tile.state == TileState::Selected {
                    tile.state = TileState::Empty;
                }
            }
        }
        
        // Set new selection
        self.selected_tile = tile_idx;
        if let Some(selected) = tile_idx {
            self.set_tile_state(selected, TileState::Selected);
        }
    }
    
    /// Get tiles by state
    pub fn get_tiles_by_state(&self, state: TileState) -> Vec<usize> {
        self.tiles.iter()
            .enumerate()
            .filter(|(_, tile)| tile.state == state)
            .map(|(idx, _)| idx)
            .collect()
    }
    
    /// Get tiles by type
    pub fn get_tiles_by_type(&self, pentagon: bool) -> Vec<usize> {
        self.tiles.iter()
            .enumerate()
            .filter(|(_, tile)| {
                if pentagon {
                    tile.face_type.is_pentagon()
                } else {
                    tile.face_type.is_hexagon()
                }
            })
            .map(|(idx, _)| idx)
            .collect()
    }
    
    /// Find tiles within a certain distance of a point
    pub fn find_tiles_near_point(&self, point: Vec3, max_distance: f32) -> Vec<usize> {
        self.tiles.iter()
            .enumerate()
            .filter_map(|(idx, tile)| {
                if let Some(center) = self.get_tile_center(idx) {
                    if center.distance(point) <= max_distance {
                        Some(idx)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Print statistics about the tile system
    pub fn print_statistics(&self) {
        let pentagon_count = self.get_tiles_by_type(true).len();
        let hexagon_count = self.get_tiles_by_type(false).len();
        
        let state_counts = [
            (TileState::Empty, self.get_tiles_by_state(TileState::Empty).len()),
            (TileState::Occupied, self.get_tiles_by_state(TileState::Occupied).len()),
            (TileState::Selected, self.get_tiles_by_state(TileState::Selected).len()),
            (TileState::Blocked, self.get_tiles_by_state(TileState::Blocked).len()),
            (TileState::Special, self.get_tiles_by_state(TileState::Special).len()),
        ];
        
        println!("\n🎯 Game Tile System Statistics:");
        println!("   Total tiles: {}", self.tiles.len());
        println!("   Pentagon tiles: {}", pentagon_count);
        println!("   Hexagon tiles: {}", hexagon_count);
        println!("   Tile states:");
        for (state, count) in state_counts {
            if count > 0 {
                println!("     {:?}: {}", state, count);
            }
        }
        
        if let Some(hovered) = self.hovered_tile {
            println!("   Hovered tile: {} ({:?})", hovered, self.get_tile(hovered).unwrap().face_type);
        }
        
        if let Some(selected) = self.selected_tile {
            println!("   Selected tile: {} ({:?})", selected, self.get_tile(selected).unwrap().face_type);
        }
    }
}

impl TileData {
    /// Set a property on this tile
    pub fn set_property(&mut self, key: impl Into<String>, value: TileProperty) {
        self.properties.insert(key.into(), value);
    }
    
    /// Get a property from this tile
    pub fn get_property(&self, key: &str) -> Option<&TileProperty> {
        self.properties.get(key)
    }
    
    /// Remove a property from this tile
    pub fn remove_property(&mut self, key: &str) -> Option<TileProperty> {
        self.properties.remove(key)
    }
    
    /// Check if this tile has a property
    pub fn has_property(&self, key: &str) -> bool {
        self.properties.contains_key(key)
    }
}

impl TileProperty {
    /// Try to get this property as an integer
    pub fn as_int(&self) -> Option<i32> {
        match self {
            TileProperty::Integer(i) => Some(*i),
            _ => None,
        }
    }
    
    /// Try to get this property as a float
    pub fn as_float(&self) -> Option<f32> {
        match self {
            TileProperty::Float(f) => Some(*f),
            TileProperty::Integer(i) => Some(*i as f32),
            _ => None,
        }
    }
    
    /// Try to get this property as a string
    pub fn as_string(&self) -> Option<&str> {
        match self {
            TileProperty::String(s) => Some(s),
            _ => None,
        }
    }
    
    /// Try to get this property as a boolean
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            TileProperty::Boolean(b) => Some(*b),
            _ => None,
        }
    }
    
    /// Try to get this property as a color
    pub fn as_color(&self) -> Option<Color> {
        match self {
            TileProperty::Color(c) => Some(*c),
            _ => None,
        }
    }
}

/// Default colors for different tile types and states
pub struct TileColors;

impl TileColors {
    pub const PENTAGON_DEFAULT: Color = Color::srgb(0.8, 0.3, 0.3); // Red-ish
    pub const HEXAGON_DEFAULT: Color = Color::srgb(0.3, 0.8, 0.3);  // Green-ish
    pub const PENTAGON_HOVER: Color = Color::srgb(1.0, 0.5, 0.5);   // Bright red
    pub const HEXAGON_HOVER: Color = Color::srgb(0.5, 1.0, 0.5);    // Bright green
    pub const SELECTED: Color = Color::srgb(1.0, 1.0, 0.3);         // Yellow
    pub const OCCUPIED: Color = Color::srgb(0.3, 0.3, 0.8);         // Blue
    pub const BLOCKED: Color = Color::srgb(0.5, 0.5, 0.5);          // Gray
    pub const SPECIAL: Color = Color::srgb(0.8, 0.3, 0.8);          // Purple
}

impl GameTile {
    /// Get the appropriate color for this tile based on its state
    pub fn get_display_color(&self) -> Color {
        // Color override takes precedence
        if let Some(color) = self.color_override {
            return color;
        }
        
        // State-based colors
        match self.state {
            TileState::Selected => TileColors::SELECTED,
            TileState::Occupied => TileColors::OCCUPIED,
            TileState::Blocked => TileColors::BLOCKED,
            TileState::Special => TileColors::SPECIAL,
            TileState::Empty => {
                if self.highlighted {
                    match self.face_type {
                        FaceType::Pentagon(_) => TileColors::PENTAGON_HOVER,
                        FaceType::Hexagon(_) => TileColors::HEXAGON_HOVER,
                    }
                } else {
                    match self.face_type {
                        FaceType::Pentagon(_) => TileColors::PENTAGON_DEFAULT,
                        FaceType::Hexagon(_) => TileColors::HEXAGON_DEFAULT,
                    }
                }
            }
        }
    }
}
