use bevy::{
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, MaterialMesh2dBundle},
    window::PrimaryWindow,
};
use bevy_cosmic_edit::*;

// Define material
#[derive(AsBindGroup, TypeUuid, TypePath, Debug, Clone, Default)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct CustomMaterial {
    #[uniform(0)]
    color: Color,
    #[texture(1)]
    #[sampler(2)]
    color_texture: Handle<Image>,
}

impl Material2d for CustomMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/test.wgsl".into()
    }
}

// implement required cosmic bits
impl CosmicMaterial2d for CustomMaterial {
    fn color_texture(&self) -> &Handle<Image> {
        // if using different bindings change the names in the impl too
        &self.color_texture
    }
    fn set_color_texture(&mut self, texture: &Handle<Image>) -> &mut Self {
        self.color_texture = texture.clone_weak();
        self
    }
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<CustomMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());

    // Shaders need to be in loaded to be accessed
    commands.spawn(asset_server.load("shaders/test_export.wgsl") as Handle<Shader>);

    // Sprite editor
    let editor = commands
        .spawn((CosmicEditBundle {
            sprite_bundle: SpriteBundle {
                // Sets size of text box
                sprite: Sprite {
                    custom_size: Some(Vec2::new(300., 100.)),
                    ..default()
                },
                // Position of text box
                transform: Transform::from_xyz(0., 200., 0.),
                ..default()
            },
            ..default()
        },))
        .id();

    // TODO: click fns
    //
    // TODO: set size of sprite from mesh automagically
    commands
        .spawn(MaterialMesh2dBundle {
            mesh: meshes
                .add(Mesh::from(shape::Quad::new(Vec2::new(300., 100.))))
                .into(),
            material: materials.add(CustomMaterial::default()),
            ..default()
        })
        .insert(CosmicSource(editor));

    commands.insert_resource(Focus(Some(editor)));
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

fn change_active_editor_sprite(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<Input<MouseButton>>,
    mut cosmic_edit_query: Query<
        (&mut Sprite, &GlobalTransform, &Visibility, Entity),
        (With<CosmicEditor>, Without<ReadOnly>),
    >,
    camera_q: Query<(&Camera, &GlobalTransform)>,
) {
    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();
    if buttons.just_pressed(MouseButton::Left) {
        for (sprite, node_transform, visibility, entity) in &mut cosmic_edit_query.iter_mut() {
            if visibility == Visibility::Hidden {
                continue;
            }
            let size = sprite.custom_size.unwrap_or(Vec2::ONE);
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
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CosmicEditPlugin {
            change_cursor: CursorConfig::Default,
            ..default()
        })
        // This works like a passthrough, passes the material to Material2dPlugin
        // once it assigns required fns
        .add_plugins(CosmicMaterial2dPlugin::<CustomMaterial>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, change_active_editor_ui)
        .add_systems(Update, change_active_editor_sprite)
        // TODO: change_active_editor_mesh
        .run();
}
