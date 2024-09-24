use std::time::SystemTime;
use crate::ai::working_memory::{FactQuery, FactQueryCheck, Knowledge, AIStimuli, WMProperty};
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
            // bail if target outside vision cone (~73 deg)
            if args.thinker_forward_axis.dot(
                args.head_position
                    .direction_to(target.area_transform.origin),
            ) < 0.3
            {
                continue;
            }

            let mut is_target_visible: bool = false;
            let target_height = target.area_transform.basis.col_b() * target.shape_height;

            // visibility check
            // todo –  project ray targets on a clipped plane that uses reflected area's direction to the head as its normal
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
            let mut see_point: Option<Vector3> = None;

            for raycast_target in raycast_directions {
                ray_params.set_to(raycast_target);
                let intersection_result = direct_space.intersect_ray(ray_params.clone());
                let Some(intersection_point) = intersection_result
                    .get("position")
                    .map(|v| v.to::<Vector3>()) else {continue};

                is_target_visible = intersection_result
                    .get("rid")
                    .map(|r| r.to::<Rid>() == target.area_rid)
                    .unwrap_or(false);

                // bail if we see a target
                if is_target_visible {
                    distance_to_target = intersection_point.distance_to(args.head_position);
                    character_id = intersection_result.get("collider").map(|v| {
                        let area = v.to::<Gd<GodotVisibilityArea3D>>();
                        let instance_id = area.bind().owner.as_ref().unwrap().instance_id();
                        instance_id
                    });
                    see_point = Some(intersection_point);
                    break;
                }
            }
            // bail if target is not visible
            if !is_target_visible {
                continue;
            }
            // get detection strength – it takes at least two updates to spot one target
            let detection_strength = ((16.0 - distance_to_target.min(16.0)) / 16.0).min(1.);
            let mut previous_stimulation: f32 = 0.0;
            let current_stimulation: f32;
            let fact_query =
                FactQuery::with_check(
                    FactQueryCheck::Match(
                        WMProperty::AIStimuli(AIStimuli::Character(character_id.unwrap(), None))
                    )
                );
            // update fact
            if let Some(fact) = args.working_memory.find_fact_mut(fact_query) {
                fact.update_time = SystemTime::now();
                previous_stimulation = fact.confidence;
                fact.confidence = (fact.confidence + detection_strength).min(1.1);
                current_stimulation = fact.confidence;
                if let WMProperty::AIStimuli(AIStimuli::Character(_i, pos)) = &mut fact.f_type {
                    *pos = see_point;
                }
            } else {
                args.working_memory.add_working_memory_fact(
                    WMProperty::AIStimuli(AIStimuli::Character(character_id.unwrap(), see_point)),
                    detection_strength,
                    // Store a stimuli for two updates – don't keep it if character isn't actually visible
                    self.update_every * 3.0 + delta,
                );
                current_stimulation = detection_strength;
            }

            // new target spotted
            if current_stimulation >= 1.0 && previous_stimulation < 1.0 {
                let fact_query =
                    FactQuery::with_check(
                        FactQueryCheck::Match(WMProperty::Knowledge(Knowledge::Character(character_id.unwrap(), None))));
                if let Some(fact) = args.working_memory.find_fact_mut(fact_query) {
                    // update
                    let WMProperty::Knowledge(Knowledge::Character(_i, pos)) = &mut fact.f_type else { return false };
                    *pos = see_point;
                    fact.confidence = distance_to_target;
                    fact.update_time = SystemTime::now();
                } else {
                    // new character spotted!!
                    args.working_memory.add_working_memory_fact(
                        WMProperty::Knowledge(Knowledge::Character(character_id.unwrap(), see_point)),
                        distance_to_target,
                        240.0
                    );
                    // add desire to be surprised upon spotting new enemy
                    args.working_memory.add_working_memory_fact(
                        WMProperty::Desire(Surprise),
                        1.0,
                        15.0
                    );
                }
                // force retargeting
                args.blackboard.invalidate_target = true;
                args.blackboard.valid_targets = args.blackboard.valid_targets.union(TargetMask::VisibleCharacter);
            } else if current_stimulation > 1.0 {
                // update character knowledge with latest known position
                let fact_query =
                    FactQuery::with_check(
                        FactQueryCheck::Match(WMProperty::Knowledge(Knowledge::Character(character_id.unwrap(), None))));
                let Some(fact) = args.working_memory.find_fact_mut(fact_query) else {return false};
                fact.confidence = distance_to_target;
                fact.update_time = SystemTime::now();
                let WMProperty::Knowledge(Knowledge::Character(_i, pos)) = &mut fact.f_type else { return false };
                *pos = see_point;
            }
        }
        false
    }
}
