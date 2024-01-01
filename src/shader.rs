use std::{hash::Hash, marker::PhantomData};

use bevy::{
    prelude::*,
    sprite::{Material2d, Material2dPlugin},
};

use crate::{CosmicEditor, CosmicRenderSet, CosmicSource};

// TODO: document bindings here. Always expect color_texture bound to same place, show example.
// Allows shared shaders. Though API is like water so might be a job for later.
pub trait CosmicMaterial2d {
    fn color_texture(&self) -> &Handle<Image>;
    fn set_color_texture(&mut self, texture: &Handle<Image>) -> &mut Self;
}

pub struct CosmicMaterial2dPlugin<M: CosmicMaterial2d + Material2d>(PhantomData<M>);

impl<M: CosmicMaterial2d + Material2d> Default for CosmicMaterial2dPlugin<M> {
    fn default() -> Self {
        Self(Default::default())
    }
}

// TODO: docs here, explain passthrough to Material2dPlugin
impl<M: Material2d + CosmicMaterial2d> Plugin for CosmicMaterial2dPlugin<M>
where
    M::Data: PartialEq + Eq + Hash + Clone,
{
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<M>::default())
            .add_systems(
                PostUpdate,
                add_cosmic_material::<M>.after(CosmicRenderSet::Draw),
            );
    }
}

pub fn add_cosmic_material<M: Material2d + CosmicMaterial2d>(
    source_q: Query<&Handle<Image>, With<CosmicEditor>>,
    dest_q: Query<(&Handle<M>, &CosmicSource), Without<CosmicEditor>>,
    mut materials: ResMut<Assets<M>>,
) {
    // TODO: do this once
    for (dest_handle, source_entity) in dest_q.iter() {
        if let Ok(source_handle) = source_q.get(source_entity.0) {
            if let Some(dest_material) = materials.get_mut(dest_handle) {
                if dest_material.color_texture() == source_handle {
                    // TODO: instead of all this looping find a way to check if handle is changed
                    // earlier in the chain
                    return;
                }
                dest_material.set_color_texture(source_handle);
            }
        }
    }
}
