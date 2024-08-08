use std::ops::Index;
use godot::prelude::*;
use crate::act_react::stimulis::Stimuli;


/// todo – automate it with some kind of macro
/// all the stims are hardcoded since Godot dictionaries are untyped.
#[derive(GodotClass, Debug)]
#[class(init, base=Resource)]
pub struct ActReactResource {
    #[export]
    pub metaproperties: Array<Gd<ActReactResource>>,
    /// dumb hack before proper groups will be implemented
    #[var(usage_flags = [GROUP, EDITOR, READ_ONLY])]
    acts: u32,
    /// acts that are being emitted by this entity
    #[export]
    pub emits: Array<Gd<Resource>>,
    /// dumb hack before proper groups will be implemented
    #[var(usage_flags = [GROUP, EDITOR, READ_ONLY])]
    reacts: u32,

    #[export]
    pub cold: Array<Gd<Resource>>,
    #[export]
    pub combine: Array<Gd<Resource>>,
    #[export]
    pub damage_bash: Array<Gd<Resource>>,
    #[export]
    pub damage_pierce: Array<Gd<Resource>>,
    #[export]
    pub damage_slash: Array<Gd<Resource>>,
    #[export]
    pub electrify: Array<Gd<Resource>>,
    #[export]
    pub fire: Array<Gd<Resource>>,
    #[export]
    pub frob: Array<Gd<Resource>>,
    #[export]
    pub grab: Array<Gd<Resource>>,
    #[export]
    pub heat: Array<Gd<Resource>>,
    #[export]
    pub kick: Array<Gd<Resource>>,
    #[export]
    pub pain: Array<Gd<Resource>>,
    #[export]
    pub player_frob: Array<Gd<Resource>>,
    #[export]
    pub poison: Array<Gd<Resource>>,
    #[export]
    pub repair: Array<Gd<Resource>>,
    #[export]
    pub slime: Array<Gd<Resource>>,
    #[export]
    pub stun: Array<Gd<Resource>>,
    #[export]
    pub toxic: Array<Gd<Resource>>,
    #[export]
    pub water: Array<Gd<Resource>>,

    base: Base<Resource>
}

impl ActReactResource {
    pub fn is_reacting(&self, other: Gd<ActReactResource>) -> bool {
        for mut act in other.bind().emits.iter_shared() {
            let stimuli: Stimuli = act.get("stim_type".into()).to::<Stimuli>();
            if self[stimuli].is_empty() {
                continue
            }
            let act_context = act.call("get_context".into(), &[]);
            if let Some(mut react) = self[stimuli].iter_shared().next() {
                return if react.has_method("can_react".into()) {
                    react.call("can_react".into(), &[act_context.clone()]).to::<bool>()
                } else {
                    true
                }
            }
        }
        false
    }
}


impl Index<Stimuli> for ActReactResource {
    type Output = Array<Gd<Resource>>;

    fn index(&self, index: Stimuli) -> &Self::Output {
        match index {
            Stimuli::Cold => {&self.cold}
            Stimuli::Combine => {&self.combine}
            Stimuli::DamageBash => {&self.damage_bash }
            Stimuli::DamagePierce => {&self.damage_pierce }
            Stimuli::DamageSlash => {&self.damage_slash }
            Stimuli::Electrify => {&self.electrify}
            Stimuli::Fire => {&self.fire}
            Stimuli::Frob => {&self.frob}
            Stimuli::Grab => {&self.grab}
            Stimuli::Heat => {&self.heat}
            Stimuli::Kick => {&self.kick}
            Stimuli::Pain => {&self.pain}
            Stimuli::PlayerFrob => {&self.player_frob}
            Stimuli::Poison => {&self.poison}
            Stimuli::Repair => {&self.repair}
            Stimuli::Slime => {&self.slime}
            Stimuli::Stun => {&self.stun}
            Stimuli::Toxic => {&self.toxic}
            Stimuli::Water => {&self.water}
        }
    }
}