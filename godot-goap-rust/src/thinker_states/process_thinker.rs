use crate::ai::thinker::Thinker;
use crate::ai_nodes::ai_node::AINode;
use crate::sensors::sensor_types::{ThinkerProcessArgs, SensorPolling};
use crate::thinker_states::navigation_subsystem::{navigate, NavigationArguments};
use crate::thinker_states::types::StateArguments;
use godot::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::ai::world_state::WorldStateProperty;
use crate::ai::world_state::WSProperty::{Target, Truth};
use crate::targeting::targeting_systems::TargetMask;
use crate::thinker_states::polling::PollingResult;

pub fn process_thinker(
    thinker: &mut Thinker,
    delta: f64,
    ainodes: &Arc<RwLock<HashMap<u32, AINode>>>,
) {
    let mut polls = PollingResult::from_godot_thinker(thinker.base.as_ref().unwrap());
    let base = thinker.base.as_mut().unwrap();
    let Ok(mut shared_guard) = thinker.shared.lock() else {
        panic!("couldn't open thinker mutex!")
    };
    let shared = &mut *shared_guard;
    let mut sensor_args = ThinkerProcessArgs {
        id: thinker.id,
        character_rid: base.bind().character_body.as_ref().unwrap().get_rid(),
        head_position: base
            .bind()
            .head_position
            .as_ref()
            .map(|hp| hp.get_global_position())
            .unwrap_or(Vector3::ZERO),
        thinker_forward_axis: base
            .bind()
            .character_body
            .as_ref()
            .map(|ch| ch.get_global_basis().col_c())
            .unwrap_or(Vector3::ZERO),
        world_state: &mut shared.world_state,
        working_memory: &mut shared.working_memory,
        blackboard: &mut shared.blackboard,
        polls: &mut polls,
        target_mask: &mut shared.target_mask,
        ainodes,
    };

    // run polling sensors
    // todo – benchmark if rayon wouldn't do a job faster
    for sensor in thinker.polling_sensors.iter_mut() {
        sensor.process(delta, &mut sensor_args);
    }
    // update target selectors
    // todo – replace with some more sophisticated target selectors (sensors-like)
    if sensor_args.blackboard.invalidate_target {
        sensor_args.blackboard.target = None;
        let valid_target_selectors = TargetMask::valid_target_selectors(sensor_args.blackboard.valid_targets);
        for (_target_mask, target_type, target_selector) in valid_target_selectors {
            if let Some(target) = target_selector(&mut sensor_args) {
                sensor_args.blackboard.invalidate_target = false;
                sensor_args.blackboard.target = Some(target);
                sensor_args.world_state[WorldStateProperty::HasTarget] = Some(Target(target_type));
                break
            }
        }
        if sensor_args.blackboard.target.is_none() {
            sensor_args.world_state[WorldStateProperty::HasTarget] = Some(Truth(false));
        }
    }


    // state change
    let new_bb_state = shared.blackboard.new_state.take();
    if let Some(mut new_state) = new_bb_state {
        let mut state_args = StateArguments {
            base: base.clone(),
            world_state: &mut shared.world_state,
            working_memory: &mut shared.working_memory,
            blackboard: &mut shared.blackboard,
            delta,
        };
        if let Some(mut old_state) = thinker.state.take() {
            old_state.exit(&mut state_args);
        }
        new_state.enter(state_args);
        thinker.state = Some(new_state);
    }

    // run state
    if let Some(mut state) = thinker.state.take() {
        let state_args = StateArguments {
            base: base.clone(),
            world_state: &mut shared.world_state,
            working_memory: &mut shared.working_memory,
            blackboard: &mut shared.blackboard,
            delta,
        };
        state.physics_process(delta, state_args);
        thinker.state = Some(state);
    }
    // run navigation subsystem
    let navigation_arguments = NavigationArguments {
        base: base.clone(),
        blackboard: &mut shared.blackboard,
        navigation_data: &mut thinker.navigation_data,
        delta,
    };
    navigate(navigation_arguments, delta);
}
