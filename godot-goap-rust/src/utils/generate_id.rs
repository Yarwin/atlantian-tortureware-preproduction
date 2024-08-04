

pub fn assign_id(initial_id: u32, current_max: &mut u32) -> u32 {
    if initial_id != 0 {
        if *current_max < initial_id {
            *current_max = initial_id;
        }
        return initial_id
    }
    *current_max += 1;
    *current_max
}