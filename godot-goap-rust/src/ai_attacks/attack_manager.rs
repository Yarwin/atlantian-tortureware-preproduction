use crate::ai::world_state::WorldState;
use crate::ai_attacks::attack_validators::{AttackValidator, AttackValidatorType};
/// attack subsystem is responsible for finding the best attack for given entity
/// and putting this information inside blackboard
use crate::animations::animation_data::AnimationProps;
use crate::sensors::sensor_types::ThinkerProcessArgs;
use godot::global::is_zero_approx;
use rand::prelude::*;
use rand::{rng};
use serde::{Deserialize, Serialize};

/// Immutable struct that keeps all the data related to given attack.
#[derive(Debug, Serialize, Deserialize)]
pub struct AttackData {
    /// Attack name
    name: String,
    required_state: WorldState,
    validators: Vec<AttackValidatorType>,
    /// default priority of given attack. Can be modified (for example entity might follow some attack with combo)
    default_weight: f64,
    weight_range: (f64, f64),
    /// Steps required to perform given attack (prepare/execute/recover)
    steps: Vec<AnimationProps>,
}

impl AttackData {
    pub fn is_valid(&self, args: &ThinkerProcessArgs) -> bool {
        for (id, validator) in self.validators.iter().enumerate() {
            if !validator.validate(args, self, &id) {
                return false;
            }
        }
        true
    }

    pub fn get_weight(&self) -> f64 {
        // bail if no randomization
        if is_zero_approx(self.weight_range.0) && is_zero_approx(self.weight_range.1) {
            return self.default_weight;
        }
        let mut rng = rng();
        let weight_mod = rng.random_range(self.weight_range.0..self.weight_range.1);
        self.default_weight + weight_mod
    }
}
