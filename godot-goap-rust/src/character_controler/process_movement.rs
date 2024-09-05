
use godot::prelude::*;
use godot::classes::{ProjectSettings, CylinderShape3D, PhysicsTestMotionResult3D, KinematicCollision3D, PhysicsTestMotionParameters3D, PhysicsBody3D, CollisionShape3D, PhysicsServer3D};
use std::cmp::Ordering;
use std::f32::consts::FRAC_PI_2;
use godot::global::{cos, tan};
use crate::character_controler::movement_data::MovementData;

const GROUND_CAST_DISTANCE: f32 = 0.004;
const MAX_ITERATIONS_COUNT: u32 = 4;
/// 0,51m or <~17QU
const MAX_STEP_HEIGHT: f32 = 0.51;
const STEP_FORWARD_MULTIPLIER: f32 = 0.05;
/// PI / 4 => 45*
const SLOPE_LIMIT: f32 = std::f32::consts::FRAC_PI_3;
const SNAP_TO_GROUND_DISTANCE: f32 = 0.2;

#[derive(PartialEq, Debug)]
enum MovementType {
    Lateral,
    Vertical,
    Snap
}

#[derive(Debug)]
pub struct MovementParameters {
    pub(crate) direction: Vector3,
    pub(crate) jump_force: f32,
    pub(crate) body: Gd<PhysicsBody3D>,
    pub(crate) collision_shape: Gd<CollisionShape3D>,
    pub(crate) excluded_bodies: Option<Array<Rid>>,
    pub(crate) deceleration: f32,
    pub(crate) speed: f32,
    pub(crate) acceleration: f32,
    pub(crate) gravity_scale: f32,
    pub(crate) current_platform_translation: Vector3
}


pub fn process_movement(delta: f32, mut args: MovementParameters, previous_movement: Option<MovementData>) -> Option<MovementData> {
    let previous_horizontal_speed = previous_movement.as_ref().map(|pm| (pm.velocity * Vector3::new(1.0, 0.0, 1.0)).length()).unwrap_or(0.0);
    let current_speed: f32;
    let mut desired_motion: Vector3;
    if args.direction.is_zero_approx() && previous_movement.is_some() {
        current_speed = previous_horizontal_speed.lerp(0.0, args.deceleration);
        let previous_velocity = previous_movement.as_ref().unwrap().velocity;
        let previous_direction = if previous_velocity.is_zero_approx() {
            Vector3::ZERO
        } else {
            previous_velocity.normalized() * Vector3::new(1.0, 0.0, 1.0)
        };
        desired_motion = previous_direction * current_speed;
    }  else {
        current_speed = previous_horizontal_speed.lerp(args.speed, args.acceleration);
        desired_motion = args.direction * current_speed;
    }
    let mut platform_velocity = Vector3::ZERO;
    if let Some(previous_mov) = previous_movement.as_ref() {
        if !previous_mov.grounded {
            desired_motion += Vector3::DOWN * (previous_mov.velocity.y + ProjectSettings::singleton().get_setting("physics/3d/default_gravity".into()).to::<f32>() * args.gravity_scale * delta);
        } else if previous_mov.grounded && !args.jump_force.is_zero_approx() {
            desired_motion += Vector3::UP * args.jump_force;
        } else if previous_mov.grounded && previous_mov.velocity.y < 0. {
            desired_motion.y = 0.0;
        }
        if let Some(platform) = previous_mov.platform_data.as_ref() {
            if let Some(body_space) = PhysicsServer3D::singleton().body_get_direct_state(platform.rid) {
                let local_position = args.body.get_global_position() - body_space.get_transform().origin;
                platform_velocity += body_space.get_velocity_at_local_position(local_position);
            }
        }
    }
    args.current_platform_translation += platform_velocity * delta;
    let mut current_movement_data = execute_movement(desired_motion, args, delta);
    if let Some(mov_data) = current_movement_data.as_mut() {
        if mov_data.platform_data.is_none() {
            mov_data.velocity += platform_velocity;
        }
    }
    current_movement_data
}


