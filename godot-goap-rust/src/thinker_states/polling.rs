use godot::builtin::{Dictionary, Rid, Transform3D};
use godot::obj::{Gd, NewGd};
use godot::classes::{PhysicsServer3D, PhysicsShapeQueryParameters3D};
use crate::ai_nodes::godot_ai_node::{AINodeType, GodotAINode};
use crate::godot_api::godot_thinker::GodotThinker;

/// a struct that lazily polls the world around an agent
#[derive(Debug)]
pub struct PollingResult {
    space_rid: Rid,
    shape_ainodes_rid: Rid,
    shape_vision_rid: Rid,
    transform: Transform3D,
    ainodes: Option<Vec<(u32, AINodeType)>>,
    visible_targets: Option<Vec<VisibleTarget>>,
}

#[derive(Debug)]
pub struct VisibleTarget {
    pub area_rid: Rid,
    pub area_transform: Transform3D,
    pub shape_height: f32,
    pub shape_radius: f32,
}

impl PollingResult {
    const AINODES_MASK: u32 = 2;
    const VISION_MASK: u32 = 4;

    pub fn from_godot_thinker(thinker: &Gd<GodotThinker>) -> Self {
        Self {
            space_rid: PhysicsServer3D::singleton()
                .body_get_space(thinker.bind().character_body.as_ref().unwrap().get_rid()),
            shape_ainodes_rid: thinker
                .bind()
                .ainodes_detection_shape
                .as_ref()
                .unwrap()
                .get_rid(),
            shape_vision_rid: thinker
                .bind()
                .vision_detection_shape
                .as_ref()
                .unwrap()
                .get_rid(),
            transform: thinker
                .bind()
                .character_body
                .as_ref()
                .unwrap()
                .get_global_transform(),
            ainodes: None,
            visible_targets: None,
        }
    }

    fn poll<T>(
        &mut self,
        collision_mask: u32,
        shape_res_rid: Rid,
        collide_with_bodies: bool,
        collide_with_arenas: bool,
        mapping: fn(Dictionary) -> Option<T>,
    ) -> Vec<T> {
        let mut space = PhysicsServer3D::singleton()
            .space_get_direct_state(self.space_rid)
            .expect("no space?!");
        let mut query_parameters = PhysicsShapeQueryParameters3D::new_gd();
        query_parameters.set_transform(self.transform);
        query_parameters.set_shape_rid(shape_res_rid);
        query_parameters.set_collide_with_bodies(collide_with_bodies);
        query_parameters.set_collide_with_areas(collide_with_arenas);
        query_parameters.set_collision_mask(collision_mask);
        space
            .intersect_shape(&query_parameters)
            .iter_shared()
            .filter_map(mapping)
            .collect()
    }

    /// returns colliding ainodes
    pub fn get_ainodes(&mut self) -> Option<&Vec<(u32, AINodeType)>> {
        // check for cached result
        if self.ainodes.is_none() {
            self.ainodes = Some(self.poll(
                PollingResult::AINODES_MASK,
                self.shape_ainodes_rid,
                false,
                true,
                |d| {
                    let mut x: Option<(u32, AINodeType)> = None;
                    if let Some(ainode) = d.get("collider").map(|c| c.to::<Gd<GodotAINode>>()) {
                        x = Some((ainode.bind().ainode_id, ainode.bind().node_type));
                    }
                    x
                },
            ));
        }
        // return None if cached result is empty
        let is_empty = self.ainodes.as_ref().map(|v| v.is_empty()).unwrap_or(true);
        if is_empty {
            return None;
        }
        return self.ainodes.as_ref();
    }

    /// returns visible areas
    pub fn get_visible(&mut self) -> Option<&Vec<VisibleTarget>> {
        // check for cached result
        if self.visible_targets.is_none() {
            self.visible_targets = Some(self.poll(
                PollingResult::VISION_MASK,
                self.shape_vision_rid,
                false,
                true,
                |d| {
                    let collider_rid = d.get("rid").map(|v| v.to::<Rid>())?;
                    let shape_index = d.get("shape").map(|v| v.to::<i32>())?;
                    let shape_rid =
                        PhysicsServer3D::singleton().area_get_shape(collider_rid, shape_index);
                    let shape_data = PhysicsServer3D::singleton()
                        .shape_get_data(shape_rid)
                        .to::<Dictionary>();
                    let radius = shape_data.get("radius").map(|v| v.to::<f32>())?;
                    let height = shape_data.get("height").map(|v| v.to::<f32>())?;
                    let area_global_transform =
                        PhysicsServer3D::singleton().area_get_transform(collider_rid);
                    let visible_target = VisibleTarget {
                        area_rid: collider_rid,
                        area_transform: area_global_transform,
                        shape_height: radius,
                        shape_radius: height,
                    };
                    Some(visible_target)
                },
            ));
        }
        // return None if cached result is empty
        let is_empty = self
            .visible_targets
            .as_ref()
            .map(|v| v.is_empty())
            .unwrap_or(true);
        if is_empty {
            return None;
        }
        self.visible_targets.as_ref()
    }
}
