use bevy::{
    prelude::*,
    window::{PresentMode, PrimaryWindow},
};
use bevy_cosmic_edit::*;

fn create_editable_widget(commands: &mut Commands, scale_factor: f32, text: String) -> Entity {
    let attrs =
        AttrsOwned::new(Attrs::new().color(bevy_color_to_cosmic(Color::hex("4d4d4d").unwrap())));
    let placeholder_attrs =
        AttrsOwned::new(Attrs::new().color(bevy_color_to_cosmic(Color::hex("#e6e6e6").unwrap())));
    commands
        .spawn((
            CosmicEditBundle {
                attrs: CosmicAttrs(attrs.clone()),
                metrics: CosmicMetrics {
                    font_size: 18.,
                    line_height: 18. * 1.2,
                    scale_factor,
                },
                max_lines: CosmicMaxLines(1),
                text_setter: CosmicText::OneStyle(text),
                text_position: CosmicTextPosition::Left { padding: 20 },
                mode: CosmicMode::InfiniteLine,
                ..default()
            },
            ButtonBundle {
                border_color: Color::hex("#ededed").unwrap().into(),
                style: Style {
                    border: UiRect::all(Val::Px(3.)),
                    width: Val::Percent(20.),
                    height: Val::Px(50.),
                    left: Val::Percent(40.),
                    top: Val::Px(100.),
                    ..default()
                },
                background_color: Color::WHITE.into(),
                ..default()
            },
            CosmicEditPlaceholderBundle {
                text_setter: PlaceholderText(CosmicText::OneStyle("Type something...".into())),
                attrs: PlaceholderAttrs(placeholder_attrs.clone()),
            },
        ))
        .id()
}

fn create_readonly_widget(commands: &mut Commands, scale_factor: f32, text: String) -> Entity {
    let attrs =
        AttrsOwned::new(Attrs::new().color(bevy_color_to_cosmic(Color::hex("4d4d4d").unwrap())));
    commands
        .spawn((
            CosmicEditBundle {
                attrs: CosmicAttrs(attrs.clone()),
                metrics: CosmicMetrics {
                    font_size: 18.,
                    line_height: 18. * 1.2,
                    scale_factor,
                },
                text_setter: CosmicText::OneStyle(text),
                text_position: CosmicTextPosition::Left { padding: 20 },
                mode: CosmicMode::AutoHeight,
                ..default()
            },
            ButtonBundle {
                border_color: Color::hex("#ededed").unwrap().into(),
                style: Style {
                    border: UiRect::all(Val::Px(3.)),
                    width: Val::Percent(20.),
                    height: Val::Px(50.),
                    left: Val::Percent(40.),
                    top: Val::Px(100.),
                    ..default()
                },
                background_color: Color::WHITE.into(),
                ..default()
            },
            ReadOnly,
        ))
        .id()
}

fn setup(mut commands: Commands, windows: Query<&Window, With<PrimaryWindow>>) {
    commands.spawn(Camera2dBundle::default());
    let primary_window = windows.single();
    let editor = create_editable_widget(
        &mut commands,
        primary_window.scale_factor() as f32,
        "".to_string(),
    );
    commands.insert_resource(Focus(Some(editor)));
}

fn handle_enter(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    mut mode: Query<(Entity, &CosmicEditor, &mut CosmicMode)>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if keys.just_pressed(KeyCode::Return) {
        let scale_factor = windows.single().scale_factor() as f32;
        for (entity, editor, mode) in mode.iter_mut() {
            let text = editor.get_text();
            commands.entity(entity).despawn_recursive();
            if *mode == CosmicMode::AutoHeight {
                let editor = create_editable_widget(&mut commands, scale_factor, text);
                commands.insert_resource(Focus(Some(editor)));
            } else {
                let editor = create_readonly_widget(&mut commands, scale_factor, text);
                commands.insert_resource(Focus(Some(editor)));
            };
        }
    }
}

fn bevy_color_to_cosmic(color: bevy::prelude::Color) -> CosmicColor {
    cosmic_text::Color::rgba(
        (color.r() * 255.) as u8,
        (color.g() * 255.) as u8,
        (color.b() * 255.) as u8,
        (color.a() * 255.) as u8,
    )
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "bevy • text_input".into(),
                        present_mode: PresentMode::AutoVsync,
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                })
                .build(),
        )
        .add_plugins(CosmicEditPlugin::default())
        .add_systems(Update, handle_enter)
        .add_systems(Startup, setup)
        .run();
}
