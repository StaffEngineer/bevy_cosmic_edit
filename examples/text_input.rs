use bevy::{prelude::*, window::PresentMode};
use bevy_cosmic_edit::*;
use cosmic_text::LineHeight;

fn create_editable_widget(commands: &mut Commands, text: String) -> Entity {
    let attrs = AttrsOwned::new(
        Attrs::new()
            .color(bevy_color_to_cosmic(Color::hex("4d4d4d").unwrap()))
            .size(18.)
            .line_height(LineHeight::Proportional(1.2)),
    );
    let placeholder_attrs =
        AttrsOwned::new(Attrs::new().color(bevy_color_to_cosmic(Color::hex("#e6e6e6").unwrap())));
    let editor = commands
        .spawn((
            CosmicEditBundle {
                attrs: CosmicAttrs(attrs.clone()),
                max_lines: CosmicMaxLines(1),
                text_setter: CosmicText::OneStyle(text),
                text_position: CosmicTextPosition::Left { padding: 20 },
                mode: CosmicMode::InfiniteLine,
                ..default()
            },
            CosmicEditPlaceholderBundle {
                text_setter: PlaceholderText(CosmicText::OneStyle("Type something...".into())),
                attrs: PlaceholderAttrs(placeholder_attrs.clone()),
            },
        ))
        .id();
    commands
        .spawn(ButtonBundle {
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
        })
        .insert(CosmicSource(editor));

    editor
}

fn create_readonly_widget(commands: &mut Commands, text: String) -> Entity {
    let attrs = AttrsOwned::new(
        Attrs::new()
            .color(bevy_color_to_cosmic(Color::hex("4d4d4d").unwrap()))
            .size(18.)
            .line_height(LineHeight::Proportional(1.2)),
    );

    let editor = commands
        .spawn((
            CosmicEditBundle {
                attrs: CosmicAttrs(attrs.clone()),
                text_setter: CosmicText::OneStyle(text),
                text_position: CosmicTextPosition::Left { padding: 20 },
                mode: CosmicMode::AutoHeight,
                ..default()
            },
            ReadOnly,
        ))
        .id();

    commands
        .spawn(ButtonBundle {
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
        })
        .insert(CosmicSource(editor));

    editor
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    let editor = create_editable_widget(&mut commands, "".to_string());
    commands.insert_resource(Focus(Some(editor)));
}

fn handle_enter(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    mut query_dest: Query<(Entity, &CosmicSource)>,
    mut query_source: Query<(Entity, &CosmicEditor, &CosmicMode)>,
) {
    if keys.just_pressed(KeyCode::Return) {
        for (entity, editor, mode) in query_source.iter_mut() {
            // Remove UI elements
            for (dest_entity, source) in query_dest.iter_mut() {
                if source.0 == entity {
                    commands.entity(dest_entity).despawn_recursive();
                }
            }

            let text = editor.get_text();
            commands.entity(entity).despawn_recursive();
            if *mode == CosmicMode::AutoHeight {
                let editor = create_editable_widget(&mut commands, text);
                commands.insert_resource(Focus(Some(editor)));
            } else {
                let editor = create_readonly_widget(&mut commands, text);
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
