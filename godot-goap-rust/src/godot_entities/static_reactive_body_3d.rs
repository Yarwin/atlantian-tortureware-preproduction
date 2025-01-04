use crate::act_react::act_react_resource::ActReactResource;
use godot::prelude::*;

#[derive(GodotClass, Debug)]
#[class(init, base=StaticBody3D)]
pub struct StaticReactiveBody3D {
    #[export]
    pub act_react: Option<Gd<ActReactResource>>,
}
