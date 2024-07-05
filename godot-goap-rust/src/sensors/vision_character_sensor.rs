use godot::prelude::*;
use godot::classes::{PhysicsRayQueryParameters3D, PhysicsServer3D};
use serde::{Deserialize, Serialize};
use crate::ai::working_memory::{FactQuery, FactQueryCheck, WorkingMemoryFactType};
use crate::godot_api::godot_visible_area_3d::GodotVisibilityArea3D;
use crate::sensors::sensor_types::{SensorArguments, SensorPolling};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionCharacterSensor {
    update_every: f64,
    last_update_delta: f64
}


impl SensorPolling for VisionCharacterSensor {
    fn process(&mut self, delta: f64, args: &mut SensorArguments) -> bool {
        self.last_update_delta += delta;
        if self.last_update_delta < self.update_every {
            return false;
        }
        self.last_update_delta = 0.0;

        if args.polls.get_visible().is_none() {return false}
        for target in args.polls.get_visible().unwrap() {
            // bail if target outside vision cone
            if args.thinker_forward_axis.dot(args.head_position.direction_to(target.area_transform.origin)) < 0.12 {
                continue
            }

            let mut is_target_visible: bool = false;
            let target_height = target.area_transform.basis.col_b() * target.shape_height;
            // visibility check

            // todo –  project ray targets on a clipped plane that uses area direction to the head as its normal
            // we assume that target's visible area is always a cylinder

            let raycast_directions = [
                target.area_transform.origin + target_height / 2.0, // midpoint
                target.area_transform.origin + target_height, // top
                target.area_transform.origin, // bottom
                target.area_transform.origin + target_height + target.area_transform.basis.col_a() * target.shape_radius, // left
                target.area_transform.origin + target_height - target.area_transform.basis.col_a() * target.shape_radius, // right
            ];
            let mut ray_params = PhysicsRayQueryParameters3D::new_gd();
            ray_params.set_collision_mask(4);
            ray_params.set_from(args.head_position);
            ray_params.set_collide_with_areas(true);
            ray_params.set_collide_with_bodies(true);
            ray_params.set_exclude(array![args.character_rid]);
            let space_rid = PhysicsServer3D::singleton().body_get_space(args.character_rid);
            let Some(mut direct_space) = PhysicsServer3D::singleton().space_get_direct_state(space_rid) else {return  false };
            let mut distance_to_target: f32 = 0.0;
            let mut character_id: Option<InstanceId> = None;

            for raycast_target in raycast_directions {
                ray_params.set_to(raycast_target);
                let intersection_result = direct_space.intersect_ray(ray_params.clone());
                is_target_visible = intersection_result.get("rid").map(|r| r.to::<Rid>() == target.area_rid).unwrap_or(false);

                // bail if we found a target
                if is_target_visible {
                    distance_to_target = intersection_result.get("position").map(|v| v.to::<Vector3>().distance_to(args.head_position)).unwrap();
                    character_id = intersection_result.get("collider")
                        .map(|v|{
                            let area = v.to::<Gd<GodotVisibilityArea3D>>();
                            let instance_id = area.bind().owner.as_ref().unwrap().instance_id();
                            instance_id
                        });
                    break
                }
            }

            // bail if target is not visible
            if !is_target_visible {
                return false
            }
            // get detection strength
            let detection_strength = ((11.0 - distance_to_target) / 11.0).min(1.0);
            let mut previous_stimulation: f32 = 0.0;
            let current_stimulation: f32;
            // check for fact
            let fact_query = FactQuery::with_check(FactQueryCheck::Character(character_id.unwrap()));
            // update fact
            let update_time = args.working_memory.elapsed_time;
            if let Some(fact) = args.working_memory.find_fact_mut(fact_query) {
                fact.update_time = update_time;
                previous_stimulation = fact.confidence;
                fact.confidence = (fact.confidence + detection_strength).min(1.1);
                current_stimulation = fact.confidence;
            } else {
                // write new fact
                args.working_memory.add_working_memory_fact(WorkingMemoryFactType::Character(character_id.unwrap()), detection_strength, 120.0);
                current_stimulation = detection_strength;
            }
            if current_stimulation >= 1.0 && previous_stimulation < 1.0 {
                // let Ok(mut bb) = args.blackboard.lock() else {panic!("mutex failed!")};

            }
        }

        false
    }
}