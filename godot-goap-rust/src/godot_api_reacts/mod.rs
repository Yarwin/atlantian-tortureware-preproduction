// A module containing React resources
// React resource is ued to create some commands that are later being executed by dispatcher

use godot::prelude::GodotClass;
use crate::act_react::game_effect_builder::register_effect_builder;

mod print_message;
mod call_method;
mod combine;
mod pickup;
mod grab;
pub mod fly;
mod apply_damage;


pub fn register_reacts_dispatch() {
    register_effect_builder::<call_method::CallMethodGameEffect>(call_method::CallMethodGameEffect::class_name().to_gstring());
    register_effect_builder::<combine::CombineInventoryItemGameEffect>(combine::CombineInventoryItemGameEffect::class_name().to_gstring());
    register_effect_builder::<pickup::PickupItemGameEffect>(pickup::PickupItemGameEffect::class_name().to_gstring());
    register_effect_builder::<print_message::PrintMessageGameEffect>(print_message::PrintMessageGameEffect::class_name().to_gstring());
    register_effect_builder::<grab::GrabGameEffect>(grab::GrabGameEffect::class_name().to_gstring());
    register_effect_builder::<fly::FlyGameEffect>(fly::FlyGameEffect::class_name().to_gstring());
    register_effect_builder::<apply_damage::ApplyDamageGameEffect>(apply_damage::ApplyDamageGameEffect::class_name().to_gstring());
}
