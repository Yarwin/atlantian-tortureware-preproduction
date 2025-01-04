use crate::act_react::act_react_executor::ActReactExecutor;
use crate::act_react::act_react_resource::ActReactResource;
use crate::act_react::react_area_3d::ActReactArea3D;
use crate::character_controler::character_controller_3d::CharacterController3D;
use crate::equipment::equip_component::{
    Equipment, EquipmentComponent, EquipmentComponentResource, ItemEquipmentComponent,
};
use crate::equipment::gun_ui::GunDisplay;
use crate::godot_api::gamesys::{GameSys, GameSystem};
use crate::godot_api::godot_inventory::InventoryAgent;
use crate::godot_api::inventory_manager::InventoryManager;
use crate::godot_api::item_object::{Item, ItemResource};
use crate::godot_api::{CONNECT_DEFERRED, CONNECT_ONE_SHOT};
use crate::godot_entities::rigid_reactive_body3d::WorldObject;
use crate::godot_entities::static_reactive_body_3d::StaticReactiveBody3D;
use crate::multi_function_display::mfd_main::DisplayType;
use crate::player_controller::player_frob_controller::PlayerController;
use godot::classes::{
    AnimationPlayer, Control, GpuParticles3D, Marker3D, PackedScene, ParticleProcessMaterial,
    PhysicsRayQueryParameters3D, Texture2D,
};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=Resource)]
pub(crate) struct SpreadGunAmmo {
    #[export]
    pub accepted_ammo: Option<Gd<ItemResource>>,
    #[export]
    pub trail_particle: Option<Gd<PackedScene>>,
    #[export]
    pub bullet_act_react: Option<Gd<ActReactResource>>,
    #[export]
    pub ui_texture: Option<Gd<Texture2D>>,
    #[export]
    pub ammo_name: GString,
    base: Base<Resource>,
}

#[godot_api]
impl SpreadGunAmmo {
    #[func(virtual)]
    pub fn get_spread(&self) -> Array<Vector2> {
        array![
            Vector2::new(0.01, 0.01),
            Vector2::new(-0.01, -0.01),
            Vector2::new(0.01, -0.01),
            Vector2::new(-0.01, 0.01),
            Vector2::new(0., -0.0141),
            Vector2::new(0., 0.0141),
            Vector2::new(0.0141, 0.),
            Vector2::new(-0.0141, 0.),
        ]
    }
}

/// represents some gun with some spread pattern
#[derive(GodotClass)]
#[class(init, base=Resource)]
pub(crate) struct SpreadGunResource {
    #[export]
    pub mag_size: u32,
    #[export]
    pub ammo_count: u32,
    #[export]
    pub gun_scene: Option<Gd<PackedScene>>,
    #[export]
    pub ui_scene: Option<Gd<PackedScene>>,
    #[export]
    pub gun_sprite: Option<Gd<Texture2D>>,
    #[export]
    pub accepted_ammo: Array<Gd<SpreadGunAmmo>>,
    #[export]
    pub current_ammo: Option<Gd<SpreadGunAmmo>>,
    #[export]
    pub gun_name: GString,
    base: Base<Resource>,
}

#[godot_api]
impl IResource for SpreadGunResource {}

impl EquipmentComponentResource for SpreadGunResource {
    fn init_component(&self) -> Box<dyn ItemEquipmentComponent> {
        Box::new(SpreadGunItemComponent {
            ammo_count: self.ammo_count,
            current_ammo: self.current_ammo.clone(),
            data: self.base().clone().cast::<SpreadGunResource>(),
        })
    }
}

pub struct SpreadGunItemComponent {
    pub(crate) ammo_count: u32,
    pub(crate) current_ammo: Option<Gd<SpreadGunAmmo>>,
    pub(crate) data: Gd<SpreadGunResource>,
}

impl ItemEquipmentComponent for SpreadGunItemComponent {
    fn initialize_equipment_scene(&mut self) -> (EquipmentComponent, DisplayType) {
        let mut gun_scene = self
            .data
            .bind()
            .gun_scene
            .as_ref()
            .unwrap()
            .instantiate()
            .unwrap()
            .cast::<SpreadGun>();
        gun_scene.bind_mut().eq_component = Some(self as *mut SpreadGunItemComponent);
        (
            EquipmentComponent::new(gun_scene.upcast::<Node3D>()),
            DisplayType::SpreadGunDisplay,
        )
    }
}

