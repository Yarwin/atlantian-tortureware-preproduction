use serde::{Deserialize, Serialize};
use std::ops::Index;
use crate::thinker_states::animate::AnimationMode;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationType {
    Walk,
    Idle,
    Patrol,
    AttackPrepare,
    AttackReady,
    AttackExhaustion,
    CivilianPose,
    Hurt,
    Invalid,
    Surprised
}


#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AnimationProps {
    pub name: String,
    pub mode: AnimationMode,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AnimationsData {
    walk: Option<AnimationProps>,
    idle: Option<AnimationProps>,
    patrol: Option<AnimationProps>,
    attack_prepare: Option<AnimationProps>,
    attack_ready: Option<AnimationProps>,
    attack_exhaustion: Option<AnimationProps>,
    civilian_pose: Option<AnimationProps>,
    hurt: Option<AnimationProps>,
    surprised: Option<AnimationProps>
}

impl Index<AnimationType> for AnimationsData {
    type Output = AnimationProps;

    fn index(&self, index: AnimationType) -> &Self::Output {
        return self.index(&index)
    }
}

impl Index<&AnimationType> for AnimationsData {
    type Output = AnimationProps;

    fn index(&self, index: &AnimationType) -> &Self::Output {
        match index {
            AnimationType::Walk => self.walk.as_ref().expect("no animation data!"),
            AnimationType::Idle => self.idle.as_ref().expect("no animation data!"),
            AnimationType::AttackPrepare => {
                self.attack_prepare.as_ref().expect("no animation data!")
            }
            AnimationType::AttackReady => self.attack_ready.as_ref().expect("no animation data!"),
            AnimationType::AttackExhaustion => {
                self.attack_exhaustion.as_ref().expect("no animation data!")
            }
            AnimationType::CivilianPose => self.civilian_pose.as_ref().expect("no animation data!"),
            AnimationType::Hurt => self.hurt.as_ref().expect("no animation data!"),
            AnimationType::Patrol => self.patrol.as_ref().expect("no animation data!"),
            AnimationType::Surprised => self.surprised.as_ref().expect("no animation data!"),
            _ => {
                panic!("no animation data for {:?}!", index)
            }
        }
    }
}
