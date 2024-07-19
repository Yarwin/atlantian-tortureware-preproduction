use crate::ai::blackboard::{Blackboard, SpeedMod};
use crate::godot_api::godot_thinker::GodotThinker;
use godot::classes::{
    CharacterBody3D, MeshInstance3D, PhysicsRayQueryParameters3D, PhysicsServer3D,
};
use godot::prelude::*;
use std::mem;
use std::mem::MaybeUninit;

const AVOIDANCE_COLLISION_MASK: u32 = 8;
const UP_OFFSET: Vector3 = Vector3::new(0.0, 0.1, 0.0);

const ANGLES: [f32; 8] = [
    0.0,                         // 0
    std::f32::consts::FRAC_PI_6, // 30
    std::f32::consts::FRAC_PI_3, // 60
    std::f32::consts::FRAC_PI_2, // 90
    std::f32::consts::PI,        //180
    4.712_389,                   // 270
    5.235_987_7,                 // 300
    5.759_586_3,                 // 330
];

type SteeringTable = [f32; 8];

#[allow(dead_code)]
pub struct NavigationArguments<'a> {
    pub base: Gd<GodotThinker>,
    pub blackboard: &'a mut Blackboard,
    pub navigation_data: &'a mut Navigator,
    pub delta: f64,
}

#[derive(Default, Debug)]
pub struct Navigator {
    pub danger_table: SteeringTable,
}

#[derive(Debug)]
pub enum RotationTarget {
    Position(Vector3),
    Character(InstanceId)
}

fn combine_tables_and_get_velocity(
    interest_table: &SteeringTable,
    danger_table: &SteeringTable,
    base_vec: Vector3,
) -> Vector3 {
    let mut result = Vector3::new(0.0, 0.0, 0.0);
    for (i, angle) in ANGLES.iter().enumerate() {
        if interest_table[i] <= 0.0 {
            continue;
        }
        // don't apply the repulsion if danger is less than 1
        let modifier: f32 = if danger_table[i] > 1.0 {
            interest_table[i] - danger_table[i]
        } else {
            (interest_table[i] - danger_table[i]).max(0.0)
        };
        result += base_vec.rotated(Vector3::UP, *angle).normalized() * modifier;
    }
    result
}

fn seek(target: Vector3, base_vec: Vector3) -> SteeringTable {
    unsafe {
        let mut arr: [MaybeUninit<f32>; 8] = MaybeUninit::uninit().assume_init();
        for (i, item) in &mut arr[..].iter_mut().enumerate() {
            let weight = base_vec
                .rotated(Vector3::UP, ANGLES[i])
                .normalized()
                .dot(target)
                .max(0.0);
            item.write(weight);
        }
        mem::transmute::<_, SteeringTable>(arr)
    }
}

fn avoid(
    interest_table: &SteeringTable,
    old_danger_table: &SteeringTable,
    avoidance_radius: f32,
    caster: &mut Gd<CharacterBody3D>,
    base_vec: Vector3,
    agent_radius: f32,
) -> SteeringTable {
    let mut danger_table: SteeringTable = Default::default();
    let caster_rid = caster.get_rid();
    let space_rid = PhysicsServer3D::singleton().body_get_space(caster_rid);
    if matches!(caster_rid, Rid::Invalid) {
        return *old_danger_table;
    };
    let Some(mut direct_space) = PhysicsServer3D::singleton().space_get_direct_state(space_rid)
    else {
        return *old_danger_table;
    };

    let caster_pos = caster.get_global_position();
    let mut ray_params = PhysicsRayQueryParameters3D::new_gd();
    ray_params.set_collision_mask(AVOIDANCE_COLLISION_MASK);
    ray_params.set_from(caster_pos + UP_OFFSET);
    ray_params.set_collide_with_bodies(true);
    ray_params.set_exclude(array![caster_rid]);

    for (i, angle) in ANGLES.iter().enumerate() {
        let mut debug_node = caster.get_node_as::<MeshInstance3D>(format!("Debug/{}", i + 1));
        debug_node.set_global_position(
            caster_pos + (base_vec * avoidance_radius).rotated(Vector3::UP, *angle) + UP_OFFSET,
        );

        // bail if agent has no interest to go into such position
        if interest_table[i] < 0.0 {
            continue;
        }

        ray_params.set_to(
            caster_pos
                + (base_vec * avoidance_radius)
                    .rotated(Vector3::UP, *angle)
                    .normalized()
                + UP_OFFSET,
        );
        let intersection_result = direct_space.intersect_ray(ray_params.clone());
        if let Some(colpos) = intersection_result
            .get("position")
            .map(|v| v.to::<Vector3>())
        {
            let distance = caster_pos.distance_to(colpos - UP_OFFSET);
            if distance > agent_radius {
                let current_danger =
                    (1.0 - (distance / (avoidance_radius - agent_radius))).max(0.0);
                // interpolate the result over few frames to prevent "cycling"
                danger_table[i] = old_danger_table[i].lerp(current_danger, 0.33);
                continue;
            }
            // apply repulsion force
            danger_table[i] = 1.5;
        }
    }
    danger_table
}