#[derive(GodotClass)]
#[class(init, base=Node3D)]
pub struct SpreadGun {
    pub item: Option<Gd<Item>>,
    #[init(val = OnReady::manual())]
    pub player: OnReady<Gd<PlayerController>>,
    #[export]
    ray_length: f32,
    #[init(node = "AnimationPlayer")]
    animation_player: OnReady<Gd<AnimationPlayer>>,
    #[init(node = "../../Camera3D")]
    camera: OnReady<Gd<Camera3D>>,
    #[init(node = "anchor/Muzzle")]
    muzzle: OnReady<Gd<Marker3D>>,
    excluded: Array<Rid>,
    /// direct pointer to given item component. Is valid as long as its Item is valid.
    eq_component: Option<*mut SpreadGunItemComponent>,
    base: Base<Node3D>,
}

impl SpreadGun {
    // reloads gun with some very specific ammo
    fn reload_with_concrete_ammo(
        &mut self,
        eq_component: &mut SpreadGunItemComponent,
        reloaded: &mut u32,
        to_reload: &mut u32,
        player_inventories_ids: Vec<u32>,
    ) {
        let accepted_ammo_item_type = {
            let Some(ammo_type) = eq_component.current_ammo.as_ref().map(|a| a.bind()) else {
                return;
            };
            let accepted_ammo = ammo_type.accepted_ammo.as_ref().unwrap();
            let accepted_ammo_bind = accepted_ammo.bind();
            accepted_ammo_bind.inventory.clone().unwrap()
        };

        for inventory in player_inventories_ids {
            let items = InventoryManager::singleton()
                .bind()
                .get_items_of_the_same_type(inventory, accepted_ammo_item_type.clone());
            for item in items.iter_shared() {
                if *to_reload == 0 {
                    return;
                }
                let stack = {
                    let item_bind = item.bind();
                    item_bind.inventory.as_ref().unwrap().stack
                };
                let to_remove = if stack > *to_reload {
                    *to_reload
                } else {
                    stack
                };
                InventoryManager::singleton()
                    .bind_mut()
                    .reduce_stack(item, to_remove);
                *reloaded += to_remove;
                *to_reload -= to_remove;
            }
        }
    }

    /// reloads the weapon.
    /// returns true if weapon is being reloaded, false otherwise
    fn reload(&mut self) {
        let eq_comp = self
            .eq_component
            .as_mut()
            .map(|c| unsafe { &mut **c })
            .unwrap();
        let eq_data = eq_comp.data.bind();
        let mut to_reload = eq_data.mag_size - eq_comp.ammo_count;
        drop(eq_data);

        if to_reload == 0 {
            return;
        };
        let mut reloaded = 0u32;
        let player_inventories: Vec<u32> = self
            .base()
            .get_tree()
            .unwrap()
            .get_nodes_in_group("player_inventory")
            .iter_shared()
            .map(|n| n.cast::<InventoryAgent>().bind().id)
            .collect();

        if eq_comp.current_ammo.is_some() {
            self.reload_with_concrete_ammo(
                eq_comp,
                &mut reloaded,
                &mut to_reload,
                player_inventories,
            );
        }

        if reloaded > 0 {
            let new_ammo_count = eq_comp.ammo_count + reloaded;
            let on_animation_finished = self
                .base()
                .callable("on_reload_animation_finished")
                .bindv(&varray![new_ammo_count.to_variant()]);
            self.animation_player.play_ex().name("reload").done();
            self.animation_player
                .connect_ex("animation_finished", &on_animation_finished)
                .flags(CONNECT_ONE_SHOT + CONNECT_DEFERRED)
                .done();
            self.base_mut()
                .emit_signal("gun_status_changed", &["reloading".to_variant()]);
        }
    }

