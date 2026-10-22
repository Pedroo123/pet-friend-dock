use cosmic::prelude::*;
use cosmic::iced::*;

pub mod window_helper {
    // Returns the window or pannel ID that is supposed to be the popUp parent.
    // Centralizes the logic for determining which window is the panel, so the app can stay small
    pub fn parent_panel_window_id(core: &cosmic::Core) -> Id {
        core.applet
            .window_id()
            .or_else(|| core.main_window_id())
            .expect("no parent panel window")
    }

    // Compute an rectangle for the pet, so the popUp can spawn exactly on the cosmic dock.
    // Takes the position and sprite size.
    // position: pet offset along the dock.
    // sprite_size: size of the pet sprite in pixels.
    pub fn rectangle_for_pet(position: f32, sprite_size: f32) -> iced::Rectangle {
        let x = position.round() as f32;
        let y = 0;
        let wh = sprite_size.round() as f32;
        iced::Rectangle {
            x,
            y,
            width: wh,
            height: wh,
        }
    }
}