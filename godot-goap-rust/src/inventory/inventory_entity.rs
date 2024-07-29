use std::collections::HashSet;
use crate::inventory::grid::Grid;


/// A struct that represents a single inventory
#[derive(Debug)]
pub struct InventoryEntity {
    grid: Grid,
}

impl InventoryEntity {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            grid: Grid::new(width, height),
        }
    }

    /// returns all items ids in this inventory
    pub fn get_items(&self) -> HashSet<u32> {
        self.grid
            .array
            .iter()
            .flatten()
            .copied()
            .collect::<HashSet<u32>>()
    }
}