    fn get_act_react_context(
        &mut self,
        collision: Dictionary,
        ammo: &Gd<SpreadGunAmmo>,
    ) -> Option<(Gd<ActReactResource>, Gd<ActReactResource>, Dictionary)> {
        let collider = collision.get("collider")?;
        let create_context = |this: &mut SpreadGun, col: Dictionary| {
            dict! {
                    "actor": this.player.clone(),
                    "position": col.get("position").unwrap().to::<Vector3>(),
                    "direction": col.get("position").unwrap().to::<Vector3>().direction_to(this.muzzle.get_global_position()),
                    "normal": col.get("normal").unwrap().to::<Vector3>(),
            }
        };

        if let Ok(reactive_body) = collider.try_to::<Gd<StaticReactiveBody3D>>() {
            let act_react = reactive_body.bind().act_react.clone()?;
            let mut context = create_context(self, collision.clone());
            context.set("reactor", reactive_body.clone());
            Some((
                ammo.bind().bullet_act_react.clone().unwrap(),
                act_react,
                context,
            ))
        } else if let Ok(rigid_body) = collider.try_to::<Gd<WorldObject>>() {
            let rigid_bind = rigid_body.bind();
            let act_react_area = rigid_bind.act_react_area.as_ref()?;
            let act_react = act_react_area.bind().act_react.clone()?;
            let mut context = create_context(self, collision.clone());
            context.set("reactor", rigid_body.clone());
            Some((
                ammo.bind().bullet_act_react.clone().unwrap(),
                act_react,
                context,
            ))
        } else if let Ok(act_react_area) = collider.try_to::<Gd<ActReactArea3D>>() {
            let act_react = act_react_area.bind().act_react.clone()?;
            let mut context = create_context(self, collision.clone());
            context.set(
                "reactor",
                act_react_area
                    .bind()
                    .target
                    .clone()
                    .unwrap_or(act_react_area.clone().upcast::<Node>()),
            );
            Some((
                ammo.bind().bullet_act_react.clone().unwrap(),
                act_react,
                context,
            ))
        } else {
            None
        }
    }

    fn apply_hitscan_collision(&mut self, collision: Dictionary, ammo: &Gd<SpreadGunAmmo>) {
        if let Some((actor, reactor, context)) = self.get_act_react_context(collision.clone(), ammo)
        {
            ActReactExecutor::singleton()
                .bind_mut()
                .react(actor, reactor, context);
        }
        let Some(col) = collision.get("position") else {
            return;
        };
        let Some(normal) = collision.get("normal") else {
            return;
        };
        // draw some trails
        if let Some(particle) = ammo.bind().trail_particle.as_ref() {
            let mut particle = particle.instantiate().unwrap().cast::<GpuParticles3D>();
            let mat = particle
                .get_process_material()
                .unwrap()
                .cast::<ParticleProcessMaterial>();
            let v = mat.get("initial_velocity_min").to::<f64>();
            let lifetime = col
                .to::<Vector3>()
                .distance_to(self.muzzle.get_global_position()) as f64
                / v;
            particle.set_lifetime(lifetime);
            self.muzzle.add_child(&particle);
        }
        GameSys::singleton().emit_signal("new_hitscan_collision_registered", &[col, normal]);
    }

    fn shoot(&mut self) {
        let ammo = unsafe {
            (**self.eq_component.as_mut().unwrap())
                .current_ammo
                .as_ref()
                .unwrap()
        };
        let mut space_state = self
            .base()
            .get_world_3d()
            .unwrap()
            .get_direct_space_state()
            .unwrap();
        let viewport_size = self.base().get_viewport().unwrap().get_visible_rect();
        let screen_center = viewport_size.size / 2.0;
        let from = self.camera.project_ray_origin(screen_center);
        let spreads = ammo.bind().get_spread();
        for spread in spreads.iter_shared() {
            let to = from
                + self
                    .camera
                    .project_ray_normal(screen_center + viewport_size.size * spread)
                    * self.ray_length;
            let mut query = PhysicsRayQueryParameters3D::create(from, to).unwrap();
            query.set_collision_mask(64 + 1);
            query.set_collide_with_bodies(true);
            query.set_collide_with_areas(true);
            query.set_exclude(&self.excluded);
            let result = space_state.intersect_ray(&query);
            if result.is_empty() {
                continue;
            };
            self.apply_hitscan_collision(result, ammo);
        }
        self.base_mut().emit_signal("shoot", &[]);
    }
}

#[godot_api]
impl INode3D for SpreadGun {
    fn ready(&mut self) {
        let player = self
            .base()
            .get_tree()
            .unwrap()
            .get_first_node_in_group("player")
            .unwrap()
            .cast::<CharacterController3D>();
        self.excluded.push(player.get_rid());
        self.animation_player.play_ex().name("prepare").done();
        self.player.init(
            self.base()
                .get_tree()
                .unwrap()
                .get_first_node_in_group("player_controller")
                .unwrap()
                .cast::<PlayerController>(),
        );
    }
}

#[godot_api]
impl SpreadGun {
    #[signal]
    fn shoot();

    #[signal]
    fn reloaded();

    #[signal]
    fn gun_status_changed(new_status: GString);

