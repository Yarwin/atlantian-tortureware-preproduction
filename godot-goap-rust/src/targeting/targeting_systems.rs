use crate::targeting::target_select_character::select_character;
use bitflags::bitflags;
use crate::sensors::sensor_types::SensorArguments;

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct TargetMask: u32 {
        const None = 0;
        /// patrol point select
        /// character select
        const VisibleCharacter = 1;
        /// Damager Select
        const Damager = 1 << 1;
        const Interest = 1 << 2;
        const Node = 1 << 3;
        const Danger = 1 << 4;
        const CharacterAimingAtMe = 1 << 5;
    }
}

impl Default for TargetMask {
    fn default() -> Self {
        TargetMask::None
    }
}

impl TargetMask {
    fn priority() -> [(TargetMask, for<'a, 'b> fn(&'a mut SensorArguments<'b>)); 1]
    {
        [
            // (TargetMask::CharacterAimingAtMe, select_character),
            (TargetMask::VisibleCharacter, select_character),
        ]
    }

    pub fn valid_target_selectors(other: TargetMask) -> impl Iterator<Item=(TargetMask, for<'a, 'b> fn(&'a mut SensorArguments<'b>))> + 'static
    {
        TargetMask::priority().into_iter().filter(move |(bits, _func)| other.contains(*bits))
    }
}
