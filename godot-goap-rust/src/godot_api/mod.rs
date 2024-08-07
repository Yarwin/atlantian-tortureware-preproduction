pub(crate) mod ai_manager;
pub(crate) mod godot_thinker;
pub(crate) mod godot_visible_area_3d;
pub mod inventory_manager;
pub mod item_object;
pub mod godot_inventory;
pub mod gamesys;

#[allow(dead_code)]
pub(crate) const CONNECT_DEFERRED: u32 = 1 << 0;
#[allow(dead_code)]
const CONNECT_PERSIST: u32 = 1 << 1;
pub(crate) const CONNECT_ONE_SHOT: u32 = 1 << 2;
#[allow(dead_code)]
const CONNECT_REFERENCE_COUNTED: u32 = 1 << 3;
