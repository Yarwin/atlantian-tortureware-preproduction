use godot::classes::{IArea3D, Area3D, Engine};
use godot::prelude::*;
use crate::act_react::act_react_executor::ActReactExecutor;
use crate::act_react::act_react_resource::ActReactResource;
use bitflags::bitflags;
use godot::global::PropertyHint;
use godot::register::property::PropertyHintInfo;

const FROB_PHYSICS_MASK: u32 = 128;


bitflags! {
    #[derive(Default)]
    pub struct PropagationMode: u32 {
        const None = 0;
        const Contact = 1;
        const Continuous = 1 << 1;
        }
}

impl Var for PropagationMode {
    fn get_property(&self) -> Self::Via {
        self.bits()
    }

    fn set_property(&mut self, value: Self::Via) {
        *self = PropagationMode::from_bits(value).unwrap();
    }
}

impl Export for PropagationMode {
    fn default_export_info() -> PropertyHintInfo {
        let mut hint = PropertyHintInfo::with_hint_none("TYPE_INT");
        hint.hint = PropertyHint::FLAGS;
        hint
    }
}


impl GodotConvert for PropagationMode { type Via = u32; }

impl ToGodot for PropagationMode {
    fn to_godot(&self) -> Self::Via {
        self.bits()
    }

    fn into_godot(self) -> Self::Via {
        self.bits()
    }

    fn to_variant(&self) -> Variant {
        self.bits().to_variant()
    }
}

impl FromGodot for PropagationMode {
    fn try_from_godot(via: Self::Via) -> Result<Self, ConvertError> {
        match PropagationMode::from_bits(via) {
            None => {
                Err(ConvertError::default())
            }
            Some(val) => {Ok(val)}
        }
    }
}

#[derive(GodotClass)]
#[class(init, base=Area3D)]
pub struct ActReactArea3D {
    #[export(flags = (Contact = 1, Continuous = 2) )]
    propagation_mode: PropagationMode,
    #[export]
    pub target: Option<Gd<Node3D>>,
    #[export]
    pub act_react: Option<Gd<ActReactResource>>,
    base: Base<Area3D>
}

#[godot_api]
impl IArea3D for ActReactArea3D {

    fn physics_process(&mut self, _delta: f64) {
        if !self.propagation_mode.contains(PropagationMode::Continuous) {
            return;
        }
        let colliding_areas = self.base().get_overlapping_areas();
        for area in colliding_areas.iter_shared() {
            self.on_other_area_act(area.cast());
        }
    }

    fn ready(&mut self) {
        self.base_mut().set_monitoring(true);
        self.base_mut().set_collision_mask(FROB_PHYSICS_MASK);
        self.base_mut().set_collision_layer(FROB_PHYSICS_MASK);
        godot_print!("oopsie");
        if self.propagation_mode.contains(PropagationMode::Contact) {
            self.base_mut().set_monitorable(true);
            let callable = self.base().callable("on_other_area_act");
            self.base_mut().connect("area_entered".into(), callable);
        }
    }
}


#[godot_api]
impl ActReactArea3D {

    
    #[func]
    fn on_other_area_act(&self, actor: Gd<ActReactArea3D>) {
        let Some(react) = self.act_react.clone() else {return;};
        let Some(act) = actor.bind().act_react.clone() else {return;};
        let mut act_react_executor = Engine::singleton()
            .get_singleton("ActReactExecutor".into())
            .unwrap()
            .cast::<ActReactExecutor>();
        act_react_executor.bind_mut().react(act, react, dict!{});
    }

    #[func]
    pub fn react(&self, act: Gd<ActReactResource>) {
        let Some(react) = self.act_react.clone() else {return;};
        let mut act_react_executor = Engine::singleton()
            .get_singleton("ActReactExecutor".into())
            .unwrap()
            .cast::<ActReactExecutor>();
        act_react_executor.bind_mut().react(act, react, dict!{});
    }
}