    #[signal]
    fn taken_off();

    #[func]
    fn primary_use(&mut self) {
        self.shoot();
    }

    #[func]
    fn reset_status(&mut self, _a_name: StringName) {
        self.base_mut()
            .emit_signal("gun_status_changed", &[GString::default().to_variant()]);
    }

    #[func]
    fn on_reload_animation_finished(&mut self, _a_name: StringName, new_ammo_count: u32) {
        let Some(eq_component) = self.eq_component.as_mut().map(|c| unsafe { &mut **c }) else {
            return;
        };
        eq_component.ammo_count = new_ammo_count;
        let Some(item) = self.item.as_mut() else {
            return;
        };
        item.emit_signal("updated", &[]);
        self.base_mut()
            .emit_signal("gun_status_changed", &[GString::default().to_variant()]);
        self.base_mut().emit_signal("reloaded", &[]);
    }

    #[func]
    fn on_takeoff_animation_finished(&mut self, _a_name: Variant) {
        self.item.take().unwrap().emit_signal("taken_off", &[]);
        self.base_mut().emit_signal("taken_off", &[]);
        GameSys::singleton().emit_signal("ui_item_taken_off", &[]);
        self.base_mut().queue_free();
    }

    #[func]
    fn take_off(&mut self) {
        self.base_mut()
            .emit_signal("gun_status_changed", &["taking off".to_variant()]);
        let on_animation_finished = self.base().callable("on_takeoff_animation_finished");
        self.animation_player
            .connect_ex("animation_finished", &on_animation_finished)
            .flags(CONNECT_ONE_SHOT + CONNECT_DEFERRED)
            .done();
        self.animation_player
            .play_backwards_ex()
            .name("prepare")
            .done();
    }
}

impl Equipment for SpreadGun {
    fn initialize(&mut self, mut item: Gd<Item>) {
        let on_item_deleted = self.base().callable("queue_free");
        item.connect_ex("item_deleted", &on_item_deleted)
            .flags(CONNECT_ONE_SHOT)
            .done();
        self.item = Some(item);
    }

    fn take_off(&mut self) {
        if self.animation_player.is_playing() {
            // take off after animation stops playing
            let callable = Callable::from_local_fn("take_off", |args: &[&Variant]| {
                let mut iter = args.iter();
                let (Some(_animation_name), Some(base)) = (iter.next(), iter.next()) else {
                    return Err(());
                };
                let Ok(mut spreadgun) = base.try_to::<Gd<SpreadGun>>() else {
                    return Err(());
                };
                spreadgun.bind_mut().take_off();
                Ok(Variant::nil())
            })
            .bindv(&varray![self.base().clone().to_variant()]);
            self.animation_player
                .connect_ex("animation_finished", &callable)
                .flags(CONNECT_DEFERRED + CONNECT_ONE_SHOT)
                .done();
            return;
        }
        self.take_off();
    }

    fn activate(&mut self) {
        if self.animation_player.is_playing() {
            return;
        }
        let Some(eq_comp) = self.eq_component.as_mut().map(|c| unsafe { &mut **c }) else {
            return;
        };
        if eq_comp.ammo_count < 1 {
            self.reload();
            return;
        }
        eq_comp.ammo_count -= 1;
        self.animation_player.play_ex().name("shoot").done();
    }

    fn reload(&mut self) {
        self.reload();
    }

    fn point_down(&mut self) {
        self.animation_player.play_ex().name("point_down").done();
    }

    fn point_up(&mut self) {
        self.animation_player
            .play_backwards_ex()
            .name("point_down")
            .done();
    }

    fn connect_component_to_ui(&mut self, gun_ui: Gd<Control>) {
        let on_shoot = gun_ui.callable("on_shoot");
        self.base_mut().connect("shoot", &on_shoot);
        let on_reloaded = gun_ui.callable("on_reloaded");
        self.base_mut().connect("reloaded", &on_reloaded);
        let on_status_changed = gun_ui.callable("on_gun_status_changed");
        self.base_mut()
            .connect("gun_status_changed", &on_status_changed);
        let on_gun_taken_off = gun_ui.callable("on_gun_taken_off");
        self.base_mut().connect("taken_off", &on_gun_taken_off);

        let mut gun_display = gun_ui.cast::<GunDisplay>();
        gun_display.bind_mut().eq_component = Some(self.eq_component.unwrap());
        gun_display.bind_mut().init_with_component();
    }
}
