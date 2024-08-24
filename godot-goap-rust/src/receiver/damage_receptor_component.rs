use std::fmt::Debug;
use godot::prelude::*;


#[derive(Debug)]
pub struct ReceivedDamage {
    pub damager: Option<InstanceId>,
    pub strength: f64,
    pub pain: f64,
    pub pos: Vector3,
    pub normal: Vector3,
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
        let damager = via.get("actor").map(|v| v.to::<Gd<Node>>().instance_id());
        Ok(
            Self {
                damager,
                strength,
                pain,
                pos,
                normal,
            }
        )
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
    #[export]
    pub pain: f64,
    #[var]
    pain_taken: f64,
    #[var]
    #[export]
    pub pain_threshold: f64,
    base: Base<Node>
}

impl DamageReceptorComponent {
    pub fn resolve_damage(&mut self, damage: ReceivedDamage) {
        if damage.strength > 0. {
            self.damage_taken += damage.strength;
            self.base_mut().emit_signal("damage_taken".into(),
                                        &[damage.strength.to_variant(), damage.pos.to_variant(), damage.normal.to_variant()]);
        }
        self.pain_taken += damage.pain;

    }
}

#[godot_api]
impl INode for DamageReceptorComponent {
    fn physics_process(&mut self, _delta: f64) {
        if self.damage_taken > 0. {
            self.hp -= self.damage_taken;
            self.damage_taken = 0.;
        }
        if self.hp < 0. {
            let is_gib = if (self.hp.abs() / self.max_hp) > 0.1 {
                true
            } else {
                false
            };
            self.base_mut().emit_signal("health_depleted".into(), &[is_gib.to_variant()]);
        }
    }

    fn ready(&mut self) {
        if self.hp.is_zero_approx() {
            self.hp = self.max_hp;
        }
        // init receptors
    }
}

#[godot_api]
impl DamageReceptorComponent {
    #[signal]
    fn health_depleted(is_gib: bool);
    #[signal]
    fn damage_taken(damage: f64, position: Vector3, normal: Vector3);

    #[func]
    fn hello_world() {
        godot_print!("hello from receptor!");
    }
}
