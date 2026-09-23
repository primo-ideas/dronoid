use bevy::window::{PrimaryWindow, Window, WindowResolution};
use bevy_ecs::{query::With, system::Query};
use clap::Parser;
use display_info::DisplayInfo;
use std::str::FromStr;

use crate::app::gen_name;

#[derive(Parser, Debug)]
struct Args {
    #[arg(default_value_t = false)]
    tls: bool,
    #[arg(default_value_t = String::from_str("").unwrap())]
    suffix: String,
}

pub fn init() -> (String, String, u16, bool, String) {
    dronoid_logger::init();
    (
        gen_name(),
        "127.0.0.1".to_string(),
        8080,
        false,
        String::from_str("dronoid/ws").unwrap(),
    )
}

pub fn setup_display(mut window: Query<&mut Window, With<PrimaryWindow>>) {
    let mut current_display_resolution = (1280, 720);
    let display_infos = DisplayInfo::all().unwrap();

    for display_info in display_infos {
        if display_info.is_builtin {
            current_display_resolution.0 = display_info.width;
            current_display_resolution.1 = display_info.height;
            break;
        }
    }

    for mut window in window.iter_mut() {
        window.borderless_game = false;
        window.fullsize_content_view = false;
        window.resizable = true;
        window.resolution = WindowResolution::new(
            (current_display_resolution.0 as f64 * (2. / 3.)) as u32,
            (current_display_resolution.1 as f64 * (2. / 3.)) as u32,
        )
    }
}
