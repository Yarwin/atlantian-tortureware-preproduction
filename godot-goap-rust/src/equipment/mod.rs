use godot::prelude::GodotClass;
pub mod equip_component;
pub mod gun_ui;
mod spreadgun;

pub fn register_equipment_dispatch() {
    equip_component::register_item_equipment_component::<spreadgun::SpreadGunResource>(
        spreadgun::SpreadGunResource::class_name().to_gstring(),
    );
    equip_component::register_equipment_component::<spreadgun::SpreadGun>(
        spreadgun::SpreadGun::class_name().to_gstring(),
    );
}
