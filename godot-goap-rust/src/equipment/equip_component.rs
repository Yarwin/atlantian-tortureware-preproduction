use crate::godot_api::item_object::Item;
use crate::multi_function_display::mfd_main::DisplayType;
use godot::classes::Control;
use godot::obj::bounds::DeclUser;
use godot::obj::Bounds;
use godot::prelude::*;
use std::collections::HashMap;

type EqDispatchSelf = fn(Gd<Node3D>, fn(&mut dyn Equipment));
type EqDispatchItem = fn(Gd<Node3D>, Gd<Item>, fn(&mut dyn Equipment, Gd<Item>));
type EqDispatchControl = fn(Gd<Node3D>, Gd<Control>, fn(&mut dyn Equipment, Gd<Control>));

/// a struct that keeps Fn pointers required to dispatch calls to &mut dyn Equipment
#[derive(Debug)]
pub struct EqDispatch {
    dispatch_self: EqDispatchSelf,
    dispatch_item: EqDispatchItem,
    dispatch_control: EqDispatchControl,
}

impl EqDispatch {
    fn new<T>() -> Self
    where
        T: Inherits<Node3D> + GodotClass + Bounds<Declarer = DeclUser> + Equipment,
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
            dispatch_control: |base, control, closure| {
                let mut instance = base.cast::<T>();
                let mut guard: GdMut<T> = instance.bind_mut();
                closure(&mut *guard, control)
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
    T: Inherits<Node3D> + GodotClass + Bounds<Declarer = DeclUser> + Equipment,
{
    unsafe {
        if EQUIPMENT_COMPONENT_REGISTRY.is_none() {
            EQUIPMENT_COMPONENT_REGISTRY = Some(HashMap::new());
        }
        EQUIPMENT_COMPONENT_REGISTRY
            .as_mut()
            .unwrap()
            .entry(name)
            .or_insert_with(|| EqDispatch::new::<T>());
    }
}

pub type ItemEquipmentComponentInit = fn(Gd<Resource>) -> Box<dyn ItemEquipmentComponent>;
static mut ITEM_EQUIPMENT_COMPONENT_REGISTRY: Option<HashMap<GString, ItemEquipmentComponentInit>> =
    None;

pub fn item_equipment_component_registry() -> &'static HashMap<GString, ItemEquipmentComponentInit>
{
    unsafe {
        if ITEM_EQUIPMENT_COMPONENT_REGISTRY.is_none() {
            ITEM_EQUIPMENT_COMPONENT_REGISTRY = Some(HashMap::new());
        }
        ITEM_EQUIPMENT_COMPONENT_REGISTRY.as_ref().unwrap()
    }
}

pub fn register_item_equipment_component<T>(name: GString)
where
    T: Inherits<Resource> + GodotClass + Bounds<Declarer = DeclUser> + EquipmentComponentResource,
{
    unsafe {
        if ITEM_EQUIPMENT_COMPONENT_REGISTRY.is_none() {
            ITEM_EQUIPMENT_COMPONENT_REGISTRY = Some(HashMap::new());
        }
        ITEM_EQUIPMENT_COMPONENT_REGISTRY
            .as_mut()
            .unwrap()
            .entry(name)
            .or_insert_with(|| {
                |base| {
                    let mut instance = base.cast::<T>();
                    let guard: GdMut<T> = instance.bind_mut();
                    guard.init_component()
                }
            });
    }
}

/// a wrapper for Equipment 3Dnode responsible for dispatching all the calls
#[derive(Debug, Clone)]
pub struct EquipmentComponent {
    pub base: Gd<Node3D>,
    dispatch: *const EqDispatch,
}

impl GodotConvert for EquipmentComponent {
    type Via = Gd<Node3D>;
}

impl ToGodot for EquipmentComponent {
    type ToVia<'v> = Self::Via;

    fn to_godot(&self) -> Self::Via {
        self.base.to_godot()
    }
}

impl FromGodot for EquipmentComponent {
    fn try_from_godot(via: Self::Via) -> Result<Self, ConvertError> {
        Ok(Self::new(via))
    }
}

impl Eq for EquipmentComponent {}

impl PartialEq for EquipmentComponent {
    fn eq(&self, other: &Self) -> bool {
        other.base == self.base
    }
}

impl EquipmentComponent {
    pub fn new(base: Gd<Node3D>) -> Self {
        let dispatch = &equipment_component_registry()[&base.get_class()] as *const EqDispatch;

        Self {
            base: base.clone(),
            dispatch,
        }
    }
}

impl Equipment for EquipmentComponent {
    fn initialize(&mut self, item: Gd<Item>) {
        unsafe {
            ((*self.dispatch).dispatch_item)(
                self.base.clone(),
                item,
                |e: &mut dyn Equipment, item: Gd<Item>| e.initialize(item),
            )
        }
    }

    fn take_off(&mut self) {
        unsafe {
            ((*self.dispatch).dispatch_self)(self.base.clone(), |e: &mut dyn Equipment| {
                e.take_off()
            })
        }
    }
    fn activate(&mut self) {
        unsafe {
            ((*self.dispatch).dispatch_self)(self.base.clone(), |e: &mut dyn Equipment| {
                e.activate()
            })
        }
    }
    fn deactivate(&mut self) {
        unsafe {
            ((*self.dispatch).dispatch_self)(self.base.clone(), |e: &mut dyn Equipment| {
                e.deactivate()
            })
        }
    }
    fn reload(&mut self) {
        unsafe {
            ((*self.dispatch).dispatch_self)(self.base.clone(), |e: &mut dyn Equipment| e.reload())
        }
    }
    fn point_down(&mut self) {
        unsafe {
            ((*self.dispatch).dispatch_self)(self.base.clone(), |e: &mut dyn Equipment| {
                e.point_down()
            })
        }
    }
    fn point_up(&mut self) {
        unsafe {
            ((*self.dispatch).dispatch_self)(self.base.clone(), |e: &mut dyn Equipment| {
                e.point_up()
            })
        }
    }

    fn connect_component_to_ui(&mut self, ui: Gd<Control>) {
        unsafe {
            ((*self.dispatch).dispatch_control)(
                self.base.clone(),
                ui,
                |e: &mut dyn Equipment, ui: Gd<Control>| e.connect_component_to_ui(ui),
            )
        }
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
    fn initialize_equipment_scene(&mut self) -> (EquipmentComponent, DisplayType);
}

pub trait Equipment {
    fn initialize(&mut self, item: Gd<Item>);
    /// removes given scene
    fn take_off(&mut self);
    /// called when action button is pressed. Responsible for creating new effects in the world.
    fn activate(&mut self) {}
    /// called when action button is released.
    fn deactivate(&mut self) {}
    fn reload(&mut self) {}
    /// called when equipment should be inactive, but still in scene
    fn point_down(&mut self) {}
    /// called when equipment was inactive, but should be activated
    fn point_up(&mut self) {}

    fn connect_component_to_ui(&mut self, ui: Gd<Control>);
}
