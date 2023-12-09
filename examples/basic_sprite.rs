use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*, window::PrimaryWindow};
use bevy_cosmic_edit::{
    AttrsOwned, CosmicAttrs, CosmicEditBundle, CosmicEditPlugin, CosmicFontConfig, CosmicText,
    CosmicTextPosition, Focus,
};

fn setup(mut commands: Commands, windows: Query<&Window, With<PrimaryWindow>>) {
    let primary_window = windows.single();
    let camera_bundle = Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::Custom(Color::WHITE),
        },
        ..default()
    };
    commands.spawn(camera_bundle);

    let mut attrs = cosmic_text::Attrs::new();
    attrs = attrs.family(cosmic_text::Family::Name("Victor Mono"));
    attrs = attrs.color(cosmic_text::Color::rgb(0x94, 0x00, 0xD3));

    let cosmic_edit = (CosmicEditBundle {
        text_position: CosmicTextPosition::Center,
        attrs: CosmicAttrs(AttrsOwned::new(attrs)),
        text_setter: CosmicText::OneStyle("😀😀😀 x => y".to_string()),
        sprite_bundle: SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(primary_window.width(), primary_window.height())),
                ..default()
            },
            ..default()
        },
        ..default()
    },);

    let cosmic_edit = commands.spawn(cosmic_edit).id();

    commands.insert_resource(Focus(Some(cosmic_edit)));
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
        .run();
}
