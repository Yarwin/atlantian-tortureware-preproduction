use std::time::SystemTime;
use crate::ai::working_memory::{FactQuery, FactQueryCheck, Knowledge, Stimuli, WorkingMemoryFactType};
use crate::godot_api::godot_visible_area_3d::GodotVisibilityArea3D;
use crate::sensors::sensor_types::{SensorArguments, SensorPolling};
use godot::classes::{PhysicsRayQueryParameters3D, PhysicsServer3D};
use godot::prelude::*;
use serde::{Deserialize, Serialize};
use crate::ai::working_memory::Desire::Surprise;
use crate::targeting::targeting_systems::TargetMask;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionCharacterSensor {
    update_every: f64,
    last_update_delta: f64,
}

impl SensorPolling for VisionCharacterSensor {
    fn process(&mut self, delta: f64, args: &mut SensorArguments) -> bool {
        self.last_update_delta += delta;
        if self.last_update_delta < self.update_every {
            return false;
        }
        self.last_update_delta = 0.0;

        if args.polls.get_visible().is_none() {
            return false;
        }

        for target in args.polls.get_visible().unwrap() {
            // bail if target outside vision cone
            if args.thinker_forward_axis.dot(
                args.head_position
                    .direction_to(target.area_transform.origin),
            ) < 0.12
            {
                continue;
            }

            let mut is_target_visible: bool = false;
            let target_height = target.area_transform.basis.col_b() * target.shape_height;

            // visibility check
            // todo –  project ray targets on a clipped plane that uses area direction to the head as its normal
            // we assume that target's visible area is always a cylinder

            let raycast_directions = [
                target.area_transform.origin + target_height / 2.0, // midpoint
                target.area_transform.origin + target_height,       // top
                target.area_transform.origin,                       // bottom
                target.area_transform.origin
                    + target_height
                    + target.area_transform.basis.col_a() * target.shape_radius, // """left"""
                target.area_transform.origin + target_height
                    - target.area_transform.basis.col_a() * target.shape_radius, // """right"""
            ];

            let mut ray_params = PhysicsRayQueryParameters3D::new_gd();
            ray_params.set_collision_mask(4);
            ray_params.set_from(args.head_position);
            ray_params.set_collide_with_areas(true);
            ray_params.set_collide_with_bodies(true);
            ray_params.set_exclude(array![args.character_rid]);
            let space_rid = PhysicsServer3D::singleton().body_get_space(args.character_rid);
            let Some(mut direct_space) =
                PhysicsServer3D::singleton().space_get_direct_state(space_rid)
            else {
                return false;
            };

            let mut distance_to_target: f32 = 0.0;
            let mut character_id: Option<InstanceId> = None;

            for raycast_target in raycast_directions {
                ray_params.set_to(raycast_target);
                let intersection_result = direct_space.intersect_ray(ray_params.clone());
                is_target_visible = intersection_result
                    .get("rid")
                    .map(|r| r.to::<Rid>() == target.area_rid)
                    .unwrap_or(false);

                // bail if we see a target
                if is_target_visible {
                    distance_to_target = intersection_result
                        .get("position")
                        .map(|v| v.to::<Vector3>().distance_to(args.head_position))
                        .unwrap();
                    character_id = intersection_result.get("collider").map(|v| {
                        let area = v.to::<Gd<GodotVisibilityArea3D>>();
                        let instance_id = area.bind().owner.as_ref().unwrap().instance_id();
                        instance_id
                    });
                    break;
                }
            }

            // bail if target is not visible
            if !is_target_visible {
                continue;
            }
            // get detection strength – it takes at least two updates to spot one target
            let detection_strength = ((11.0 - distance_to_target.min(11.0)) / 11.0).min(1.);
            let mut previous_stimulation: f32 = 0.0;
            let current_stimulation: f32;
            let fact_query =
                FactQuery::with_check(
                    FactQueryCheck::Match(
                        WorkingMemoryFactType::Stimuli(Stimuli::Character(character_id.unwrap()))
                    )
                );
            // update fact
            if let Some(fact) = args.working_memory.find_fact_mut(fact_query) {
                fact.update_time = SystemTime::now();
                previous_stimulation = fact.confidence;
                fact.confidence = (fact.confidence + detection_strength).min(1.1);
                current_stimulation = fact.confidence;
            } else {
                args.working_memory.add_working_memory_fact(
                    WorkingMemoryFactType::Stimuli(Stimuli::Character(character_id.unwrap())),
                    detection_strength,
                    // Store a stimuli for four updates – don't keep it if character isn't actually visible
                    self.update_every * 3.0,
                );
                current_stimulation = detection_strength;
            }

            // new target spotted
            if current_stimulation >= 1.0 && previous_stimulation < 1.0 {
                let fact_query =
                    FactQuery::with_check(
                        FactQueryCheck::Match(WorkingMemoryFactType::Knowledge(Knowledge::Character(character_id.unwrap()))));
                if let Some(fact) = args.working_memory.find_fact_mut(fact_query) {
                    // update
                    fact.confidence = distance_to_target;
                    fact.update_time = SystemTime::now();
                } else {
                    godot_print!("new character spotted!");
                    // new character spotted!!
                    args.working_memory.add_working_memory_fact(
                        WorkingMemoryFactType::Knowledge(Knowledge::Character(character_id.unwrap())),
                        distance_to_target,
                        30.0
                    );
                    // add desire to be surprised upon spotting new enemy
                    args.working_memory.add_working_memory_fact(
                        WorkingMemoryFactType::Desire(Surprise),
                        1.0,
                        15.0
                    );
                }
                // force retargeting
                args.blackboard.invalidate_target = true;
                args.blackboard.valid_targets = args.blackboard.valid_targets.union(TargetMask::VisibleCharacter);
            }
        }
        false
    }
}
