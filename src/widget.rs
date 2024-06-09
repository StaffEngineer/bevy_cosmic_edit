use std::sync::Arc;

use crate::*;
use bevy::{prelude::*, window::PrimaryWindow};
use cosmic_text::Affinity;

/// System set for cosmic text layout systems. Runs in [`PostUpdate`]
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct WidgetSet;

pub(crate) struct WidgetPlugin;

impl Plugin for WidgetPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, reshape.in_set(WidgetSet).after(InputSet))
            .add_systems(
                PostUpdate,
                (
                    (new_image_from_default, set_sprite_size_from_ui),
                    set_widget_size,
                    set_buffer_size,
                    set_padding,
                    set_x_offset,
                )
                    .chain()
                    .in_set(WidgetSet)
                    .after(TransformSystem::TransformPropagate),
            );
    }
}

/// Wrapper for a [`Vec2`] describing the horizontal and vertical padding of a widget.
/// This is set programatically, not for user modification.
/// To set a widget's padding, use [`CosmicTextAlign`]
#[derive(Component, Default, Deref, DerefMut, Debug)]
pub struct CosmicPadding(pub Vec2);

/// Wrapper for a [`Vec2`] describing the horizontal and vertical size of a widget.
/// This is set programatically, not for user modification.
/// To set a widget's size, use either it's [`Sprite`] dimensions or modify the target UI element's
/// size.
#[derive(Component, Default, Deref, DerefMut)]
pub struct CosmicWidgetSize(pub Vec2);

/// Reshapes text in a [`CosmicEditor`]
fn reshape(mut query: Query<&mut CosmicEditor>, mut font_system: ResMut<CosmicFontSystem>) {
    for mut cosmic_editor in query.iter_mut() {
        cosmic_editor.shape_as_needed(&mut font_system.0, false);
    }
}

/// Programatically sets the [`CosmicPadding`] of a widget based on it's [`CosmicTextAlign`]
fn set_padding(
    mut query: Query<
        (
            &mut CosmicPadding,
            &CosmicTextAlign,
            &CosmicBuffer,
            &CosmicWidgetSize,
            Option<&CosmicEditor>,
        ),
        Or<(
            With<CosmicEditor>,
            Changed<CosmicTextAlign>,
            Changed<CosmicBuffer>,
            Changed<CosmicWidgetSize>,
        )>,
    >,
) {
    for (mut padding, position, buffer, size, editor_opt) in query.iter_mut() {
        let mut redraw = buffer.redraw();
        let mut buffer = buffer.0.as_ref().clone();

        if let Some(editor) = editor_opt {
            redraw = editor.redraw();
            buffer = editor.with_buffer(|b| b.clone());
        }

        if !redraw {
            continue;
        }

        padding.0 = match position {
            CosmicTextAlign::Center { padding: _ } => Vec2::new(
                get_x_offset_center(size.0.x, &buffer) as f32,
                get_y_offset_center(size.0.y, &buffer) as f32,
            ),
            CosmicTextAlign::TopLeft { padding } => Vec2::new(*padding as f32, *padding as f32),
            CosmicTextAlign::Left { padding } => Vec2::new(
                *padding as f32,
                get_y_offset_center(size.0.y, &buffer) as f32,
            ),
        }
    }
}

/// Programatically sets the [`CosmicWidgetSize`] of a widget based on it's [`Sprite`] properties
fn set_widget_size(
    mut query: Query<(&mut CosmicWidgetSize, &Sprite), Changed<Sprite>>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if windows.iter().len() == 0 {
        return;
    }
    // TODO: early return if sprite size is unchanged
    let scale = windows.single().scale_factor();
    for (mut size, sprite) in query.iter_mut() {
        size.0 = sprite.custom_size.unwrap().ceil() * scale;
    }
}

/// Sets the internal [`Buffer`]'s size according to the [`CosmicWidgetSize`] and [`CosmicTextAlign`]
fn set_buffer_size(
    mut query: Query<
        (
            &mut CosmicBuffer,
            &CosmicWrap,
            &CosmicWidgetSize,
            &CosmicTextAlign,
        ),
        Or<(
            Changed<CosmicWrap>,
            Changed<CosmicWidgetSize>,
            Changed<CosmicTextAlign>,
        )>,
    >,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    for (mut buffer, mode, size, position) in query.iter_mut() {
        let padding_x = match position {
            CosmicTextAlign::Center { padding: _ } => 0.,
            CosmicTextAlign::TopLeft { padding } => *padding as f32,
            CosmicTextAlign::Left { padding } => *padding as f32,
        };

        let (buffer_width, buffer_height) = match mode {
            CosmicWrap::InfiniteLine => (f32::MAX, size.0.y),
            CosmicWrap::Wrap => (size.0.x - padding_x, size.0.y),
        };

        let buffer = Arc::make_mut(&mut buffer);

        buffer.set_size(&mut font_system.0, buffer_width, buffer_height);
    }
}

/// Instantiates a new image for a [`CosmicBuffer`]
fn new_image_from_default(
    mut query: Query<&mut Handle<Image>, Added<CosmicBuffer>>,
    mut images: ResMut<Assets<Image>>,
) {
    for mut canvas in query.iter_mut() {
        *canvas = images.add(Image::default());
    }
}

fn set_x_offset(
    mut query: Query<(
        &mut XOffset,
        &CosmicWrap,
        &CosmicEditor,
        &CosmicWidgetSize,
        &CosmicTextAlign,
    )>,
) {
    for (mut x_offset, mode, editor, size, position) in query.iter_mut() {
        if mode != &CosmicWrap::InfiniteLine {
            return;
        }

        let mut cursor_x = 0.;
        let cursor = editor.cursor();

        if let Some(line) = editor.with_buffer(|b| b.clone()).layout_runs().next() {
            for (idx, glyph) in line.glyphs.iter().enumerate() {
                if cursor.affinity == Affinity::Before {
                    if idx <= cursor.index {
                        cursor_x += glyph.w;
                    } else {
                        break;
                    }
                } else if idx < cursor.index {
                    cursor_x += glyph.w;
                } else {
                    break;
                }
            }
        }

        let padding_x = match position {
            CosmicTextAlign::Center { padding } => *padding as f32,
            CosmicTextAlign::TopLeft { padding } => *padding as f32,
            CosmicTextAlign::Left { padding } => *padding as f32,
        };

        if x_offset.width == 0. {
            x_offset.width = size.x - padding_x * 2.;
        }

        let right = x_offset.width + x_offset.left;

        if cursor_x > right {
            let diff = cursor_x - right;
            x_offset.left += diff;
        }
        if cursor_x < x_offset.left {
            let diff = x_offset.left - cursor_x;
            x_offset.left -= diff;
        }
    }
}

fn set_sprite_size_from_ui(
    mut source_q: Query<&mut Sprite, With<CosmicBuffer>>,
    dest_q: Query<(&Node, &CosmicSource), Changed<Node>>,
) {
    for (node, source) in dest_q.iter() {
        if let Ok(mut sprite) = source_q.get_mut(source.0) {
            sprite.custom_size = Some(node.size().ceil().max(Vec2::ONE));
        }
    }
}