fn execute_movement(desired_motion: Vector3, mut args: MovementParameters, delta: f32) -> Option<MovementData> {
    let start_position = args.body.get_transform().origin;
    let mut movement_data = MovementData::new(desired_motion * delta);
    // align to platform
    if !args.current_platform_translation.length_squared().is_zero_approx() {
        // don't add upward motion velocity (player will be lifted on its own by colliding body instead)
        if args.current_platform_translation.y > 0. {
            args.current_platform_translation.y = 0.;
        }
        args.body.move_and_collide(args.current_platform_translation);
    }

    if movement_data.vertical_translation.length() <= f32::EPSILON {
        if let Some(ground_collision) = args.body.move_and_collide_ex(Vector3::DOWN * GROUND_CAST_DISTANCE).test_only(true).done() {
            if ground_collision.get_normal().angle_to(Vector3::UP) < SLOPE_LIMIT {
                movement_data.grounded = true;
                movement_data.ground_normal = Some(ground_collision.get_normal());
            }
        }
    }
    let mut lateral_iterations: u32 = 0;
    while movement_data.lateral_translation.length_squared() > 0.0 && lateral_iterations < MAX_ITERATIONS_COUNT {
        move_iteration(MovementType::Lateral, &mut args, &mut movement_data);
        lateral_iterations += 1;
    }

    let mut vertical_iterations: u32 = 0;
    while movement_data.vertical_translation.length_squared() > 0.0 && vertical_iterations < MAX_ITERATIONS_COUNT {
        move_iteration(MovementType::Vertical, &mut args, &mut movement_data);
        vertical_iterations += 1;
    }
    movement_data.velocity = {
        (start_position - args.body.get_position() + movement_data.total_stepped_height.unwrap_or(Vector3::ZERO)) / delta
    };

    if movement_data.grounded {
        let mut snap_iterations: u32 = 0;
        movement_data.ground_snap_translation = Vector3::DOWN * SNAP_TO_GROUND_DISTANCE;
        movement_data.initial_ground_snap_translation = Vector3::DOWN;
        while movement_data.ground_snap_translation.length_squared() > 0.0 && snap_iterations < MAX_ITERATIONS_COUNT {
            move_iteration(MovementType::Snap, &mut args, &mut movement_data);
            snap_iterations += 1;
        }
    }

    Some(movement_data)
}


