use bevy::prelude::*;
use bevy_cosmic_edit::*;

#[derive(Resource)]
struct TextChangeTimer(pub Timer);

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    let attrs =
        AttrsOwned::new(Attrs::new().color(bevy_color_to_cosmic(Color::rgb(0.27, 0.27, 0.27))));

    let editor = commands
        .spawn(CosmicEditBundle {
            text_position: CosmicTextPosition::default(),
            fill_color: FillColor::default(),
            background_image: CosmicBackground::default(),
            attrs: CosmicAttrs(attrs.clone()),
            max_chars: CosmicMaxChars(15),
            max_lines: CosmicMaxLines(1),
            text_setter: CosmicText::OneStyle("BANANA IS THE CODEWORD!".into()),
            mode: CosmicMode::Wrap,
            // CosmicEdit draws to this spritebundle
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    // when using another target like a UI element, this is overridden
                    custom_size: Some(Vec2::ONE * 128.0),
                    ..default()
                },
                // this is the default behaviour for targeting UI elements.
                // If wanting a sprite, define your own SpriteBundle and
                // leave the visibility on. See examples/basic_sprite.rs
                visibility: Visibility::Hidden,
                ..default()
            },
            // Computed fields
            padding: Default::default(),
            widget_size: Default::default(),
        })
        .insert(CosmicEditPlaceholderBundle {
            text_setter: PlaceholderText(CosmicText::OneStyle("Placeholder".into())),
            attrs: PlaceholderAttrs(AttrsOwned::new(
                Attrs::new().color(CosmicColor::rgb(88, 88, 88)),
            )),
        })
        .id();

    commands
        .spawn(ButtonBundle {
            border_color: Color::LIME_GREEN.into(),
            style: Style {
                // Size and position of text box
                border: UiRect::all(Val::Px(4.)),
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

    commands.insert_resource(Focus(Some(editor)));

    commands.insert_resource(TextChangeTimer(Timer::from_seconds(
        1.,
        TimerMode::Repeating,
    )));
}

pub fn bevy_color_to_cosmic(color: bevy::prelude::Color) -> CosmicColor {
    cosmic_text::Color::rgba(
        (color.r() * 255.) as u8,
        (color.g() * 255.) as u8,
        (color.b() * 255.) as u8,
        (color.a() * 255.) as u8,
    )
}

// Test for update_buffer_text
fn text_swapper(
    mut timer: ResMut<TextChangeTimer>,
    time: Res<Time>,
    mut cosmic_q: Query<&mut CosmicText>,
    mut count: Local<usize>,
    editor_q: Query<&CosmicEditor>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    *count += 1;
    for mut text in cosmic_q.iter_mut() {
        text.set_if_neq(CosmicText::OneStyle(format!("TIMER {}", *count)));
    }

    let editor = editor_q.single();
    println!("X OFFSET: {}", get_x_offset_center(50., editor.0.buffer()));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CosmicEditPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, text_swapper)
        .run();
}
