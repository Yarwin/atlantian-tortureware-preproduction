use bitflags::bitflags;
use crate::targeting::target_select_character::select_character;

bitflags! {
    #[derive(Debug)]
    pub struct TargetMask: u32 {
        const None = 0;
        /// patrol point select
        const PatrolPoint = 1;
        /// character select
        const VisibleCharacter = 1 << 1;
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
    pub fn priority() -> [(TargetMask, impl Fn()); 3] {
        [
            (TargetMask::CharacterAimingAtMe, select_character),
            (TargetMask::VisibleCharacter, select_character),
            (TargetMask::PatrolPoint, select_character),
        ]
    }

    pub fn get_valid_target_selector(&self) -> Option<impl Fn()> {
        for (bits, func) in TargetMask::priority() {
            if self.contains(bits) {
                return Some(func)
            }
        }
        None
    }
}