fn move_iteration(movement_type: MovementType, args: &mut MovementParameters, movement_data: &mut MovementData) {
    if movement_type == MovementType::Lateral && !move_and_step(movement_data, args) && movement_data.movement_collision.is_none() {
        args.body.move_and_collide(movement_data.lateral_translation);
        movement_data.lateral_translation = Vector3::ZERO;
        return;
    }
    let translation: &mut Vector3 = match movement_type {
        MovementType::Lateral => &mut movement_data.lateral_translation,
        MovementType::Vertical => &mut movement_data.vertical_translation,
        MovementType::Snap => &mut movement_data.ground_snap_translation,
    };
    let initial_translation: &Vector3 = match movement_type {
        MovementType::Lateral => &mut movement_data.initial_lateral_translation,
        MovementType::Vertical => &mut movement_data.initial_vertical_translation,
        MovementType::Snap => &mut movement_data.initial_ground_snap_translation,
    };

    movement_data.movement_collision = args.body.move_and_collide_ex(*translation).test_only(false).done();
    if movement_data.movement_collision.is_none() {
        *translation = Vector3::ZERO;
        return;
    } else {
        *translation -= movement_data.movement_collision.as_ref().unwrap().get_travel();
    }
    if movement_data.movement_collision.as_ref().map(|c| c.get_normal().angle_to(Vector3::UP) < SLOPE_LIMIT).unwrap_or(false) {
        movement_data.grounded = true;
        movement_data.ground_normal = movement_data.movement_collision.as_ref().map(|c| c.get_normal());
    }

    let min_block_angle = match movement_type {
        MovementType::Lateral => SLOPE_LIMIT,
        MovementType::Vertical | MovementType::Snap => 0.0
    };

    let max_block_angle = match movement_type {
        MovementType::Lateral => {
            if movement_data.grounded {
                2.0 * std::f32::consts::PI
            } else {
                std::f32::consts::FRAC_PI_2
            }
        }
        MovementType::Vertical | MovementType::Snap => {
            SLOPE_LIMIT
        }
    };

    if let Some(collision) = movement_data.movement_collision.take() {
        let surface_angle = collision.get_normal().angle_to(Vector3::UP);
        let mut projection_normal = collision.get_normal();
        // If collision happens on the "side" of the cylinder, treat it as a vertical
        // wall in all the cases (we use the tangent of the cylinder)
        let collision_point = collision.get_position();

        let shape_height = args.collision_shape.get_shape().unwrap().cast::<CylinderShape3D>().get_height();
        let distance_to_bottom = (args.collision_shape.get_global_position() - args.body.get_global_basis().col_b() * shape_height * 0.5).y;
        if movement_type == MovementType::Lateral && collision_point.y > distance_to_bottom + 0.005 {
            projection_normal = args.collision_shape.get_global_position() - collision_point;
            projection_normal.y = 0.0;
            projection_normal = projection_normal.normalized();
        }
        else if surface_angle >= min_block_angle && surface_angle <= max_block_angle {
            if movement_type == MovementType::Vertical || movement_type == MovementType::Snap {
                if movement_type == MovementType::Snap {
                    movement_data.add_platform_collision(&collision);
                }
                // If vertical is blocked, you're on solid ground - just stop moving
                movement_data.vertical_translation = Vector3::ZERO;
                return;
            }
            projection_normal = (projection_normal * Vector3::new(1.0, 0.0, 1.0)).normalized();
            if movement_data.grounded && surface_angle < std::f32::consts::FRAC_PI_2 {
                if movement_data.steep_slope_normals.is_none()
                {
                    movement_data.steep_slope_normals = Some(vec![collision.get_normal()]);
                }
                else {
                    let is_already_touched = movement_data.steep_slope_normals.as_ref().map(
                        |v| v.iter().any(
                            |n| n.distance_squared_to(collision.get_normal()) < f32::EPSILON
                        )
                    ).unwrap_or(false);
                    if is_already_touched {
                        movement_data.steep_slope_normals.as_mut().unwrap().push(collision.get_normal());
                    }
                }
                let seam = collision.get_normal().cross(movement_data.ground_normal.unwrap_or(Vector3::ZERO));
                let temp_projection_plane = Plane::from_points(Vector3::ZERO, seam, seam + Vector3::UP);
                projection_normal = temp_projection_plane.normal;
            }
        }
        else if movement_type == MovementType::Lateral && (surface_angle - std::f32::consts::FRAC_PI_2) < 0.0 {
            if let Some(slope_normal) = relative_slope_normal(collision.get_normal(), translation) {
                projection_normal = slope_normal;
            }
        }
        let projection_plane = Plane::from_normal_at_origin(projection_normal);
        let continued_translation = projection_plane.project(collision.get_remainder());
        let initial_influenced_translation = projection_plane.project(*initial_translation);
        let mut next_translation: Vector3;
        if initial_influenced_translation.dot(continued_translation) >= 0.0 {
            next_translation = continued_translation;
        } else {
            next_translation = initial_influenced_translation.normalized() * continued_translation.length();
        }
        if !next_translation.is_zero_approx() &&
            !translation.is_zero_approx() &&
            next_translation.normalized().distance_to(translation.normalized()).is_zero_approx()
        {
            next_translation += collision.get_normal() * 0.001;
        }
        *translation = next_translation;
    }
}

fn call_body_test_motion(varargs: &[Variant]) -> bool {
    PhysicsServer3D::singleton().call("body_test_motion".into(),
                                      varargs).to::<bool>()
}


