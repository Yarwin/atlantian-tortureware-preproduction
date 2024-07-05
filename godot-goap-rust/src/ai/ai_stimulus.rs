#[derive(Debug, Default, Eq, PartialEq)]
pub enum AIStimulusType {
    #[default]
    None,
    CharacterVisible,
    LeanVisible,
    WeaponFireSound,
    WeaponReloadSound,
    WeaponImpactSound,
    WeaponImpactVisible,
    FootstepSound,
    AlarmSound,
    DisturbanceSound,
    DisturbanceVisible,
    DangerVisible,
    DeathVisible,
    DeathSound,
    PainSound,
    DamageBullet,
    DamageExplode,
    DamageMelee,
    DamageStun,
    CombatOpportunity
}