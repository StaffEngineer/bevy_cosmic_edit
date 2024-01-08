use bevy::prelude::*;
use bevy_cosmic_edit::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CosmicEditPlugin {
            change_cursor: CursorConfig::None,
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, set_texture)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let editor = commands
        .spawn((
            CosmicEditBundle {
                text_setter: CosmicText::OneStyle("It is a period of civil wars in the galaxy. A brave alliance of underground freedom fighters has challenged the tyranny and oppression of the awesome GALACTIC EMPIRE.
Striking from a fortress hidden among the billion stars of the galaxy, rebel spaceships have won their first victory in a battle with the powerful Imperial Starfleet. The EMPIRE fears that another defeat could bring a thousand more solar systems into the rebellion, and Imperial control over the galaxy would be lost forever.

To crush the rebellion once and for all, the EMPIRE is constructing a sinister new battle station. Powerful enough to destroy an entire planet, its completion spells certain doom for the champions of freedom.".into()),
                fill_color: FillColor(Color::NONE),
                attrs: CosmicAttrs(AttrsOwned::new(Attrs::new().color(CosmicColor::rgba(255,200,0,255)))),
                metrics: CosmicMetrics {
                    font_size: 20.0,
                    line_height: 20.0,
                    ..default()
                },
                ..default()
            },
        ))
        .id();

    commands.insert_resource(Focus(Some(editor)));

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 9000.0,
            range: 100.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(8.0, 16.0, 8.0),
        ..default()
    });

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Plane::from_size(1.0).into()),
            material: materials.add(Color::RED.into()),
            transform: Transform::from_scale(Vec3::ONE * 10.0),
            ..default()
        },
        CosmicSource(editor),
    ));

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 6., 12.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        ..default()
    });
}

fn set_texture(
    plane_q: Query<&Handle<StandardMaterial>>,
    canvas_q: Query<&Handle<Image>, With<CosmicEditor>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for handle in plane_q.iter() {
        if let Some(mut material) = materials.get_mut(handle) {
            if let Ok(canvas) = canvas_q.get_single() {
                material.base_color = Color::WHITE;
                material.base_color_texture = Some(canvas.clone_weak());
            }
        }
    }
}