pub fn rotate(
    character: &mut Gd<CharacterBody3D>,
    rotation_target: &RotationTarget,
    rotation_speed: f32,
    delta: f64,
) {
    let target = match rotation_target {
        RotationTarget::Position(p) => {*p}
        RotationTarget::Character(character_id) => {
            let char: Gd<Node3D> = Gd::from_instance_id(*character_id);
            char.get_global_position()
        }
    };
    let ground_offset = (character.get_global_transform().origin * Vector3::UP).y;
    let lateral_plane = Plane::new(Vector3::UP, ground_offset);
    let look_at_transform =
        character
            .get_transform()
            .looking_at(lateral_plane.project(target), Vector3::UP, true);
    let new_transform = character
        .get_transform()
        .interpolate_with(look_at_transform, rotation_speed * delta as f32);
    character.set_transform(new_transform);
}

pub fn navigate(mut navigation_arguments: NavigationArguments, delta: f64) {
    let bind = navigation_arguments.base.bind_mut();
    let Some(mut character) = bind.character_body.clone() else {
        return;
    };

    // rotate character
    if let Some(rotation_target) = navigation_arguments.blackboard.rotation_target.as_ref() {
        let rotation_speed = match navigation_arguments.blackboard.rotation_speed {
            SpeedMod::Slow => { bind.rotation_speed_walk }
            SpeedMod::Normal => {bind.rotation_speed_normal}
            SpeedMod::Fast => {bind.rotation_speed_fast}
        };
        rotate(&mut character, rotation_target, rotation_speed, delta);
    }

    let mut desired_velocity = navigation_arguments
        .blackboard
        .desired_velocity
        .take()
        .unwrap_or(Vector3::ZERO);
    // calculate the Context steering – see http://www.gameaipro.com/GameAIPro2/GameAIPro2_Chapter18_Context_Steering_Behavior-Driven_Steering_at_the_Macro_Scale.pdf
    if !desired_velocity.is_zero_approx() {
        let old_danger_table = &navigation_arguments.navigation_data.danger_table;
        let dir = desired_velocity.normalized();
        let forward_vec = character.get_global_transform().basis.col_c();
        let interest_table = seek(dir, forward_vec);
        navigation_arguments.navigation_data.danger_table = avoid(
            &interest_table,
            old_danger_table,
            bind.avoidance_detection_radius,
            &mut character,
            forward_vec,
            bind.agent_radius,
        );
        let avoidance = combine_tables_and_get_velocity(
            &interest_table,
            &navigation_arguments.navigation_data.danger_table,
            forward_vec,
        );
        let mut debug_node = character.get_node_as::<MeshInstance3D>("Debug/DebugDir");
        debug_node.set_global_position(
            character.get_global_position() + desired_velocity.length() * avoidance.normalized(),
        );
        desired_velocity = desired_velocity.length() * avoidance.normalized();
        if desired_velocity.length() < 0.3 {
            desired_velocity *= 5.0;
        }
    }
    character.set_velocity(desired_velocity);
    character.move_and_slide();
    navigation_arguments.blackboard.thinker_position = character.get_global_position();
}
