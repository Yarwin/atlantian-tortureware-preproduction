use std::collections::HashSet;
use std::fmt;
use std::fmt::Formatter;


/// a representation of singular row taken by given item
#[derive(Debug, PartialEq, Eq)]
pub struct RowSpace {
    /// Informs how far away from origin is given space in x direction ("right")
    pub col_offset: usize,
    /// Informs how far away from origin is given space in y direction ("down")
    pub row_offset: usize,
    /// how many spaces this item takes
    pub growth: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ItemSize {
    pub space: Vec<RowSpace>
}

impl fmt::Display for ItemSize {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let (width, height) = self.total_rectangular_space_taken();
        let mut fmt_grid = Grid::new(width, height);
        let r = fmt_grid.insert_item_at(1, (0, 0), &self);
        fmt_grid.fmt(f)
    }
}

impl ItemSize {
    /// returns width and height of total rectangular space taken
    pub fn total_rectangular_space_taken(&self) -> (usize, usize) {
        let (mut max_row, mut max_col) = (0, 0);
        for space in self.space.iter() {
            max_row = (space.row_offset + 1).max(max_row);
            max_col = (space.col_offset + space.growth).max(max_col);
        }
        (max_col, max_row)
    }
    /// creates new rectangular item size
    pub fn new_rectangular(x: usize, y: usize) -> Self {
        let mut taken_space = Vec::with_capacity(x);
        for row_i in 0..y {
            taken_space.push(
              RowSpace {
                  col_offset: 0,
                  row_offset: row_i,
                  growth: x
              });
        }
        ItemSize {
            space: taken_space
        }
    }
}



#[derive(Debug)]
pub enum InventoryResult {
    Inserted(Vec<usize>),
    Free(Vec<usize>),
    OutsideRange,
    Taken(HashSet<u32>),
}

