use crate::act_react::game_effect::GameEffect;
use crate::act_react::stimulis::Stimuli;
use godot::meta::PropertyInfo;
use godot::prelude::*;
use std::ops::Index;
use strum::IntoEnumIterator;

#[derive(GodotClass)]
#[class(init, tool, base=Resource)]
pub struct ActReactResource {
    /// metaproperties used by given entity.
    /// All acts and reacts defined in metaproperties are being applied before ones defined by given entity.
    pub metaproperties: Array<Gd<ActReactResource>>,
    pub emits: Array<DynGd<Resource, dyn Emitter>>,
    pub reacts: [Array<DynGd<Resource, dyn Reaction>>; Stimuli::MAX as usize],
    base: Base<Resource>,
}

#[godot_api]
impl IResource for ActReactResource {
    fn get_property(&self, property: StringName) -> Option<Variant> {
        let prop_str = property.to_string();
        if prop_str == "metaproperties" {
            return Some(self.metaproperties.to_variant());
        } else if prop_str == "emits" {
            return Some(self.emits.to_variant());
        }
        for stim in Stimuli::iter() {
            if stim == Stimuli::MAX {
                break;
            }
            if prop_str == stim.as_ref() {
                return Some(self.reacts[stim as usize].to_variant());
            }
        }
        None
    }

    fn set_property(&mut self, property: StringName, value: Variant) -> bool {
        let prop_str = property.to_string();
        if prop_str == "metaproperties" {
            self.metaproperties = value.to::<Array<Gd<ActReactResource>>>();
            return true;
        } else if prop_str == "emits" {
            self.emits = value.to::<Array<DynGd<Resource, dyn Emitter>>>();
            return true;
        }
        for stim in Stimuli::iter() {
            if stim == Stimuli::MAX {
                break;
            }
            if prop_str == stim.as_ref() {
                self.reacts[stim as usize] = value.to::<Array<DynGd<Resource, dyn Reaction>>>();
                return true;
            }
        }
        false
    }

    fn get_property_list(&mut self) -> Vec<PropertyInfo> {
        let mut property_list = vec![
            PropertyInfo::new_export::<Array<Gd<ActReactResource>>>("metaproperties"),
            PropertyInfo::new_group("acts", ""),
            PropertyInfo::new_export::<Array<Gd<Resource>>>("emits"),
            PropertyInfo::new_group("reacts", ""),
        ];
        for stim in Stimuli::iter() {
            if stim == Stimuli::MAX {
                break;
            }
            property_list.push(PropertyInfo::new_export::<Array<Gd<Resource>>>(
                stim.as_ref(),
            ));
        }
        property_list
    }
}

impl ActReactResource {
    pub fn get_playerfrob_display(&self) -> GString {
        if let Some(display) = self[Stimuli::PlayerFrob]
            .iter_shared()
            .find_map(|react| react.dyn_bind().get_react_display())
        {
            return display;
        }
        for meta in self.metaproperties.iter_shared() {
            if let Some(display) = meta.bind()[Stimuli::PlayerFrob]
                .iter_shared()
                .find_map(|react| react.dyn_bind().get_react_display())
            {
                return display;
            }
        }
        GString::default()
    }

    pub fn is_reacting(&self, other: Gd<ActReactResource>) -> bool {
        for act in other.bind().emits.iter_shared() {
            let stim_type = act.dyn_bind().get_stim_type();
            let reacts = &self[stim_type];
            if reacts.is_empty() {
                continue;
            }

            let context = act.dyn_bind().get_context();
            for react in reacts.iter_shared() {
                if react.dyn_bind().can_react(&context) {
                    return true;
                }
            }
        }
        false
    }
}

impl Index<Stimuli> for ActReactResource {
    type Output = Array<DynGd<Resource, dyn Reaction>>;

    fn index(&self, index: Stimuli) -> &Self::Output {
        &self.reacts[index as usize]
    }
}

pub trait Emitter {
    fn get_stim_type(&self) -> Stimuli;
    fn get_context(&self) -> Dictionary;
}

pub trait Reaction {
    fn can_react(&self, _context: &Dictionary) -> bool {
        true
    }

    fn get_react_display(&self) -> Option<GString> {
        None
    }

    fn build_effect(
        &self,
        act_context: &Dictionary,
        context: &Dictionary,
    ) -> Option<DynGd<Object, dyn GameEffect>>;
}
