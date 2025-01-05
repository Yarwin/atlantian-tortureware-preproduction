
pub enum EffectResult {
    /// effect has been executed and command object can be freed
    Free,
    /// effect should be reverted after `n` seconds
    Revert(f64),
    /// Failed to execute given command
    Failed,
}

impl PartialEq for EffectResult {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (EffectResult::Free, EffectResult::Free) => true,
            (EffectResult::Revert(_), EffectResult::Revert(_)) => true,
            (EffectResult::Failed, EffectResult::Failed) => true,
            (_, _) => false,
        }
    }
}

impl Eq for EffectResult {}


pub trait GameEffect {
    fn execute(&mut self) -> EffectResult;
    fn revert(&mut self) -> EffectResult {
        EffectResult::Free
    }
}
