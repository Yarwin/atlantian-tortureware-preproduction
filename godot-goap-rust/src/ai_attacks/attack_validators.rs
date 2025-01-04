use crate::ai::working_memory::{AIStimuli, Event, FactQuery, FactQueryCheck, WMProperty};
use crate::ai::world_state::WorldState;
use crate::ai_attacks::attack_manager::AttackData;
use crate::sensors::sensor_types::ThinkerProcessArgs;
use crate::targeting::target::AITarget;
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

#[enum_dispatch]
#[derive(Debug, Serialize, Deserialize)]
pub enum AttackValidatorType {
    AttackCooldown,
    AttackWorldStateRequirement,
    AttackVisibleWithinRange,
}

#[enum_dispatch(AttackValidatorType)]
pub trait AttackValidator {
    fn validate(&self, args: &ThinkerProcessArgs, attack: &AttackData, attack_id: &usize) -> bool;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AttackCooldown {
    /// How fast given attack can be executed again.
    /// Information about given attack is being kept in working memory.
    cooldown: f64,
}

impl AttackValidator for AttackCooldown {
    fn validate(&self, args: &ThinkerProcessArgs, _attack: &AttackData, attack_id: &usize) -> bool {
        let query = FactQuery::with_check(FactQueryCheck::Match(WMProperty::Event(
            Event::AttackPerformed { id: *attack_id },
        )));
        if let Some(fact) = args.working_memory.find_fact(query) {
            if fact.update_time.elapsed().unwrap().as_secs_f64() < self.cooldown {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AttackVisibleWithinRange {
    min: f32,
    max: f32,
}

impl AttackValidator for AttackVisibleWithinRange {
    fn validate(
        &self,
        args: &ThinkerProcessArgs,
        _attack: &AttackData,
        _attack_id: &usize,
    ) -> bool {
        let Some(AITarget::Character(character_id, ..)) = args.blackboard.target.as_ref() else {
            return false;
        };
        let fact_query = FactQuery::with_check(FactQueryCheck::Match(WMProperty::AIStimuli(
            AIStimuli::Character(*character_id, None),
        )));

        let Some(WMProperty::AIStimuli(AIStimuli::Character(_, Some(position)))) =
            args.working_memory.find_fact(fact_query).map(|f| &f.f_type)
        else {
            return false;
        };

        let distance_to_target = args.blackboard.thinker_position.distance_to(*position);
        self.min < distance_to_target && distance_to_target < self.max
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AttackWorldStateRequirement {
    required_world_state: WorldState,
}

impl AttackValidator for AttackWorldStateRequirement {
    fn validate(
        &self,
        args: &ThinkerProcessArgs,
        _attack: &AttackData,
        _attack_id: &usize,
    ) -> bool {
        self.required_world_state
            .count_state_differences(args.world_state)
            == 0
    }
}