/// A rectangular grid filled with items ids
#[derive(Debug, Default)]
pub struct Grid {
    pub array: Vec<Option<u32>>,
    pub width: usize,
    pub height: usize,
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut grid_strings = String::new();
        for i in 0..self.array.len() {
            grid_strings.push_str(&format!(" {} ", self.array[i].unwrap_or(0)));
            if (i + 1) % self.width == 0 {
                grid_strings.push('\n');
            }
        }
        write!(f, "{}", grid_strings)
    }
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        let array = vec![None; width * height];
        Self {
            array,
            width,
            height,
        }
    }

    pub fn free(&mut self, id: u32) {
        for item in self.array.iter_mut() {
            if let Some(i) = item {
                if *i == id {
                    *item = None;
                }
            }
        }
    }

    pub fn get_first_free_space(&self, size: ItemSize) -> InventoryResult {
        let (max_width, max_height) = size.total_rectangular_space_taken();
        for row in 0..(self.height + 1 - max_width) {
            for column in 0..(self.width + 1 - max_height) {
                match self.check_at((column, row), &size, None) {
                    InventoryResult::Free(free) => {
                        return InventoryResult::Free(free);
                    }
                    _ => continue,
                }
            }
        }
        InventoryResult::OutsideRange
    }

    pub fn force_insert(&mut self, ids: &[usize], item_id: u32) {
        for id in ids.iter() {
            self.array[*id] = Some(item_id);
        }
    }

    pub fn insert_item_at(
        &mut self,
        item_id: u32,
        at: (usize, usize),
        size: &ItemSize,
    ) -> InventoryResult {
        let result = self.check_at(at, size, Some(item_id));
        match result {
            InventoryResult::Free(ids) => {
                self.force_insert(&ids, item_id);
                InventoryResult::Inserted(ids)
            }
            _ => result,
        }
    }

    pub fn check_row_at(
        &self,
        at: (usize, usize),
        row_size: &RowSpace,
        item_id: Option<u32>,
    ) -> InventoryResult {
        let (row, col_start) = (at.1 + row_size.row_offset, at.0 + row_size.col_offset);
        if row > self.height || col_start + row_size.growth > self.width {
            return InventoryResult::OutsideRange;
        }
        let mut free: Vec<usize> = Vec::with_capacity(row_size.growth);
        let mut taken: HashSet<u32> = HashSet::with_capacity(row_size.growth);
        for x in col_start..(col_start + row_size.growth) {
            let coord = x + (row * self.width);
            if let Some(id) = self.array[coord] {
                if let Some(current_item) = item_id {
                    if current_item == id {
                        free.push(coord);
                        continue;
                    }
                }
                taken.insert(id);
            } else {
                free.push(coord);
            }
        }
        if taken.is_empty() {
            return InventoryResult::Free(free);
        }
        InventoryResult::Taken(taken)
    }

    pub fn check_at(
        &self,
        at: (usize, usize),
        size: &ItemSize,
        item_id: Option<u32>
    ) -> InventoryResult {
        let mut free = Vec::new();
        let mut taken = HashSet::new();
        for row_space in size.space.iter() {
            match self.check_row_at(at, row_space, item_id) {
                InventoryResult::Free(f) => {free.extend(f)}
                InventoryResult::OutsideRange => {
                    return InventoryResult::OutsideRange
                }
                InventoryResult::Taken(t) => {taken.extend(t)}
                InventoryResult::Inserted(_) => {panic!("logic error")}
            }
        }
        if taken.is_empty() {
            return InventoryResult::Free(free);
        }
        InventoryResult::Taken(taken)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_creating_item_sizes() {
        // two rows taking 2 spaces each
        let excepted_item_sizes = vec![
            ItemSize {
                space: vec![
                    RowSpace {row_offset: 0, col_offset: 0, growth: 2 }, RowSpace {row_offset: 1, col_offset: 0, growth: 2 }
                ],
            },
            ItemSize {
                space: vec![
                    RowSpace {row_offset: 0, col_offset: 0, growth: 3 }, RowSpace {row_offset: 1, col_offset: 0, growth: 3 }
                ],
            },
            ItemSize {
                space: vec![
                    RowSpace {row_offset: 0, col_offset: 0, growth: 2 }, RowSpace {row_offset: 1, col_offset: 0, growth: 2 }, RowSpace {row_offset: 2, col_offset: 0, growth: 2 }
                ],
            },
        ];
        let created_item_sizes = vec![
            ItemSize::new_rectangular(2, 2),
            ItemSize::new_rectangular(3, 2),
            ItemSize::new_rectangular(2, 3)
        ];
        for i in 0..excepted_item_sizes.len() {
            if excepted_item_sizes[i] != created_item_sizes[i] {
                panic!("failed creating rectangular item. Excepted: {:?}, given: {:?}", excepted_item_sizes[i], created_item_sizes[i])
            }
        }
    }

    #[test]
    fn test_inserting_space_free() {
        let mut grid = Grid::new(3, 3);
        let item_a = ItemSize {
            space: vec![
                RowSpace {row_offset: 0, col_offset: 1, growth: 2 },
                RowSpace {row_offset: 1, col_offset: 0, growth: 1 }, RowSpace {row_offset: 1, col_offset: 2, growth: 1 },
                RowSpace {row_offset: 2, col_offset: 0, growth: 3 },
            ]
        };
        let insert_result = grid.insert_item_at(1, (0, 0), &item_a);
        // excepted grid result:
        // 0  1  1
        // 1  0  1
        // 1  1  1
        if !matches!(insert_result, InventoryResult::Inserted(_)) {
            panic!("failed to insert an item. Excepted: InventoryResult::Inserted, actual: {:?}", insert_result);
        }
        if grid.array[0].is_some() || grid.array[4].is_some() {
            panic!("excepted grid indices 0 and 4 to be free!")
        }
        let item_b = ItemSize::new_rectangular(1, 1);
        let insert_result = grid.insert_item_at(2, (1, 1), &item_b);
        // excepted grid result:
        // 0  1  1
        // 1  2  1
        // 1  1  1
        if !matches!(insert_result, InventoryResult::Inserted(_)) {
            panic!("failed to insert an item. Excepted: InventoryResult::Inserted, actual: {:?}", insert_result);
        }
    }

    #[test]
    fn test_inserting_space_taken() {
        let mut grid = Grid::new(3, 3);
        let item_a = ItemSize::new_rectangular(2, 2);
        let _ = grid.insert_item_at(1, (0, 0), &item_a);
        let insert_result = grid.insert_item_at(2, (1, 1), &item_a);
        if !matches!(insert_result, InventoryResult::Taken(_)) {
            panic!("given space should be taken!")
        }
    }

    #[test]
    fn test_inserting_outside_space() {
        let mut grid = Grid::new(3, 3);
        let item_a = ItemSize::new_rectangular(3, 3);
        let insert_result = grid.insert_item_at(2, (1, 1), &item_a);
        if !matches!(insert_result, InventoryResult::OutsideRange) {
            panic!("Wrong result! Excepted: InventoryResult::OutsideRange, actual: {:?}", insert_result)
        }
    }

    #[test]
    fn test_removing() {
        let mut grid = Grid::new(3, 3);
        let item_a = ItemSize::new_rectangular(2, 2);
        let _ = grid.insert_item_at(1, (0, 0), &item_a);
        grid.free(1);
        let insert_result = grid.insert_item_at(2, (1, 1), &item_a);
        if !matches!(insert_result, InventoryResult::Inserted(_)) {
            panic!("given space should be free! Excepted: InventoryResult::Inserted, actual: {:?}", insert_result)
        }
    }

}