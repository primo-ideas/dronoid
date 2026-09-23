use bevy::{
    color::palettes::{
        css::{DARK_SLATE_GRAY, WHITE},
        tailwind::SLATE_300,
    },
    input_focus::{AutoFocus, tab_navigation::TabIndex},
    text::{EditableText, EditableTextFilter, TextCursorStyle},
    ui::{BackgroundColor, BorderColor, BorderRadius, Node, Val, px, widget::Text},
    utils::default,
};
use bevy_ecs::{
    component::Component, hierarchy::ChildOf, relationship::RelatedSpawnerCommands, system::Res,
};

use crate::app::ProgramOptions;

#[derive(Component)]
pub struct HostField;

#[derive(Component)]
pub struct PortField;

pub fn setup_host_port(
    parent: &mut RelatedSpawnerCommands<'_, ChildOf>,
    program_options: Res<ProgramOptions>,
) {
    let mut host_editable_text = EditableText::new(program_options.hostname.to_string().as_str());
    host_editable_text.cursor_width = 0.4;
    host_editable_text.max_characters = Some(62);
    let mut port_editable_text = EditableText::new(program_options.port.to_string().as_str());
    port_editable_text.cursor_width = 0.4;
    port_editable_text.max_characters = Some(5);

    parent
        .spawn(Node {
            column_gap: Val::Px(10.),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((Node::default(), Text::new("Host")));
            parent.spawn((
                HostField,
                Node {
                    width: px(200),
                    border: px(2.).all(),
                    border_radius: BorderRadius::all(Val::Percent(10.)),
                    ..default()
                },
                host_editable_text,
                TabIndex(0),
                TextCursorStyle {
                    color: bevy_color::Color::Srgba(WHITE),
                    ..Default::default()
                },
                EditableTextFilter::new(|c| c.is_ascii() && c.is_ascii_graphic()),
                BackgroundColor(DARK_SLATE_GRAY.into()),
                BorderColor::all(SLATE_300),
                AutoFocus,
            ));
            parent.spawn((Node::default(), Text::new("Port")));
            parent.spawn((
                PortField,
                Node {
                    width: px(80),
                    border: px(2.).all(),
                    border_radius: BorderRadius::all(Val::Percent(10.)),
                    ..default()
                },
                port_editable_text,
                TabIndex(1),
                TextCursorStyle {
                    color: bevy_color::Color::Srgba(WHITE),
                    ..Default::default()
                },
                EditableTextFilter::new(|c| c.is_ascii() && c.is_ascii_graphic() && c.is_numeric()),
                BackgroundColor(DARK_SLATE_GRAY.into()),
                BorderColor::all(SLATE_300),
            ));
        });
}
