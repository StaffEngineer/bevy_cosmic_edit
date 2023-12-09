#![allow(clippy::type_complexity)]

use bevy::prelude::*;
use bevy_cosmic_edit::*;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    let root = commands
        .spawn(NodeBundle {
            style: bevy::prelude::Style {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..default()
            },
            ..default()
        })
        .id();

    let attrs = Attrs::new();
    let serif_attrs = attrs.family(Family::Serif);
    let mono_attrs = attrs.family(Family::Monospace);
    let comic_attrs = attrs.family(Family::Name("Comic Neue"));
    let lines: Vec<Vec<(String, AttrsOwned)>> = vec![
        vec![
            (String::from("Big"), AttrsOwned::new(attrs.size(48.))),
            (
                String::from("B"),
                AttrsOwned::new(attrs.weight(FontWeight::BOLD)),
            ),
            (String::from("old "), AttrsOwned::new(attrs)),
            (
                String::from("I"),
                AttrsOwned::new(attrs.style(FontStyle::Italic)),
            ),
            (String::from("talic "), AttrsOwned::new(attrs)),
            (String::from("f"), AttrsOwned::new(attrs)),
            (String::from("i "), AttrsOwned::new(attrs)),
            (
                String::from("f"),
                AttrsOwned::new(attrs.weight(FontWeight::BOLD)),
            ),
            (String::from("i "), AttrsOwned::new(attrs)),
            (
                String::from("f"),
                AttrsOwned::new(attrs.style(FontStyle::Italic)),
            ),
            (String::from("i "), AttrsOwned::new(attrs)),
        ],
        vec![
            (String::from("Sans-Serif Normal "), AttrsOwned::new(attrs)),
            (
                String::from("Sans-Serif Bold "),
                AttrsOwned::new(attrs.weight(FontWeight::BOLD)),
            ),
            (String::from("small"), AttrsOwned::new(attrs.size(10.))),
            (
                String::from("Sans-Serif Italic "),
                AttrsOwned::new(attrs.style(FontStyle::Italic)),
            ),
            (
                String::from("Sans-Serif Bold Italic"),
                AttrsOwned::new(attrs.weight(FontWeight::BOLD).style(FontStyle::Italic)),
            ),
        ],
        vec![
            (String::from("Serif Normal "), AttrsOwned::new(serif_attrs)),
            (
                String::from("Serif Bold "),
                AttrsOwned::new(serif_attrs.weight(FontWeight::BOLD)),
            ),
            (
                String::from("Serif Italic "),
                AttrsOwned::new(serif_attrs.style(FontStyle::Italic)),
            ),
            (
                String::from("Serif Bold Italic"),
                AttrsOwned::new(
                    serif_attrs
                        .weight(FontWeight::BOLD)
                        .style(FontStyle::Italic),
                ),
            ),
        ],
        vec![(String::from("small"), AttrsOwned::new(attrs.size(10.)))],
        vec![
            (String::from("Mono Normal "), AttrsOwned::new(mono_attrs)),
            (String::from("huge"), AttrsOwned::new(attrs.size(100.))),
            (
                String::from("Mono Bold "),
                AttrsOwned::new(mono_attrs.weight(FontWeight::BOLD)),
            ),
            (
                String::from("Mono Italic "),
                AttrsOwned::new(mono_attrs.style(FontStyle::Italic)),
            ),
            (
                String::from("Mono Bold Italic"),
                AttrsOwned::new(mono_attrs.weight(FontWeight::BOLD).style(FontStyle::Italic)),
            ),
        ],
        vec![
            (String::from("Comic Normal "), AttrsOwned::new(comic_attrs)),
            (
                String::from("Comic Bold "),
                AttrsOwned::new(comic_attrs.weight(FontWeight::BOLD)),
            ),
            (
                String::from("Comic Italic "),
                AttrsOwned::new(comic_attrs.style(FontStyle::Italic)),
            ),
            (
                String::from("Comic Bold Italic"),
                AttrsOwned::new(
                    comic_attrs
                        .weight(FontWeight::BOLD)
                        .style(FontStyle::Italic),
                ),
            ),
        ],
        vec![
            (
                String::from("R"),
                AttrsOwned::new(attrs.color(bevy_color_to_cosmic(Color::RED))),
            ),
            (
                String::from("A"),
                AttrsOwned::new(attrs.color(bevy_color_to_cosmic(Color::ORANGE))),
            ),
            (
                String::from("I"),
                AttrsOwned::new(attrs.color(bevy_color_to_cosmic(Color::YELLOW))),
            ),
            (
                String::from("N"),
                AttrsOwned::new(attrs.color(bevy_color_to_cosmic(Color::GREEN))),
            ),
            (
                String::from("B"),
                AttrsOwned::new(attrs.color(bevy_color_to_cosmic(Color::BLUE))),
            ),
            (
                String::from("O"),
                AttrsOwned::new(attrs.color(bevy_color_to_cosmic(Color::INDIGO))),
            ),
            (
                String::from("W "),
                AttrsOwned::new(attrs.color(bevy_color_to_cosmic(Color::PURPLE))),
            ),
            (
                String::from("Red "),
                AttrsOwned::new(attrs.color(bevy_color_to_cosmic(Color::RED))),
            ),
            (
                String::from("Orange "),
                AttrsOwned::new(attrs.color(bevy_color_to_cosmic(Color::ORANGE))),
            ),
            (
                String::from("Yellow "),
                AttrsOwned::new(attrs.color(bevy_color_to_cosmic(Color::YELLOW))),
            ),
            (
                String::from("Green "),
                AttrsOwned::new(attrs.color(bevy_color_to_cosmic(Color::GREEN))),
            ),
            (
                String::from("Blue "),
                AttrsOwned::new(attrs.color(bevy_color_to_cosmic(Color::BLUE))),
            ),
            (
                String::from("Indigo "),
                AttrsOwned::new(attrs.color(bevy_color_to_cosmic(Color::INDIGO))),
            ),
            (
                String::from("Violet "),
                AttrsOwned::new(attrs.color(bevy_color_to_cosmic(Color::PURPLE))),
            ),
            (
                String::from("U"),
                AttrsOwned::new(attrs.color(bevy_color_to_cosmic(Color::PURPLE))),
            ),
            (
                String::from("N"),
                AttrsOwned::new(attrs.color(bevy_color_to_cosmic(Color::INDIGO))),
            ),
            (
                String::from("I"),
                AttrsOwned::new(attrs.color(bevy_color_to_cosmic(Color::BLUE))),
            ),
            (
                String::from("C"),
                AttrsOwned::new(attrs.color(bevy_color_to_cosmic(Color::GREEN))),
            ),
            (
                String::from("O"),
                AttrsOwned::new(attrs.color(bevy_color_to_cosmic(Color::YELLOW))),
            ),
            (
                String::from("R"),
                AttrsOwned::new(attrs.color(bevy_color_to_cosmic(Color::ORANGE))),
            ),
            (
                String::from("N"),
                AttrsOwned::new(attrs.color(bevy_color_to_cosmic(Color::RED))),
            ),
        ],
        vec![(
            String::from("生活,삶,जिंदगी 😀 FPS"),
            AttrsOwned::new(attrs.color(bevy_color_to_cosmic(Color::RED))),
        )],
    ];

    let cosmic_edit_1 = commands
        .spawn(CosmicEditBundle {
            text_position: bevy_cosmic_edit::CosmicTextPosition::Center,
            attrs: CosmicAttrs(AttrsOwned::new(attrs)),
            text_setter: CosmicText::MultiStyle(lines),
            ..default()
        })
        .id();

    let mut attrs_2 = Attrs::new();
    attrs_2 = attrs_2.family(Family::Name("Times New Roman"));
    attrs_2.color_opt = Some(bevy_color_to_cosmic(Color::PURPLE));

    let cosmic_edit_2 = commands
        .spawn(CosmicEditBundle {
            attrs: CosmicAttrs(AttrsOwned::new(attrs_2)),
            text_position: CosmicTextPosition::Center,
            text_setter: CosmicText::OneStyle("Widget 2.\n Click on me =>".to_string()),
            ..default()
        })
        .id();

    // Spawn the CosmicEditUiBundles as children of root
    commands.entity(root).with_children(|parent| {
        parent
            .spawn(ButtonBundle {
                style: Style {
                    width: Val::Percent(50.),
                    height: Val::Percent(100.),
                    ..default()
                },
                background_color: BackgroundColor(Color::WHITE),
                ..default()
            })
            .insert(CosmicSource(cosmic_edit_1));

        parent
            .spawn(ButtonBundle {
                background_color: BackgroundColor(Color::WHITE.with_a(0.8)),
                style: Style {
                    width: Val::Percent(50.),
                    height: Val::Percent(100.),
                    ..default()
                },
                ..default()
            })
            .insert(CosmicSource(cosmic_edit_2));
    });

    // Set active editor
    commands.insert_resource(Focus(Some(cosmic_edit_1)));
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

fn main() {
    let font_config = CosmicFontConfig {
        fonts_dir_path: None,
        font_bytes: None,
        load_system_fonts: true,
    };

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CosmicEditPlugin {
            font_config,
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, change_active_editor_ui)
        .run();
}
