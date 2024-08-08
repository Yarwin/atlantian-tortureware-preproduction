use godot::classes::ShapeCast3D;
use godot::prelude::*;
use crate::act_react::act_react_resource::ActReactResource;

#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct PlayerInterfaceController {
    #[init(node = "Head/Camera3D/InterfaceShapeCast")]
    pub interface_shape_cast: OnReady<Gd<ShapeCast3D>>,
    pub interface_act_react: Option<Gd<ActReactResource>>,
    base: Base<Node>
}
