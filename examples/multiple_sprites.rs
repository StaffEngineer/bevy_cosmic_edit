use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*, window::PrimaryWindow};
use bevy_cosmic_edit::*;

fn setup(mut commands: Commands, windows: Query<&Window, With<PrimaryWindow>>) {
    let primary_window = windows.single();
    let camera_bundle = Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::Custom(Color::WHITE),
        },
        ..default()
    };
    commands.spawn(camera_bundle);

    let mut attrs = Attrs::new();
    attrs = attrs.family(Family::Name("Victor Mono"));
    attrs = attrs.color(bevy_color_to_cosmic(Color::PURPLE));

    let cosmic_edit_1 = (CosmicEditBundle {
        attrs: CosmicAttrs(AttrsOwned::new(attrs)),
        text_position: CosmicTextPosition::Center,
        fill_color: FillColor(Color::ALICE_BLUE),
        text_setter: CosmicText::OneStyle("😀😀😀 x => y".to_string()),
        sprite_bundle: SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2 {
                    x: primary_window.width() / 2.,
                    y: primary_window.height(),
                }),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(-primary_window.width() / 4., 0., 1.)),
            ..default()
        },
        ..default()
    },);

    let cosmic_edit_2 = (CosmicEditBundle {
        attrs: CosmicAttrs(AttrsOwned::new(attrs)),
        text_position: CosmicTextPosition::Center,
        fill_color: FillColor(Color::GRAY.with_a(0.5)),
        text_setter: CosmicText::OneStyle("Widget_2. Click on me".to_string()),
        sprite_bundle: SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2 {
                    x: primary_window.width() / 2.,
                    y: primary_window.height() / 2.,
                }),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(
                primary_window.width() / 4.,
                -primary_window.height() / 4.,
                1.,
            )),
            ..default()
        },
        ..default()
    },);

    let id = commands.spawn(cosmic_edit_1).id();

    commands.insert_resource(Focus(Some(id)));

    commands.spawn(cosmic_edit_2);
}

fn bevy_color_to_cosmic(color: bevy::prelude::Color) -> CosmicColor {
    cosmic_text::Color::rgba(
        (color.r() * 255.) as u8,
        (color.g() * 255.) as u8,
        (color.b() * 255.) as u8,
        (color.a() * 255.) as u8,
    )
}

fn change_active_editor_sprite(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<Input<MouseButton>>,
    mut cosmic_edit_query: Query<
        (&mut Sprite, &GlobalTransform, Entity),
        (With<CosmicEditor>, Without<ReadOnly>),
    >,
    camera_q: Query<(&Camera, &GlobalTransform)>,
) {
    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();
    if buttons.just_pressed(MouseButton::Left) {
        for (sprite, node_transform, entity) in &mut cosmic_edit_query.iter_mut() {
            let size = sprite.custom_size.unwrap_or(Vec2::new(1., 1.));
            let x_min = node_transform.affine().translation.x - size.x / 2.;
            let y_min = node_transform.affine().translation.y - size.y / 2.;
            let x_max = node_transform.affine().translation.x + size.x / 2.;
            let y_max = node_transform.affine().translation.y + size.y / 2.;
            if let Some(pos) = window.cursor_position() {
                if let Some(pos) = camera.viewport_to_world_2d(camera_transform, pos) {
                    if x_min < pos.x && pos.x < x_max && y_min < pos.y && pos.y < y_max {
                        commands.insert_resource(Focus(Some(entity)))
                    };
                }
            };
        }
    }
}

fn main() {
    let font_bytes: &[u8] = include_bytes!("../assets/fonts/VictorMono-Regular.ttf");
    let font_config = CosmicFontConfig {
        fonts_dir_path: None,
        font_bytes: Some(vec![font_bytes]),
        load_system_fonts: true,
    };

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CosmicEditPlugin {
            font_config,
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, change_active_editor_sprite)
        .run();
}
