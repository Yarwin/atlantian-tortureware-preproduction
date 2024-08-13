use std::collections::HashMap;
use godot::prelude::*;
use godot::obj::Bounds;
use godot::obj::bounds::DeclUser;
use crate::godot_api::item_object::Item;


type EqDispatchSelf = fn(Gd<Node3D>, fn(&mut dyn Equipment));
type EqDispatchItem = fn(Gd<Node3D>, Gd<Item>, fn(&mut dyn Equipment, Gd<Item>));

/// a struct that keeps Fn pointers required to dispatch calls to &mut dyn Equipment
#[derive(Debug)]
pub struct EqDispatch {
    dispatch_self: EqDispatchSelf,
    dispatch_item: EqDispatchItem
}

impl EqDispatch {
    fn new<T>() -> Self
        where
            T: Inherits<Node3D> + GodotClass + Bounds<Declarer = DeclUser> + Equipment
    {
        Self {
            dispatch_self: |base, closure| {
                    let mut instance = base.cast::<T>();
                    let mut guard: GdMut<T> = instance.bind_mut();
                    closure(&mut *guard)
                },
            dispatch_item: |base, item, closure| {
                let mut instance = base.cast::<T>();
                let mut guard: GdMut<T> = instance.bind_mut();
                closure(&mut *guard, item)
            },
        }
    }
}


static mut EQUIPMENT_COMPONENT_REGISTRY: Option<HashMap<GString, EqDispatch>> = None;

pub fn equipment_component_registry() -> &'static HashMap<GString, EqDispatch> {

    unsafe {
        if EQUIPMENT_COMPONENT_REGISTRY.is_none() {
            EQUIPMENT_COMPONENT_REGISTRY = Some(HashMap::new());
        }
        EQUIPMENT_COMPONENT_REGISTRY.as_ref().unwrap()
    }
}

// todo – these abominations could be some kind of plugin/macro or whatever
pub fn register_equipment_component<T>(name: GString)
    where
        T: Inherits<Node3D> + GodotClass + Bounds<Declarer = DeclUser> + Equipment
{
    unsafe {
        if EQUIPMENT_COMPONENT_REGISTRY.is_none() {
            EQUIPMENT_COMPONENT_REGISTRY = Some(HashMap::new());
        }
        EQUIPMENT_COMPONENT_REGISTRY.as_mut().unwrap().entry(name).or_insert_with(
            || EqDispatch::new::<T>()
        );
    }
}

pub type ItemEquipmentComponentInit = fn(Gd<Resource>) -> Box<dyn ItemEquipmentComponent>;
static mut ITEM_EQUIPMENT_COMPONENT_REGISTRY: Option<HashMap<GString, ItemEquipmentComponentInit>> = None;

pub fn item_equipment_component_registry() -> &'static HashMap<GString, ItemEquipmentComponentInit> {

    unsafe {
        if ITEM_EQUIPMENT_COMPONENT_REGISTRY.is_none() {
            ITEM_EQUIPMENT_COMPONENT_REGISTRY = Some(HashMap::new());
        }
        ITEM_EQUIPMENT_COMPONENT_REGISTRY.as_ref().unwrap()
    }
}

pub fn register_item_equipment_component<T>(name: GString)
    where
        T: Inherits<Resource> + GodotClass + Bounds<Declarer = DeclUser> + EquipmentComponentResource
{
    unsafe {
        if ITEM_EQUIPMENT_COMPONENT_REGISTRY.is_none() {
            ITEM_EQUIPMENT_COMPONENT_REGISTRY = Some(HashMap::new());
        }
        ITEM_EQUIPMENT_COMPONENT_REGISTRY.as_mut().unwrap().entry(name).or_insert_with(
            || {
                |base| {
                    let mut instance = base.cast::<T>();
                    let guard: GdMut<T> = instance.bind_mut();
                    guard.init_component()
                }
            }
        );
    }
}

/// a wrapper for Equipment 3Dnode responsible for dispatching all the calls
#[derive(Debug)]
pub struct EquipmentComponent {
    pub base: Gd<Node3D>,
    dispatch: *const EqDispatch
}

impl Eq for EquipmentComponent {}

impl PartialEq for EquipmentComponent {
    fn eq(&self, other: &Self) -> bool {
        other.base == self.base
    }
}

impl EquipmentComponent {
    pub fn new(base: Gd<Node3D>) -> Self {
        unsafe {
            let dispatch = &equipment_component_registry()[&base.get_class()] as *const EqDispatch;

        Self {
            base: base.clone(),
            dispatch
        }
        }
    }
}

impl Equipment for EquipmentComponent {
    fn initialize(&mut self, item: Gd<Item>) {
        unsafe { ((*self.dispatch).dispatch_item)(self.base.clone(), item, |e: &mut dyn Equipment, item: Gd<Item>| { e.initialize(item) }) }
    }

    fn take_off(&mut self) {
        unsafe { ((*self.dispatch).dispatch_self)(self.base.clone(), |e: &mut dyn Equipment| { e.take_off() }) }
    }
    fn activate(&mut self) {
        unsafe { ((*self.dispatch).dispatch_self)(self.base.clone(), |e: &mut dyn Equipment| { e.activate() }) }
    }
    fn deactivate(&mut self) {
        unsafe { ((*self.dispatch).dispatch_self)(self.base.clone(), |e: &mut dyn Equipment| { e.deactivate() }) }
    }
}


pub trait EquipmentComponentResource {
    fn init_component(&self) -> Box<dyn ItemEquipmentComponent>;
}

pub fn build_item_equipment_component(base: Gd<Resource>) -> Box<dyn ItemEquipmentComponent> {
    item_equipment_component_registry()[&base.get_class()](base)
}

pub trait ItemEquipmentComponent {
    /// initializes given packed scene and returns Node ready to be attached to the scene tree
    fn initialize_equipment_scene(&mut self) -> EquipmentComponent;
}


pub trait Equipment {
    fn initialize(&mut self, item: Gd<Item>);
    /// removes given scene
    fn take_off(&mut self);
    /// called when action button is pressed. Responsible for creating new effects in the world.
    fn activate(&mut self) {}
    /// called when action button is released.
    fn deactivate(&mut self) {}
}
