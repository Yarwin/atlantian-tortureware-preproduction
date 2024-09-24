use std::fmt::Debug;
use std::time::SystemTime;
use godot::prelude::*;


#[derive(Clone, Debug)]
pub struct ReceivedDamage {
    pub damager: Option<InstanceId>,
    pub strength: f64,
    pub pain: f64,
    pub pos: Vector3,
    pub normal: Vector3,
    pub direction: Vector3
}

impl GodotConvert for ReceivedDamage {
    type Via = Dictionary;
}

impl FromGodot for ReceivedDamage {
    fn try_from_godot(via: Self::Via) -> Result<Self, ConvertError> {
        let Some(Ok(strength)) = via.get("strength").map(|v| v.try_to::<f64>()) else {return Err(ConvertError::default())};
        let Some(Ok(pain)) = via.get("pain").map(|v| v.try_to::<f64>()) else {return Err(ConvertError::default())};
        let Some(Ok(pos)) = via.get("position").map(|v| v.try_to::<Vector3>()) else {return Err(ConvertError::default())};
        let Some(Ok(normal)) = via.get("normal").map(|v| v.try_to::<Vector3>()) else {return Err(ConvertError::default())};
        let direction = via.get("direction").map(|v| v.to::<Vector3>()).unwrap_or(Vector3::ZERO);
        let damager = via.get("actor").map(|v| v.to::<Gd<Node>>().instance_id());
        Ok(
            Self {
                damager,
                strength,
                pain,
                pos,
                normal,
                direction,
            }
        )
    }
}

impl ToGodot for ReceivedDamage {
    fn to_godot(&self) -> Self::Via {
        dict! {
            "strength": self.strength,
            "pain": self.pain,
            "position": self.pos,
            "normal": self.normal,
            "actor": self.damager.unwrap(),
            "direction": self.direction,
        }
    }
}

#[derive(GodotClass, Debug)]
#[class(init, base=Node)]
pub struct DamageReceptorComponent {
    #[var]
    #[export]
    pub max_hp: f64,
    #[var]
    pub hp: f64,
    #[var]
    damage_taken: f64,
    #[var]
    pub pain: f64,
    #[var]
    pain_taken: f64,
    #[var]
    #[export]
    pub pain_threshold: f64,
    /// defines how long given entity is invulnerable to pain after entering the pain state
    #[export]
    pub pain_resistance_time: f64,
    #[var]
    #[export(range = (0.0, 1.0))]
    #[init(default = 0.1)]
    pub gib_threshold: f64,
    time_since_last_pain_state: Option<SystemTime>,
    #[export]
    pain_recovery: f64,
    #[export]
    pain_recovery_cooldown: f64,
    #[init(default = SystemTime::now())]
    time_since_last_pain: SystemTime,
    base: Base<Node>
}

impl DamageReceptorComponent {
    pub fn resolve_damage(&mut self, damage: ReceivedDamage) {
        if damage.strength > 0. {
            self.damage_taken += damage.strength;
            self.base_mut().emit_signal("damage_taken".into(),
                                        &[damage.strength.to_variant(), damage.pos.to_variant(), damage.normal.to_variant()]);
        }
        if damage.pain > 0. {
            self.pain_taken += damage.pain;
        }
    }
}

#[godot_api]
impl INode for DamageReceptorComponent {
    fn physics_process(&mut self, delta: f64) {
        if self.damage_taken > 0. {
            self.hp -= self.damage_taken;
            self.damage_taken = 0.;
        }
        if self.hp < 0. {
            let is_gib = (self.hp.abs() / self.max_hp) > self.gib_threshold;
            self.base_mut().emit_signal("health_depleted".into(), &[is_gib.to_variant()]);
        }

        if self.pain_taken > 0. {
            let is_immune_to_pain = self.time_since_last_pain_state
                .map(|lp| lp.elapsed().unwrap().as_secs_f64() < self.pain_resistance_time).unwrap_or(false);
            if !is_immune_to_pain {
                self.pain += self.pain_taken;
                if self.pain > self.pain_threshold {
                    self.time_since_last_pain_state = Some(SystemTime::now());
                    self.pain = 0.;
                    self.base_mut().emit_signal("pain_threshold_achieved".into(), &[]);
                }
                self.time_since_last_pain = SystemTime::now();
            }
            self.pain_taken = 0.;
        } else if self.time_since_last_pain.elapsed().unwrap().as_secs_f64() > self.pain_recovery_cooldown {
            self.pain = (self.pain - self.pain_recovery * delta).max(0.);
        }
    }

    fn ready(&mut self) {
        if self.hp.is_zero_approx() {
            self.hp = self.max_hp;
        }
    }
}

#[godot_api]
impl DamageReceptorComponent {
    #[signal]
    fn health_depleted(is_gib: bool);
    #[signal]
    fn damage_taken(damage: f64, position: Vector3, normal: Vector3);

    #[signal]
    fn pain_threshold_achieved();
}
