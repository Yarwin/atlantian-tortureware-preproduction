use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=Area3D, rename=VisibilityArea3D)]
pub struct GodotVisibilityArea3D {
    /// a character tied to this very area
    #[export]
    pub owner: Option<Gd<Node3D>>,
}
