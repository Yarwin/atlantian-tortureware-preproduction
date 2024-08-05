use std::collections::HashSet;
use std::fmt;
use std::fmt::Formatter;
use crate::godot_api::item_object::Item;
use crate::inventory::grid::{Grid, InventoryResult};
use godot::prelude::*;

pub enum InventoryEntityResult {
    FreeSpace(Vec<usize>, Gd<Item>),
    SpaceTaken(Vec<(usize, u32)>, Gd<Item>),
    NoSpaceForItem(Gd<Item>),
    WrongItemType(Gd<Item>),
    ItemDepleted,
}

impl InventoryEntityResult {
    pub fn item(self) -> Gd<Item> {
        match self {
            InventoryEntityResult::SpaceTaken(_, item) |
            InventoryEntityResult::NoSpaceForItem(item) |
            InventoryEntityResult::WrongItemType(item) |
            InventoryEntityResult::FreeSpace(_, item)
            => item,
            _ => panic!("given item no longer exists!")
        }
    }
}


/// A struct that represents a single inventory
#[derive(Debug)]
pub struct InventoryEntity {
    grid: Grid,
}

impl fmt::Display for InventoryEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.grid, f)
    }
}

impl InventoryEntity {
    pub fn index_to_coord(&self, index: usize) -> Vector2i {
        let row = index / self.grid.width;
        let col = index - row * self.grid.width;
        Vector2i::new(col as i32, row as i32)
    }

    pub fn new(width: usize, height: usize) -> Self {
        Self {
            grid: Grid::new(width, height),
        }
    }

    pub fn remove_item(&mut self, item_id: u32) {
        self.grid.free(item_id);
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

    /// returns (width, height)
    pub fn get_size(&self) -> (usize, usize) {
        (self.grid.width, self.grid.height)
    }

    pub fn check_at(&self, item: Gd<Item>, position_idx: usize) -> InventoryEntityResult {
        let result = self.grid.check_at(position_idx, item.bind().inventory.as_ref().unwrap().inventory_data.bind().get_size_force(), Some(item.bind().id));
        match result {
            InventoryResult::Free(free) => {InventoryEntityResult::FreeSpace(free, item)}
            InventoryResult::OutsideRange => {InventoryEntityResult::NoSpaceForItem(item)}
            InventoryResult::Taken(by) => {InventoryEntityResult::SpaceTaken(by, item)}
            _ => !unreachable!(),
        }
    }

    pub fn insert_at_first_free_space(&mut self, mut item: Gd<Item>) -> Result<Gd<Item>, InventoryEntityResult> {
        let mut item_bind = item.bind_mut();
        let item_id = item_bind.id;
        let Some(inventory_component) = item_bind.inventory.as_mut() else {
            std::mem::drop(item_bind);
            return Err(InventoryEntityResult::WrongItemType(item))
        };
        let space_check = self.grid.get_first_free_space(inventory_component.inventory_data.bind_mut().get_size());
        match space_check {
            InventoryResult::Free(ids) => {
                std::mem::drop(item_bind);
                self.grid.force_insert(item_id, &ids);
                item.bind_mut().inventory.as_mut().unwrap().location = self.index_to_coord(ids[0]);
                Ok(item)
            }
            InventoryResult::OutsideRange => {
                std::mem::drop(item_bind);
                Err(InventoryEntityResult::NoSpaceForItem(item))
            }
            _ => unreachable!()
        }
    }

    pub fn get_item_id_at(&self, at: usize) -> Option<u32> {
        self.grid.array[at]
    }


    /// tries to insert item at given index.
    pub fn try_insert_item_at(&mut self, mut item: Gd<Item>, at: usize) -> Result<Gd<Item>, InventoryEntityResult> {
        let mut item_bind = item.bind_mut();
        let item_id = item_bind.id;
        let Some(inventory_component) = item_bind.inventory.as_mut() else {
            std::mem::drop(item_bind);
            return Err(InventoryEntityResult::WrongItemType(item))
        };
        let currently_occupied_space: Vec<usize> = self
            .grid
            .array
            .iter()
            .enumerate()
            .filter_map(|(i, id)|
                {
                    let mut result = None;
                    if id.map(|s_id| s_id == item_id).unwrap_or(false) {
                        result = Some(i);
                    }
                    result
                }
            ).collect();
        let result = self.grid.insert_item_at(
            item_id,
            at,
            inventory_component.inventory_data.bind_mut().get_size()
        );
        std::mem::drop(item_bind);

        match result {
            InventoryResult::Inserted(at) => {
                let shared = currently_occupied_space.into_iter().filter(|id| !at.contains(id)).collect::<Vec<usize>>();
                self.grid.clear(&shared);
                item.bind_mut().inventory.as_mut().unwrap().location = self.index_to_coord(at[0]);
                Ok(item)
            },
            InventoryResult::OutsideRange => Err(InventoryEntityResult::NoSpaceForItem(item)),
            InventoryResult::Taken(ids) => {Err(InventoryEntityResult::SpaceTaken(ids, item))}
            _ => unreachable!()
        }
    }
}
