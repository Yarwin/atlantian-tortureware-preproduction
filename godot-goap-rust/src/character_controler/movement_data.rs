use godot::classes::KinematicCollision3D;
use godot::prelude::*;

#[derive(Debug)]
pub struct PlatformData {
    pub rid: Rid,
    pub velocity: Vector3,
}

#[derive(Default, Debug)]
pub struct MovementData {
    pub(crate) velocity: Vector3,
    pub(crate) initial_lateral_translation: Vector3,
    pub(crate) lateral_translation: Vector3,
    pub(crate) initial_vertical_translation: Vector3,
    pub(crate) vertical_translation: Vector3,
    pub(crate) ground_snap_translation: Vector3,
    pub(crate) initial_ground_snap_translation: Vector3,
    pub(crate) total_stepped_height: Option<Vector3>,
    pub(crate) grounded: bool,
    pub(crate) movement_collision: Option<Gd<KinematicCollision3D>>,
    pub(crate) ground_normal: Option<Vector3>,
    pub(crate) steep_slope_normals: Option<Vec<Vector3>>,
    pub(crate) platform_data: Option<PlatformData>,
    // pub(crate) lateral_collisions: Option<Vec<Gd<KinematicCollision3D>>>,
    // pub(crate) vertical_collisions: Option<Vec<Gd<KinematicCollision3D>>>,
    // pub(crate) snap_collisions: Option<Vec<Gd<KinematicCollision3D>>>,
}

impl MovementData {
    pub(crate) fn new(desired_motion: Vector3) -> Self {
        let initial_lateral_translation: Vector3 = Vector3::new(1.0, 0.0, 1.0) * desired_motion;
        let initial_vertical_translation: Vector3 = Vector3::UP * desired_motion;

        Self {
            initial_lateral_translation,
            lateral_translation: initial_lateral_translation,
            initial_vertical_translation,
            vertical_translation: initial_vertical_translation,
            ..Default::default()
        }
    }

    pub(crate) fn add_platform_collision(&mut self, col: &Gd<KinematicCollision3D>) {
        self.platform_data = Some(PlatformData {
            rid: col.get_collider_rid(),
            velocity: col.get_collider_velocity(),
        });
    }
}
