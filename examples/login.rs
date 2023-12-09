use bevy::{
    prelude::*,
    window::{PrimaryWindow, WindowResolution},
};
use bevy_cosmic_edit::*;

#[derive(Component)]
struct SubmitButton;

#[derive(Component)]
struct UsernameTag;

#[derive(Component)]
struct PasswordTag;

#[derive(Component)]
struct DisplayTag;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    let login_editor = commands
        .spawn(CosmicEditBundle {
            max_lines: CosmicMaxLines(1),
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(300.0, 50.0)),
                    ..default()
                },
                visibility: Visibility::Hidden,
                ..default()
            },
            ..default()
        })
        .insert(CosmicEditPlaceholderBundle {
            text_setter: PlaceholderText(CosmicText::OneStyle("Username".into())),
            attrs: PlaceholderAttrs(AttrsOwned::new(
                Attrs::new().color(bevy_color_to_cosmic(Color::rgb_u8(128, 128, 128))),
            )),
        })
        .insert(UsernameTag)
        .id();

    let password_editor = commands
        .spawn(CosmicEditBundle {
            max_lines: CosmicMaxLines(1),
            ..default()
        })
        .insert(CosmicEditPlaceholderBundle {
            text_setter: PlaceholderText(CosmicText::OneStyle("Password".into())),
            attrs: PlaceholderAttrs(AttrsOwned::new(
                Attrs::new().color(bevy_color_to_cosmic(Color::rgb_u8(128, 128, 128))),
            )),
        })
        .insert(PasswordTag)
        .insert(PasswordInput::default())
        .id();

    let submit_editor = commands
        .spawn(CosmicEditBundle {
            max_lines: CosmicMaxLines(1),
            attrs: CosmicAttrs(AttrsOwned::new(
                Attrs::new().color(bevy_color_to_cosmic(Color::WHITE)),
            )),
            text_setter: CosmicText::OneStyle("Submit".into()),
            fill_color: FillColor(Color::GREEN),
            ..default()
        })
        .insert(ReadOnly)
        .id();

    let display_editor = commands
        .spawn(CosmicEditBundle::default())
        .insert(CosmicEditPlaceholderBundle {
            text_setter: PlaceholderText(CosmicText::OneStyle("Output".into())),
            attrs: PlaceholderAttrs(AttrsOwned::new(
                Attrs::new().color(bevy_color_to_cosmic(Color::rgb_u8(128, 128, 128))),
            )),
        })
        .insert((ReadOnly, DisplayTag))
        .id();

    commands.insert_resource(Focus(Some(login_editor)));

    // Spawn UI
    commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(15.0)),
                width: Val::Px(330.0),

                ..default()
            },
            ..default()
        })
        .with_children(|root| {
            root.spawn(ButtonBundle {
                style: Style {
                    // Size and position of text box
                    width: Val::Px(300.),
                    height: Val::Px(50.),
                    margin: UiRect::all(Val::Px(15.0)),
                    ..default()
                },
                background_color: BackgroundColor(Color::WHITE),
                ..default()
            })
            .insert(CosmicSource(login_editor));

            root.spawn(ButtonBundle {
                style: Style {
                    // Size and position of text box
                    width: Val::Px(300.),
                    height: Val::Px(50.),
                    margin: UiRect::all(Val::Px(15.0)),
                    ..default()
                },
                background_color: BackgroundColor(Color::WHITE),
                ..default()
            })
            .insert(CosmicSource(password_editor));

            root.spawn(ButtonBundle {
                style: Style {
                    // Size and position of text box
                    width: Val::Px(150.),
                    height: Val::Px(50.),
                    margin: UiRect::all(Val::Px(15.0)),
                    border: UiRect::all(Val::Px(3.0)),
                    ..default()
                },
                background_color: BackgroundColor(Color::WHITE),
                border_color: Color::DARK_GREEN.into(),

                ..default()
            })
            .insert(SubmitButton)
            .insert(CosmicSource(submit_editor));

            root.spawn(ButtonBundle {
                style: Style {
                    // Size and position of text box
                    width: Val::Px(300.),
                    height: Val::Px(100.),
                    margin: UiRect::all(Val::Px(15.0)),
                    ..default()
                },
                background_color: BackgroundColor(Color::WHITE),
                ..default()
            })
            .insert(CosmicSource(display_editor));
        });
}

fn bevy_color_to_cosmic(color: bevy::prelude::Color) -> CosmicColor {
    cosmic_text::Color::rgba(
        (color.r() * 255.) as u8,
        (color.g() * 255.) as u8,
        (color.b() * 255.) as u8,
        (color.a() * 255.) as u8,
    )
}

fn change_active_editor_ui(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &CosmicSource),
        (Changed<Interaction>, Without<ReadOnly>),
    >,
) {
    for (interaction, source) in interaction_query.iter_mut() {
        if let Interaction::Pressed = interaction {
            commands.insert_resource(Focus(Some(source.0)));
        }
    }
}

fn print_changed_input(mut evr_type: EventReader<CosmicTextChanged>) {
    for ev in evr_type.read() {
        println!("Changed: {}", ev.0 .1);
    }
}

fn submit_button(
    button_q: Query<&Interaction, With<SubmitButton>>,
    username_q: Query<
        &CosmicEditor,
        (With<UsernameTag>, Without<PasswordTag>, Without<DisplayTag>),
    >,
    password_q: Query<
        &CosmicEditor,
        (With<PasswordTag>, Without<UsernameTag>, Without<DisplayTag>),
    >,
    mut display_q: Query<
        (&mut CosmicEditor, &CosmicAttrs),
        (With<DisplayTag>, Without<UsernameTag>, Without<PasswordTag>),
    >,
    mut font_system: ResMut<CosmicFontSystem>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
) {
    for interaction in button_q.iter() {
        match interaction {
            Interaction::None => {}
            Interaction::Pressed => {
                let u = username_q.single();
                let p = password_q.single();
                let (mut d, attrs) = display_q.single_mut();

                let text = format!(
                    "Submitted!\nUsername: {}\nPassword: {}\n",
                    u.get_text(),
                    p.get_text()
                );

                d.set_text(
                    CosmicText::OneStyle(text),
                    attrs.0.clone(),
                    &mut font_system.0,
                );
            }
            Interaction::Hovered => {
                window.single_mut().cursor.icon = CursorIcon::Hand;
            }
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(330., 480.),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(CosmicEditPlugin {
            change_cursor: CursorConfig::Default,
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, change_active_editor_ui)
        .add_systems(Update, (print_changed_input, submit_button))
        .run();
}
