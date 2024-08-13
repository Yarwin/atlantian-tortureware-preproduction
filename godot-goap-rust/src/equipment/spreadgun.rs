use godot::prelude::*;
use godot::classes::{AnimationPlayer, PackedScene};
use crate::equipment::equip_component::{Equipment, EquipmentComponent, EquipmentComponentResource, ItemEquipmentComponent, register_equipment_component, register_item_equipment_component};
use crate::godot_api::{CONNECT_DEFERRED, CONNECT_ONE_SHOT};
use crate::godot_api::item_object::Item;

/// represents some gun with some spread pattern
#[derive(GodotClass)]
#[class(base=Resource)]
struct SpreadGunResource {
    #[export]
    pub mag_size: u32,
    #[export]
    pub ammo_count: u32,
    #[export]
    pub gun_scene: Option<Gd<PackedScene>>,
    base: Base<Resource>
}

#[godot_api]
impl IResource for SpreadGunResource {
    fn init(base: Base<Self::Base>) -> Self {
        register_item_equipment_component::<Self>(Self::class_name().to_gstring());
        Self {
            mag_size: 0,
            ammo_count: 0,
            gun_scene: None,
            base
        }
    }
}

impl EquipmentComponentResource for SpreadGunResource {
    fn init_component(&self) -> Box<dyn ItemEquipmentComponent> {
        Box::new(
            SpreadGunItemComponent {
                ammo_count: self.ammo_count,
                data: self.base().clone().cast::<SpreadGunResource>(),
            }
        )
    }
}

pub struct SpreadGunItemComponent {
    ammo_count: u32,
    data: Gd<SpreadGunResource>,
}

impl ItemEquipmentComponent for SpreadGunItemComponent {
    fn initialize_equipment_scene(&mut self) -> EquipmentComponent {
        let mut gun_scene = self.data.bind().gun_scene.as_ref().unwrap().instantiate().unwrap().cast::<SpreadGun>();
        unsafe {
            gun_scene.bind_mut().eq_component = Some(self as *mut SpreadGunItemComponent);
        }
        EquipmentComponent::new(gun_scene.upcast::<Node3D>())
    }
}


#[derive(GodotClass)]
#[class(base=Node3D)]
pub struct SpreadGun {
    pub item: Option<Gd<Item>>,
    animation_player: OnReady<Gd<AnimationPlayer>>,
    /// direct pointer to given item component. Is valid as long as its Item is valid.
    pub eq_component: Option<*mut SpreadGunItemComponent>,
    base: Base<Node3D>
}

#[godot_api]
impl INode3D for SpreadGun {
    fn init(base: Base<Self::Base>) -> Self {
        register_equipment_component::<Self>(Self::class_name().to_gstring());
        Self {
            item: None,
            animation_player: OnReady::manual(),
            eq_component: None,
            base
        }
    }

    fn ready(&mut self) {
        self.animation_player.init(self.base().get_node_as("AnimationPlayer"));
        self.animation_player.play_ex().name("prepare".into()).done();
    }
}

#[godot_api]
impl SpreadGun {
    #[func]
    fn on_takeoff_animation_finished(&mut self, _a_name: Variant) {
        self.item.take().unwrap().emit_signal("taken_off".into(), &[]);
        self.base_mut().queue_free();
    }
}

impl Equipment for SpreadGun {
    fn initialize(&mut self, mut item: Gd<Item>) {
        let on_item_deleted = self.base().callable("queue_free");
        item.connect_ex("item_deleted".into(), on_item_deleted).flags(CONNECT_ONE_SHOT).done();
        self.item = Some(item);
    }

    fn take_off(&mut self) {
        let on_animation_finished = self.base().callable("on_takeoff_animation_finished");
        self.animation_player.connect_ex("animation_finished".into(), on_animation_finished).flags(CONNECT_ONE_SHOT + CONNECT_DEFERRED).done();
        self.animation_player.play_backwards_ex().name("prepare".into()).done();
    }

    fn activate(&mut self) {
        let Some(eq_comp) = self.eq_component.as_mut().map(|c| unsafe { &mut **c }) else { return; };
        if eq_comp.ammo_count < 1 {
            godot_print!("must reload!");
            return;
        }
        if !self.animation_player.is_playing() {
            eq_comp.ammo_count -= 1;
            godot_print!("ammo count: {}", eq_comp.ammo_count);
            self.animation_player.play_ex().name("shoot".into()).done();
            let Some(item) = self.item.as_mut() else {return;};
            item.emit_signal("updated".into(), &[]);
        }
    }
}
