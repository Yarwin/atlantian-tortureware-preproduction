use godot::prelude::*;
use strum_macros::EnumString;
use strum_macros::{AsRefStr, EnumIter};

#[derive(
    Default,
    Export,
    Var,
    GodotConvert,
    Debug,
    Clone,
    Eq,
    PartialEq,
    Copy,
    EnumString,
    EnumIter,
    AsRefStr,
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
    Pressure,
    PlayerPressure,
    Poison,
    Parry,
    Repair,
    Slime,
    SplashDamage,
    Stun,
    Toxic,
    Water,
    #[allow(clippy::upper_case_acronyms)]
    #[strum(disabled)]
    MAX,
}
