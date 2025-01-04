use crate::thinker_states::animate::AnimationMode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Index;
use strum_macros::{EnumIter, EnumString, IntoStaticStr};

#[derive(
    IntoStaticStr,
    Debug,
    Clone,
    Copy,
    Hash,
    EnumIter,
    EnumString,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
)]
pub enum AnimationType {
    Attack = 0,
    AttackExhaustion,
    AttackPrepare,
    AttackReady,
    AttackRelease,
    CivilianPose,
    Hurt,
    Idle,
    Invalid,
    Patrol,
    Surprised,
    Walk,
    Max,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AnimationProps {
    pub tree_name: String,
    pub name: String,
    pub mode: AnimationMode,
}

#[derive(Debug, Default, Clone)]
pub struct AnimationsData {
    pub(crate) fields: [Option<AnimationProps>; AnimationType::Max as usize],
}

impl From<HashMap<AnimationType, AnimationProps>> for AnimationsData {
    fn from(value: HashMap<AnimationType, AnimationProps>) -> Self {
        let mut animation_props = Self::default();
        for (k, v) in value.into_iter() {
            animation_props.fields[k as usize] = Some(v);
        }
        animation_props
    }
}

impl Index<AnimationType> for AnimationsData {
    type Output = AnimationProps;

    fn index(&self, index: AnimationType) -> &Self::Output {
        return self.index(&index);
    }
}

impl Index<&AnimationType> for AnimationsData {
    type Output = AnimationProps;

    fn index(&self, index: &AnimationType) -> &Self::Output {
        return self.fields[*index as usize].as_ref().unwrap();
    }
}
