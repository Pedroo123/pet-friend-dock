use cosmic::iced::{Rectangle, window::Id};

// Returns the window or panel ID that is supposed to be the popup parent.
pub fn parent_panel_window_id(core: &cosmic::Core) -> Id {
    core.applet
        .window_id()
        .or_else(|| core.main_window_id())
        .expect("no parent panel window")
}

// Computes a rectangle for the pet so the popup can anchor to its position on the dock.
pub fn rectangle_for_pet(position: f32, sprite_size: f32) -> Rectangle {
    let x = position.round();
    let size = sprite_size.round();
    Rectangle {
        x,
        y: 0.0,
        width: size,
        height: size,
    }
}
