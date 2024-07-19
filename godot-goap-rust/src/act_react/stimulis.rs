use godot::prelude::*;

#[derive(
    Default, Export, Var, GodotConvert, Debug, Clone, Eq, PartialEq,
)]
#[godot(via = i32)]
pub enum Stimuli {
    Cold,
    Combine,
    DamageBash,
    DamagePierce,
    DamageSlash,
    Electrify,
    Fire,
    #[default]
    Frob,
    Grab,
    Heat,
    Kick,
    Pain,
    PlayerFrob,
    Poison,
    Repair,
    Slime,
    Stun,
    Toxic,
    Water,
}