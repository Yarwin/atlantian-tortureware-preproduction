use std::fmt;
use std::fmt::Formatter;


/// a representation of singular row taken by given item
#[derive(Debug, PartialEq, Eq)]
pub struct RowSpace {
    /// Specifies how far away from origin is given space in x direction ("right")
    pub col_offset: usize,
    /// Specifies how far away from origin is given space in y direction ("down")
    pub row_offset: usize,
    /// Specifies how many spaces in the row this item takes
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
        fmt_grid.insert_item_at(1,  0, self);
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
    pub fn new_rectangular(width: usize, height: usize) -> Self {
        let mut taken_space = Vec::with_capacity(width);
        for row_i in 0..height {
            taken_space.push(
              RowSpace {
                  col_offset: 0,
                  row_offset: row_i,
                  growth: width
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
    Taken(Vec<(usize, u32)>),
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

    pub fn clear(&mut self, ids: &[usize]) {
        for id in ids {
            self.array[*id] = None;
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

    pub fn get_first_free_space(&self, size: &ItemSize) -> InventoryResult {
        let (rec_width, rec_height) = size.total_rectangular_space_taken();

        for row in 0..(self.height - rec_height + 1) {
            for column in 0..(self.width + 1 - rec_width) {
                match self.check_at(row * self.width + column, size, None) {
                    InventoryResult::Free(free) => {
                        return InventoryResult::Free(free);
                    }
                    _ => continue,
                }
            }
        }
        InventoryResult::OutsideRange
    }

    pub fn force_insert(&mut self, item_id: u32, ids: &[usize]) {
        for id in ids.iter() {
            self.array[*id] = Some(item_id);
        }
    }

    pub fn insert_item_at(
        &mut self,
        item_id: u32,
        at: usize,
        size: &ItemSize,
    ) -> InventoryResult {
        let result = self.check_at(at, size, Some(item_id));
        match result {
            InventoryResult::Free(ids) => {
                self.force_insert(item_id, &ids);
                InventoryResult::Inserted(ids)
            }
            _ => result,
        }
    }

    fn check_row_at(
        &self,
        item_id: Option<u32>,
        at: usize,
        row_size: &RowSpace,
    ) -> InventoryResult {
        let row = at / self.width + row_size.row_offset;
        let col_start = at % self.width + row_size.col_offset;
        // let (row, col_start) = (at / self.width + row_size.row_offset, at.1 + row_size.col_offset);
        if row >= self.height || col_start + row_size.growth > self.width {
            return InventoryResult::OutsideRange;
        }
        let mut free: Vec<usize> = Vec::with_capacity(row_size.growth);
        let mut taken: Vec<(usize, u32)> = Vec::with_capacity(row_size.growth);
        for x in col_start..(col_start + row_size.growth) {
            let coord = x + (row * self.width);
            if let Some(id) = self.array[coord] {
                if let Some(current_item) = item_id {
                    if current_item == id {
                        free.push(coord);
                        continue;
                    }
                }
                taken.push((coord, id));
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
        at: usize,
        size: &ItemSize,
        item_id: Option<u32>
    ) -> InventoryResult {
        let mut free = Vec::new();
        let mut taken = Vec::new();
        for row_space in size.space.iter() {
            match self.check_row_at(item_id, at, row_space) {
                InventoryResult::Free(f) => {free.extend(f)}
                InventoryResult::OutsideRange => {
                    return InventoryResult::OutsideRange
                }
                InventoryResult::Taken(t) => {taken.extend(t)}
                _ => unreachable!()
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
        let excepted_item_sizes = [ItemSize {
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
            }];
        let created_item_sizes = [ItemSize::new_rectangular(2, 2),
            ItemSize::new_rectangular(3, 2),
            ItemSize::new_rectangular(2, 3)];
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
        let insert_result = grid.insert_item_at(1, 0, &item_a);
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
        let insert_result = grid.insert_item_at(2, 4, &item_b);
        // excepted grid result:
        // 0  1  1
        // 1  2  1
        // 1  1  1
        if !matches!(insert_result, InventoryResult::Inserted(_)) {
            panic!("failed to insert an item. Excepted: InventoryResult::Inserted, actual: {:?}", insert_result);
        }
    }

    #[test]
    fn test_inserting_first_free_space() {
        let mut grid = Grid::new(3, 3);
        let item_a = ItemSize::new_rectangular(2, 3);
        let _ = grid.insert_item_at(1, 0, &item_a);
        let item_b = ItemSize::new_rectangular(1, 2);
        let InventoryResult::Free(first_free_space) = grid.get_first_free_space(&item_b) else {panic!("no free space found!")};
        if first_free_space != vec![2, 5] {
            panic!("wrong free space!");
        }
        grid.force_insert(2, &first_free_space);
        // excepted grid result:
        // 1  1  2
        // 1  1  2
        // 1  1  0

        let item_c = ItemSize::new_rectangular(1, 1);
        let InventoryResult::Free(first_free_space) = grid.get_first_free_space(&item_c) else {panic!("no free space found!")};
        if first_free_space != vec![8] {
            panic!("wrong free space!");
        }
    }

    #[test]
    fn test_inserting_space_taken() {
        let mut grid = Grid::new(3, 3);
        let item_a = ItemSize::new_rectangular(2, 2);
        let _ = grid.insert_item_at(1, 0, &item_a);
        let insert_result = grid.insert_item_at(2, 4, &item_a);
        if !matches!(insert_result, InventoryResult::Taken(_)) {
            panic!("Wrong result! Excepted: InventoryResult::Taken, actual: {:?}", insert_result)
        }
    }

    #[test]
    fn test_inserting_outside_space() {
        let mut grid = Grid::new(3, 3);
        let item_a = ItemSize::new_rectangular(3, 3);
        let insert_result = grid.insert_item_at(2, 4, &item_a);
        if !matches!(insert_result, InventoryResult::OutsideRange) {
            panic!("Wrong result! Excepted: InventoryResult::OutsideRange, actual: {:?}", insert_result)
        }
    }

    #[test]
    fn test_removing() {
        let mut grid = Grid::new(3, 3);
        let item_a = ItemSize::new_rectangular(2, 2);
        let _ = grid.insert_item_at(1, 0, &item_a);
        assert!(grid.array[0].is_some());
        grid.free(1);
        let insert_result = grid.insert_item_at(2, 4, &item_a);
        if !matches!(insert_result, InventoryResult::Inserted(_)) {
            panic!("given space should be free! Excepted: InventoryResult::Inserted, actual: {:?}", insert_result)
        }
        assert!(grid.array[0].is_none());
        assert!(grid.array[5].is_some());
    }
}
