use crate::ai::working_memory::{FactQuery, FactQueryCheck, AIStimuli, WMProperty};
use crate::sensors::sensor_types::{SensorArguments, SensorPolling};
use crate::targeting::target::AITarget;
use serde::{Deserialize, Serialize};
use crate::ai::world_state::{DistanceToTarget, WorldStateProperty, WSProperty};


/// sensor responsible for reading distance to some visible target
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DistanceToTargetSensor {
    update_every: f64,
    last_update_delta: f64,
    distance_close: f32,
    distance_medium: f32,
    distance_far: f32,
}


impl SensorPolling for DistanceToTargetSensor {
    fn process(&mut self, delta: f64, args: &mut SensorArguments) -> bool {
        self.last_update_delta += delta;
        if self.last_update_delta < self.update_every {
            return false;
        }
        self.last_update_delta = 0.0;
        // bail if no target
        let Some(AITarget::Character(character_id, ..)) = args.blackboard.target.as_ref() else { return false };

        // bail if target is not visible
        let fact_query = FactQuery::with_check(FactQueryCheck::Match(
            WMProperty::AIStimuli(AIStimuli::Character(*character_id, None))
        ));
        let Some(WMProperty::AIStimuli(AIStimuli::Character(_, Some(position)))) = args.working_memory.find_fact(fact_query).map(|f| &f.f_type) else {
            args.blackboard.invalidate_target = true;
            args.world_state[WorldStateProperty::DistanceToTarget] = None;
            return false;
        };

        let distance_to_target = args.blackboard.thinker_position.distance_to(*position);

        if distance_to_target <= self.distance_close {
            args.world_state[WorldStateProperty::DistanceToTarget] = Some(WSProperty::DistanceToTarget(DistanceToTarget::Close));
        }
        else if distance_to_target <= self.distance_medium {
            args.world_state[WorldStateProperty::DistanceToTarget] = Some(WSProperty::DistanceToTarget(DistanceToTarget::Medium));
        }
        else if distance_to_target <= self.distance_far {
            args.world_state[WorldStateProperty::DistanceToTarget] = Some(WSProperty::DistanceToTarget(DistanceToTarget::Far));
        } else {
            args.world_state[WorldStateProperty::DistanceToTarget] = Some(WSProperty::DistanceToTarget(DistanceToTarget::OutsideReach));
        }
        false
    }
}
