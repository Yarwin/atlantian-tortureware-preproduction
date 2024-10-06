use crate::targeting::target_select_character::select_character;
use bitflags::bitflags;
use crate::sensors::sensor_types::ThinkerProcessArgs;
use crate::targeting::target::{AITarget, TargetType};


pub type TargetSelectorWithMask = (TargetMask, TargetType, for<'a, 'b> fn(&'a mut ThinkerProcessArgs<'b>) -> Option<AITarget>);

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct TargetMask: u32 {
        const None = 0;
        /// character select
        const VisibleCharacter = 1;
        const KnownCharacter = 1 << 1;
        /// Damager Select
        const Damager = 1 << 2;
        const Interest = 1 << 3;
        const Node = 1 << 4;
        const Danger = 1 << 5;
        const CharacterAimingAtMe = 1 << 6;
    }
}

impl Default for TargetMask {
    fn default() -> Self {
        TargetMask::None
    }
}

impl TargetMask {
    fn priority() -> [TargetSelectorWithMask; 1]
    {
        [
            (TargetMask::VisibleCharacter, TargetType::Character, select_character),
        ]
    }

    pub fn valid_target_selectors(other: TargetMask) -> impl Iterator<Item=(TargetMask, TargetType, for<'a, 'b> fn(&'a mut ThinkerProcessArgs<'b>) -> Option<AITarget>)> + 'static
    {
        TargetMask::priority().into_iter().filter(move |(bits, _target, _func)| other.contains(*bits))
    }
}