/// check if given collision is steppable and step over it
fn step(forward_collision: &Gd<KinematicCollision3D>, args: &mut MovementParameters, movement_data: &mut MovementData) {
    // let player_body = &args.body;
    let body_rid = args.body.get_rid();
    let mut trans_step = args.body.get_global_transform();
    let mut motion_parameters = PhysicsTestMotionParameters3D::new_gd();
    let mut motion_result = PhysicsTestMotionResult3D::new_gd();

    if let Some(excluded) = args.excluded_bodies.take() {
        motion_parameters.set_exclude_bodies(excluded)
    }
    let step_height = Vector3::UP * MAX_STEP_HEIGHT;
    motion_parameters.set_from(trans_step);
    // motion_parameters.set_motion(Vector3::UP * MAX_STEP_HEIGHT);
    motion_parameters.set_motion(step_height);
    let is_colliding_up = call_body_test_motion(&[body_rid.to_variant(), motion_parameters.to_variant(), motion_result.to_variant()]);
    let distance_to_ceiling = if is_colliding_up {
        Vector3::UP * motion_result.get_travel()
    } else {
        step_height
    };
    trans_step.origin += distance_to_ceiling;
    motion_result = PhysicsTestMotionResult3D::new_gd();
    motion_parameters.set_from(trans_step);
    let mut distance_to_next_wall: Vector3 = movement_data.lateral_translation - forward_collision.get_travel();
    motion_parameters.set_motion(distance_to_next_wall);
    let is_colliding_forward = call_body_test_motion(&[body_rid.to_variant(), motion_parameters.to_variant(), motion_result.to_variant()]);
    if is_colliding_forward {
        distance_to_next_wall = motion_result.get_travel();
        if distance_to_next_wall.length_squared().is_zero_approx() {
            return;
        }
    }

    let mut distance_to_floor: Option<f32> = None;
    motion_result = PhysicsTestMotionResult3D::new_gd();
    trans_step.origin += distance_to_next_wall;
    motion_parameters.set_from(trans_step);
    motion_parameters.set_motion(Vector3::DOWN * distance_to_ceiling);
    let is_colliding_with_floor = call_body_test_motion(&[body_rid.to_variant(), motion_parameters.to_variant(), motion_result.to_variant()]);
    if !is_colliding_with_floor { return; }
    for col_index in 0..motion_result.get_collision_count() {
        let col_point = motion_result.get_collision_point_ex().collision_index(col_index).done() * Vector3::UP;
        let body_position_y = Vector3::UP * args.body.get_global_position();
        // stupid edge case – make sure that step floor is at higher point than body
        if (col_point.y - body_position_y.y).sign() != 1. {
            continue;
        }
        let distance = col_point.distance_to(body_position_y);
        // check if given step floor is walkable
        if motion_result.get_collision_normal_ex().collision_index(col_index).done().angle_to(Vector3::UP) > SLOPE_LIMIT {
            godot_print!("given step floor is not walkable");
            continue;
        }
        distance_to_floor = Some(distance);
        break
    }
    if let Some(distance) = distance_to_floor.take() {
        args.body.move_and_collide(Vector3::UP * distance);
        movement_data.total_stepped_height = Some(Vector3::UP * distance);
        // edge case - add some extra movement to deal with stepping from slopes
        // adding small margin is easier&faster than checking if step happens on slope and works fairly well
        let step_translation = distance_to_next_wall + forward_collision.get_travel() * (1. + STEP_FORWARD_MULTIPLIER);
        args.body.move_and_collide(step_translation);
        movement_data.lateral_translation -= step_translation;
    }
}



fn move_and_step(movement_data: &mut MovementData, args: &mut MovementParameters) -> bool {
    let mut forward_test = args.body.move_and_collide_ex(movement_data.lateral_translation).test_only(true).recovery_as_collision(true).done();

    if let Some(forward_collision) = forward_test.take() {
        // don't step on slopes
        match forward_collision.get_normal().angle_to(Vector3::UP).partial_cmp(&SLOPE_LIMIT) {
            None | Some(Ordering::Less) | Some(Ordering::Equal) => {
                // non-step movement.
                movement_data.movement_collision = Some(forward_collision);
            }
            Some(Ordering::Greater) => {
                step(&forward_collision, args, movement_data);
                return true;
            }
        }
    }
    false
}

fn relative_slope_normal(slope_normal: Vector3, translation: &Vector3) -> Option<Vector3> {
    // FUCK GODOT PHYSICS
    if slope_normal.x.is_nan() || slope_normal.y.is_nan() || slope_normal.z.is_nan() {
        return None
    }
    let slope_normal_lateral = slope_normal * Vector3::new(1.0, 0.0, 1.0);
    let angle_straight = slope_normal_lateral.angle_to(-*translation);
    let angle_up = slope_normal.angle_to(Vector3::UP);
    if angle_up > FRAC_PI_2 {
        godot_print!("Trying to calculate relative slope normal for a ceiling");
        return None
    }
    let complementary_angle_to_up = FRAC_PI_2 - angle_up;
    let straight_length = cos(angle_straight as f64) as f32 * translation.length();
    let height = straight_length / tan(complementary_angle_to_up as f64) as f32;
    // actual desired movement:
    let vector_up_slope = Vector3::new(translation.x, height, translation.z);
    let rotation_axis = vector_up_slope.cross(Vector3::UP).normalized();
    if rotation_axis.is_zero_approx() {
        return None;
    }
    if !rotation_axis.is_normalized() {
        godot_print!("godot physics pls: {rotation_axis}");
        return None;
    }
    let emulated_normal = vector_up_slope.rotated(rotation_axis, FRAC_PI_2);
    Some(emulated_normal.normalized())
}
