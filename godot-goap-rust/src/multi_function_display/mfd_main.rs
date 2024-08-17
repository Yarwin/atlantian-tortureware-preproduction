use godot::classes::{Control, IControl, MarginContainer};
use godot::prelude::*;
use crate::godot_api::gamesys::GameSys;

#[derive(GodotClass)]
#[class(init, base=Control)]
pub struct MultiFunctionDisplay {
    #[init(node = "ActualDisplayContainer/GunDisplay")]
    gun_ui_display: OnReady<Gd<MarginContainer>>,
    base: Base<Control>
}

#[godot_api]
impl IControl for MultiFunctionDisplay {
    fn ready(&mut self) {
        let on_new_gun_for_ui_display = self.base().callable("on_new_gun_for_ui_display");
        GameSys::singleton().connect("new_gun_for_ui_display".into(), on_new_gun_for_ui_display);
    }
}

#[godot_api]
impl MultiFunctionDisplay {
    #[func]
    fn on_new_gun_for_ui_display(&mut self, gun_ui: Gd<Control>) {
        if self.gun_ui_display.get_child_count() > 0 {
            if let Some(mut mut_child) = self.gun_ui_display.get_child(0) {
                mut_child.queue_free();
            }
        }
        self.gun_ui_display.add_child(gun_ui);
    }
}
