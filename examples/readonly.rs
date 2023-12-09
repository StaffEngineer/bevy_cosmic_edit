use bevy::prelude::*;
use bevy_cosmic_edit::*;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    let root = commands
        .spawn(NodeBundle {
            style: Style {
                display: Display::Flex,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..default()
            },
            ..default()
        })
        .id();

    let mut attrs = Attrs::new();
    attrs = attrs.family(Family::Name("Victor Mono"));
    attrs = attrs.color(bevy_color_to_cosmic(Color::PURPLE));

    // spawn editor
    let cosmic_edit = commands
        .spawn(CosmicEditBundle {
            attrs: CosmicAttrs(AttrsOwned::new(attrs)),
            text_position: CosmicTextPosition::Center,
            text_setter: CosmicText::OneStyle("😀😀😀 x => y\nRead only widget".to_string()),
            ..default()
        })
        .insert(ReadOnly)
        .id();

    // Spawn the ButtonBundle as a child of root
    commands.entity(root).with_children(|parent| {
        parent
            .spawn(ButtonBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    ..default()
                },
                background_color: BackgroundColor(Color::WHITE),
                ..default()
            })
            // add cosmic source
            .insert(CosmicSource(cosmic_edit));
    });

    commands.insert_resource(Focus(Some(cosmic_edit)));
}

pub fn bevy_color_to_cosmic(color: bevy::prelude::Color) -> CosmicColor {
    cosmic_text::Color::rgba(
        (color.r() * 255.) as u8,
        (color.g() * 255.) as u8,
        (color.b() * 255.) as u8,
        (color.a() * 255.) as u8,
    )
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
