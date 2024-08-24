use godot::prelude::*;

#[derive(
    Default, Export, Var, GodotConvert, Debug, Clone, Eq, PartialEq, Copy
)]
#[repr(usize)]
#[godot(via = i32)]
pub enum Stimuli {
    Cold,
    Combine,
    DamageStandard,
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
    Parry,
    Repair,
    Slime,
    SplashDamage,
    Stun,
    Toxic,
    Water,
    MAX
}
