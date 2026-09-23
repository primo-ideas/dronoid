use bevy::window::{PrimaryWindow, Window, WindowResolution};
use bevy_ecs::{query::With, system::Query};

use crate::app::gen_name;

pub fn init() -> (String, String, u16, bool, String) {
    use std::str::FromStr;
    (
        gen_name(),
        web_sys::window().unwrap().location().hostname().unwrap(),
        u16::from_str(
            web_sys::window()
                .unwrap()
                .location()
                .port()
                .unwrap()
                .as_str(),
        )
        .unwrap(),
        true,
        "dronoid/ws".to_string(),
    )
}

pub fn setup_display(mut q_window: Query<&mut Window, With<PrimaryWindow>>) {
    use web_sys::window;
    let window = window().unwrap();
    let width = window.inner_width().unwrap().as_f64().unwrap() as u32;
    let height = window.inner_height().unwrap().as_f64().unwrap() as u32;
    for mut window in q_window.iter_mut() {
        window.fullsize_content_view = true;
        window.resolution = WindowResolution::new(width, height)
    }
}
