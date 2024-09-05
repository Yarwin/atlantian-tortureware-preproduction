use std::ops::Index;
use godot::meta::PropertyInfo;
use godot::prelude::*;
use strum::IntoEnumIterator;
use crate::act_react::stimulis::Stimuli;


#[derive(GodotClass, Debug)]
#[class(init, tool, base=Resource)]
pub struct ActReactResource {
    /// metaproperties used by given entity.
    /// All acts and reacts defined in metaproperties are being applied before ones defined by given entity.
    pub metaproperties: Array<Gd<ActReactResource>>,
    pub emits: Array<Gd<Resource>>,
    pub reacts: [Array<Gd<Resource>>; Stimuli::MAX as usize],

    base: Base<Resource>
}

#[godot_api]
impl IResource for ActReactResource {

    fn get_property(&self, property: StringName) -> Option<Variant> {
        let prop_str = property.to_string();
        if prop_str == "metaproperties" {
            return Some(self.metaproperties.to_variant())
        } else if prop_str == "emits" {
            return Some(self.emits.to_variant())
        }
        for stim in Stimuli::iter() {
            if stim == Stimuli::MAX {break}
            if prop_str == stim.as_ref() {
                return Some(self.reacts[stim as usize].to_variant())
            }
        }
        None
    }

    fn set_property(&mut self, property: StringName, value: Variant) -> bool {
        let prop_str = property.to_string();
        if prop_str == "metaproperties" {
            self.metaproperties = value.to::<Array<Gd<ActReactResource>>>();
            return true
        } else if prop_str == "emits" {
            self.emits = value.to::<Array<Gd<Resource>>>();
            return true
        }
        for stim in Stimuli::iter() {
            if stim == Stimuli::MAX {break}
            if prop_str == stim.as_ref() {
                self.reacts[stim as usize] = value.to::<Array<Gd<Resource>>>();
                return true
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
            if stim == Stimuli::MAX {break}
            property_list.push(
                PropertyInfo::new_export::<Array<Gd<Resource>>>(stim.as_ref()),
            );
        }
        property_list
    }
}

impl ActReactResource {
    pub fn get_playerfrob_display(&self) -> GString {
        if let Some(mut act_with_display) = self[Stimuli::PlayerFrob].iter_shared().find(|a| a.has_method("get_react_display".into())) {
            return act_with_display.call("get_react_display".into(), &[]).to::<GString>();
        } else {
            for meta in self.metaproperties.iter_shared() {
                if let Some(mut act_with_display) = meta.bind()[Stimuli::PlayerFrob].iter_shared().find(|a| a.has_method("get_react_display".into())) {
                    return act_with_display.call("get_react_display".into(), &[]).to::<GString>();
                }
            }
        }
        GString::default()
    }
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
        &self.reacts[index as usize]
    }
}
