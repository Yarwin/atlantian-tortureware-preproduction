use crate::act_react::game_effect::{EffectResult, GameEffect, GameEffectProcessor};
use crate::act_react::game_effect_builder::GameEffectInitializer;
use crate::player_controller::grab_node::GrabNode;
use crate::player_controller::player_frob_controller::PlayerController;
use godot::classes::RigidBody3D;
use godot::prelude::*;

#[derive(GodotClass, Debug)]
#[class(init, base=Resource)]
pub(crate) struct GrabGameEffect {
    base: Base<Resource>,
}

#[godot_api]
impl GrabGameEffect {
    #[func]
    fn get_react_display(&self) -> GString {
        GString::from("Grab")
    }
}

impl GrabGameEffect {
    fn get_grabber(from: &Variant) -> Option<Gd<GrabNode>> {
        match from.try_to::<Gd<GrabNode>>() {
            Ok(g) => Some(g),
            Err(e) => match e.value()?.try_to::<Gd<PlayerController>>() {
                Ok(pc) => Some(pc.bind().grab_node.clone()),
                Err(_) => None,
            },
        }
    }
}

impl GameEffectInitializer for GrabGameEffect {
    fn build(
        &self,
        _act_context: &Dictionary,
        context: &Dictionary,
    ) -> Option<GameEffectProcessor> {
        let Some(grabber) = context.get("actor") else {
            panic!("no actor!")
        };
        let Some(grabbed) = context.get("reactor").map(|v| v.to::<Gd<Node3D>>()) else {
            panic!("no reactor!")
        };
        let effect = Grab {
            grabber,
            grabbed: Some(grabbed),
        };
        let obj = Gd::from_object(effect);
        Some(GameEffectProcessor::new(obj))
    }
}

#[derive(GodotClass, Debug)]
#[class(init, base=Object)]
pub struct Grab {
    grabber: Variant,
    grabbed: Option<Gd<Node3D>>,
}

impl GameEffect for Grab {
    fn execute(&mut self) -> EffectResult {
        let Some(mut grabber) = GrabGameEffect::get_grabber(&self.grabber) else {
            return EffectResult::Failed;
        };
        grabber
            .bind_mut()
            .attach_rigid(self.grabbed.take().unwrap().cast::<RigidBody3D>());
        EffectResult::Free
    }

    fn revert(&mut self) -> EffectResult {
        todo!()
    }
}
