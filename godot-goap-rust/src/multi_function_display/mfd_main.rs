use godot::classes::{Control, IControl};
use godot::prelude::*;
use crate::equipment::equip_component::{Equipment, EquipmentComponent};
use crate::equipment::gun_ui::GunDisplay;
use crate::godot_api::gamesys::GameSys;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(GodotConvert, Var, Export, Debug, EnumIter, PartialEq, Eq, Copy, Clone)]
#[godot(via = u32)]
pub enum DisplayType {
    None,
    SpreadGunDisplay,
    EnergyGunDisplay,
}



#[derive(GodotClass)]
#[class(init, base=Control)]
pub struct MultiFunctionDisplay {
    #[init(node = "ActualDisplayContainer/GunDisplay/GunUiDisplay")]
    spreadgun_ui_display: OnReady<Gd<GunDisplay>>,
    base: Base<Control>
}

impl MultiFunctionDisplay {
    fn get_display_control(&self, display_type: DisplayType) -> Option<Gd<Control>> {
        match display_type {
            DisplayType::None => None,
            DisplayType::SpreadGunDisplay => Some(self.spreadgun_ui_display.clone().upcast()),
            DisplayType::EnergyGunDisplay => None,
        }
    }
}

#[godot_api]
impl IControl for MultiFunctionDisplay {
    fn ready(&mut self) {
        let on_new_ui_item_equipped = self.base().callable("on_new_ui_item_equipped");
        GameSys::singleton().connect("new_ui_item_equipped".into(), on_new_ui_item_equipped);
        let on_ui_item_taken_off = self.base().callable("on_ui_item_taken_off");
        GameSys::singleton().connect("ui_item_taken_off".into(), on_ui_item_taken_off);
    }
}

#[godot_api]
impl MultiFunctionDisplay {
    #[func]
    fn show_and_connect_display(&mut self, mut eq: EquipmentComponent, display_type: DisplayType) {
        if let Some(mut control) = self.get_display_control(display_type) {
            control.show();
            eq.connect_component_to_ui(control);
        }
    }
    #[func]
    fn hide_and_disconnect_displays(&mut self, display_type_to_keep: DisplayType) {
        for display in DisplayType::iter() {
            if display == display_type_to_keep {
                continue
            }
            if let Some(mut control) = self.get_display_control(display) {
                control.hide();
            }
        }
    }

    #[func]
    fn on_ui_item_taken_off(&mut self) {
        self.hide_and_disconnect_displays(DisplayType::None);
    }

    #[func]
    fn on_new_ui_item_equipped(&mut self, eq: EquipmentComponent, gun_ui: DisplayType) {
        self.hide_and_disconnect_displays(gun_ui);
        self.show_and_connect_display(eq, gun_ui);
    }
}
