use crate::act_react::act_react_executor::ActReactExecutor;
use crate::act_react::act_react_resource::ActReactResource;
use crate::godot_api::gamesys::GameSystem;
use bitflags::bitflags;
use godot::classes::{Area3D, IArea3D};
use godot::global::PropertyHint;
use godot::meta::PropertyHintInfo;
use godot::prelude::*;

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
    fn export_hint() -> PropertyHintInfo {
        PropertyHintInfo {
            hint: PropertyHint::FLAGS,
            hint_string: "TYPE_INT".into(),
        }
    }
}

impl GodotConvert for PropagationMode {
    type Via = u32;
}

impl ToGodot for PropagationMode {
    type ToVia<'v> = Self::Via;

    fn to_godot(&self) -> Self::Via {
        self.bits()
    }

    fn to_variant(&self) -> Variant {
        self.bits().to_variant()
    }
}

impl FromGodot for PropagationMode {
    fn try_from_godot(via: Self::Via) -> Result<Self, ConvertError> {
        match PropagationMode::from_bits(via) {
            None => Err(ConvertError::default()),
            Some(val) => Ok(val),
        }
    }
}

#[derive(GodotClass)]
#[class(init, base=Area3D)]
pub struct ActReactArea3D {
    #[export(flags = (Contact = 1, Continuous = 2) )]
    propagation_mode: PropagationMode,
    #[export]
    pub target: Option<Gd<Node>>,
    #[export]
    pub act_react: Option<Gd<ActReactResource>>,
    #[export]
    pub name_display: GString,
    base: Base<Area3D>,
}

impl ActReactArea3D {
    fn get_area_display(&self) -> GString {
        self.name_display.clone()
    }
    pub fn get_name(&mut self) -> GString {
        if let Some(target) = self.target.as_mut() {
            if target.has_method("get_name_display") {
                return target.call("get_name_display", &[]).to::<GString>();
            }
        }
        self.get_area_display()
    }

    pub fn get_reactor(&self) -> Gd<Object> {
        if let Some(target) = self.target.as_ref() {
            return target.clone().upcast();
        }
        self.base().clone().upcast()
    }
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
        if self.propagation_mode.contains(PropagationMode::Contact) {
            let callable = self.base().callable("on_other_area_act");
            self.base_mut().connect("area_entered", &callable);
        }
        if self.base().has_method("_post_ready") {
            self.base_mut().call_deferred("_post_ready", &[]);
        }
    }
}

#[godot_api]
impl ActReactArea3D {
    #[func(gd_self, virtual)]
    fn post_ready(_s: Gd<Self>) {
        godot_print!("running virtual function…");
    }

    #[func]
    fn on_other_area_act(&self, actor: Gd<ActReactArea3D>) {
        let Some(react) = self.act_react.clone() else {
            return;
        };
        let Some(act) = actor.bind().act_react.clone() else {
            return;
        };

        let mut act_react_executor = ActReactExecutor::singleton();
        let reactor = self
            .target
            .clone()
            .unwrap_or(self.base().clone().upcast::<Node>());
        let mut context = dict! {
            "reactor": reactor
        };
        if let Some(a) = actor.bind().target.clone() {
            let _ = context.insert("actor", a);
        }
        act_react_executor.bind_mut().react(act, react, context);
    }

    #[func]
    pub fn react(&self, act: Gd<ActReactResource>, actor_context: Dictionary) {
        let Some(react) = self.act_react.clone() else {
            return;
        };
        let mut act_react_executor = ActReactExecutor::singleton();
        act_react_executor
            .bind_mut()
            .react(act, react, actor_context);
    }
}
