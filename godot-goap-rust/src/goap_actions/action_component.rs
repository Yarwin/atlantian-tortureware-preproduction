use crate::ai::world_state::WorldState;
use crate::animations::animation_data::AnimationType;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ActionComponent {
    pub cost: u32,
    pub preconditions: WorldState,
    pub effects: WorldState,
    pub animation: AnimationType,
}

impl Hash for ActionComponent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.preconditions.hash(state);
        self.effects.hash(state);
    }
}

impl Eq for ActionComponent {}

impl PartialEq for ActionComponent {
    fn eq(&self, other: &Self) -> bool {
        if self
            .preconditions
            .count_state_differences(&other.preconditions)
            == 0
            && self.effects.count_state_differences(&other.effects) == 0
        {
            return true;
        }
        false
    }
}
