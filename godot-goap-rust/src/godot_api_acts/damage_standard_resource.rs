use crate::act_react::act_react_resource::Emitter;
use crate::act_react::stimulis::Stimuli;
use godot::builtin::math::FloatExt;
use godot::prelude::*;
use rand::prelude::*;
use rand::rng;

#[derive(GodotClass)]
#[class(init, base=Resource)]
pub struct ActDamageStandard {
    #[var]
    #[init(val = Stimuli::DamageStandard)]
    stim_type: Stimuli,
    /// determines raw damage
    #[export]
    pub strength: f64,
    #[export]
    #[init(val = 0.)]
    pub strength_range: f64,
    /// determines force applied to world objects
    #[export]
    #[init(val = 1.0)]
    pub force: f32,
    #[export]
    #[init(val = 0.)]
    pub force_range: f32,
    #[export]
    pub pain: f64,
    #[export]
    #[init(val = 0.)]
    pub pain_range: f64,
}

impl ActDamageStandard {
    fn get_value<T>(default: T, range: T, rng: &mut ThreadRng) -> T
    where
        T: Default
            + std::ops::Add<Output = T>
            + std::ops::Sub<Output = T>
            + rand::distr::uniform::SampleUniform
            + std::cmp::PartialOrd
            + FloatExt,
    {
        match (default.is_zero_approx(), range.is_zero_approx()) {
            (true, true) => T::default(),
            (false, true) => default,
            (true, false) => rng.random_range(T::default()..T::default() + range),
            (false, false) => rng.random_range(default - range..default + range),
        }
    }
}

#[godot_dyn]
impl Emitter for ActDamageStandard {
    fn get_stim_type(&self) -> Stimuli {
        Stimuli::DamageStandard
    }

    fn get_context(&self) -> Dictionary {
        let mut rng = rng();
        let strength: f64 =
            ActDamageStandard::get_value(self.strength, self.strength_range, &mut rng);
        let pain: f64 = ActDamageStandard::get_value(self.pain, self.pain_range, &mut rng);
        let force: f32 = ActDamageStandard::get_value(self.force, self.force_range, &mut rng);

        dict! {
            "strength": strength,
            "force": force,
            "pain": pain
        }
    }
}
