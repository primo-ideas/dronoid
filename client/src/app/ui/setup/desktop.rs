use bevy::{
    color::palettes::css::{DARK_SLATE_GRAY, WHITE},
    input_focus::{AutoFocus, tab_navigation::TabIndex},
    text::{EditableText, EditableTextFilter, FontSize, TextCursorStyle, TextFont},
    ui::{AlignItems, BackgroundColor, Node, px, widget::Text},
    utils::default,
};
use bevy_ecs::{
    component::Component, hierarchy::ChildOf, relationship::RelatedSpawnerCommands, system::Res,
};

use crate::app::{
    ProgramOptions,
    ui::{BORDER_THICKNESS, FONT_SIZE, PADDING, border_color, border_radius},
};

#[derive(Component)]
pub struct HostField;

#[derive(Component)]
pub struct PortField;

pub fn host_port(
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
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Node {
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                Text::new("Host"),
                TextFont::from_font_size(FontSize::VMin(FONT_SIZE)),
            ));
            parent.spawn((
                HostField,
                Node {
                    padding: px(PADDING).all(),
                    width: px(200),
                    border: px(BORDER_THICKNESS).all(),
                    border_radius: border_radius(),
                    ..default()
                },
                host_editable_text,
                TextFont::from_font_size(FontSize::VMin(FONT_SIZE)),
                TabIndex(0),
                TextCursorStyle {
                    color: bevy_color::Color::Srgba(WHITE),
                    ..Default::default()
                },
                EditableTextFilter::new(|c| c.is_ascii() && c.is_ascii_graphic()),
                BackgroundColor(DARK_SLATE_GRAY.into()),
                border_color(),
                AutoFocus,
            ));
            parent.spawn((
                Node {
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                Text::new("Port"),
                TextFont::from_font_size(FontSize::VMin(FONT_SIZE)),
            ));
            parent.spawn((
                PortField,
                Node {
                    padding: px(PADDING).all(),
                    width: px(80),
                    border: px(BORDER_THICKNESS).all(),
                    border_radius: border_radius(),
                    ..default()
                },
                port_editable_text,
                TextFont::from_font_size(FontSize::VMin(FONT_SIZE)),
                TabIndex(1),
                TextCursorStyle {
                    color: bevy_color::Color::Srgba(WHITE),
                    ..Default::default()
                },
                EditableTextFilter::new(|c| c.is_ascii() && c.is_ascii_graphic() && c.is_numeric()),
                BackgroundColor(DARK_SLATE_GRAY.into()),
                border_color(),
            ));
        });
}
