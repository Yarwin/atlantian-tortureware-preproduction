// A module containing React resources
// React resource is ued to create some commands that are later being executed by dispatcher

use crate::act_react::game_effect_builder::register_effect_builder;
use godot::prelude::GodotClass;

mod apply_damage;
mod call_method;
mod combine;
pub mod fly;
mod grab;
mod pickup;
mod print_message;

pub fn register_reacts_dispatch() {
    register_effect_builder::<call_method::CallMethodGameEffect>(
        call_method::CallMethodGameEffect::class_name().to_gstring(),
    );
    register_effect_builder::<combine::CombineInventoryItemGameEffect>(
        combine::CombineInventoryItemGameEffect::class_name().to_gstring(),
    );
    register_effect_builder::<pickup::PickupItemGameEffect>(
        pickup::PickupItemGameEffect::class_name().to_gstring(),
    );
    register_effect_builder::<print_message::PrintMessageGameEffect>(
        print_message::PrintMessageGameEffect::class_name().to_gstring(),
    );
    register_effect_builder::<grab::GrabGameEffect>(
        grab::GrabGameEffect::class_name().to_gstring(),
    );
    register_effect_builder::<fly::FlyGameEffect>(fly::FlyGameEffect::class_name().to_gstring());
    register_effect_builder::<apply_damage::ApplyDamageGameEffect>(
        apply_damage::ApplyDamageGameEffect::class_name().to_gstring(),
    );
}
