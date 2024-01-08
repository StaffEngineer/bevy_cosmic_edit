use std::time::Duration;

use bevy::{
    prelude::*,
    render::render_resource::Extent3d,
    utils::HashMap,
    window::{PrimaryWindow, WindowScaleFactorChanged},
};
use cosmic_text::{Affinity, Edit, Metrics, SwashCache};
use image::{imageops::FilterType, GenericImageView};

use crate::{
    get_text_size, get_x_offset_center, get_y_offset_center, CosmicAttrs, CosmicBackground,
    CosmicEditor, CosmicFontSystem, CosmicMetrics, CosmicMode, CosmicSource, CosmicText,
    CosmicTextPosition, FillColor, Focus, PasswordInput, PlaceholderAttrs, PlaceholderText,
    ReadOnly, XOffset, DEFAULT_SCALE_PLACEHOLDER,
};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum CosmicRenderSet {
    Setup,
    Shaping,
    Sizing,
    Cursor,
    Padding,
    Draw,
}

#[derive(Resource)]
pub(crate) struct SwashCacheState {
    pub swash_cache: SwashCache,
}

#[derive(Resource)]
pub(crate) struct CursorBlinkTimer(pub Timer);

#[derive(Resource)]
pub(crate) struct CursorVisibility(pub bool);

#[derive(Resource, Default)]
pub(crate) struct PasswordValues(pub HashMap<Entity, (String, usize)>);

#[derive(Component)]
pub(crate) struct Placeholder;

#[derive(Component, Default)]
pub struct CosmicPadding(pub Vec2);

#[derive(Component, Default)]
pub struct CosmicWidgetSize(pub Vec2);

pub(crate) fn cosmic_padding(
    mut query: Query<(
        &mut CosmicPadding,
        &CosmicTextPosition,
        &CosmicEditor,
        &CosmicWidgetSize,
    )>,
) {
    for (mut padding, position, editor, size) in query.iter_mut() {
        padding.0 = match position {
            CosmicTextPosition::Center => Vec2::new(
                get_x_offset_center(size.0.x, editor.0.buffer()) as f32,
                get_y_offset_center(size.0.y, editor.0.buffer()) as f32,
            ),
            CosmicTextPosition::TopLeft { padding } => Vec2::new(*padding as f32, *padding as f32),
            CosmicTextPosition::Left { padding } => Vec2::new(
                *padding as f32,
                get_y_offset_center(size.0.y, editor.0.buffer()) as f32,
            ),
        }
    }
}

pub(crate) fn cosmic_widget_size(
    mut query: Query<(&mut CosmicWidgetSize, &Sprite), Changed<Sprite>>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if windows.iter().len() == 0 {
        return;
    }
    let scale = windows.single().scale_factor() as f32;
    for (mut size, sprite) in query.iter_mut() {
        size.0 = sprite.custom_size.unwrap().ceil() * scale;
    }
}

pub(crate) fn cosmic_buffer_size(
    mut query: Query<(
        &mut CosmicEditor,
        &CosmicMode,
        &CosmicWidgetSize,
        &CosmicTextPosition,
    )>,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    for (mut editor, mode, size, position) in query.iter_mut() {
        let padding_x = match position {
            CosmicTextPosition::Center => 0.,
            CosmicTextPosition::TopLeft { padding } => *padding as f32,
            CosmicTextPosition::Left { padding } => *padding as f32,
        };

        let (buffer_width, buffer_height) = match mode {
            CosmicMode::InfiniteLine => (f32::MAX, size.0.y),
            CosmicMode::AutoHeight => (size.0.x - padding_x, (i32::MAX / 2) as f32),
            CosmicMode::Wrap => (size.0.x - padding_x, size.0.y),
        };

        editor
            .0
            .buffer_mut()
            .set_size(&mut font_system.0, buffer_width, buffer_height);
    }
}

pub(crate) fn cosmic_reshape(
    mut query: Query<&mut CosmicEditor>,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    for mut cosmic_editor in query.iter_mut() {
        cosmic_editor.0.shape_as_needed(&mut font_system.0);
    }
}

pub(crate) fn render_texture(
    mut query: Query<(
        &mut CosmicEditor,
        &CosmicAttrs,
        &CosmicBackground,
        &FillColor,
        &Handle<Image>,
        &CosmicWidgetSize,
        &CosmicPadding,
        &XOffset,
    )>,
    mut font_system: ResMut<CosmicFontSystem>,
    mut images: ResMut<Assets<Image>>,
    mut swash_cache_state: ResMut<SwashCacheState>,
) {
    for (mut cosmic_editor, attrs, background_image, fill_color, canvas, size, padding, x_offset) in
        query.iter_mut()
    {
        // TODO: redraw tag component
        if !cosmic_editor.0.buffer().redraw() {
            continue;
        }

        // Draw background
        let mut pixels = vec![0; size.0.x as usize * size.0.y as usize * 4];
        if let Some(bg_image) = background_image.0.clone() {
            if let Some(image) = images.get(&bg_image) {
                let mut dynamic_image = image.clone().try_into_dynamic().unwrap();
                if image.size().x != size.0.x as u32 || image.size().y != size.0.y as u32 {
                    dynamic_image = dynamic_image.resize_to_fill(
                        size.0.x as u32,
                        size.0.y as u32,
                        FilterType::Triangle,
                    );
                }
                for (i, (_, _, rgba)) in dynamic_image.pixels().enumerate() {
                    if let Some(p) = pixels.get_mut(i * 4..(i + 1) * 4) {
                        p[0] = rgba[0];
                        p[1] = rgba[1];
                        p[2] = rgba[2];
                        p[3] = rgba[3];
                    }
                }
            }
        } else {
            let bg = fill_color.0;
            for pixel in pixels.chunks_exact_mut(4) {
                pixel[0] = (bg.r() * 255.) as u8; // Red component
                pixel[1] = (bg.g() * 255.) as u8; // Green component
                pixel[2] = (bg.b() * 255.) as u8; // Blue component
                pixel[3] = (bg.a() * 255.) as u8; // Alpha component
            }
        }

        let font_color = attrs
            .0
            .color_opt
            .unwrap_or(cosmic_text::Color::rgb(0, 0, 0));

        // Draw glyphs
        cosmic_editor.0.draw(
            &mut font_system.0,
            &mut swash_cache_state.swash_cache,
            font_color,
            |x, y, w, h, color| {
                for row in 0..h as i32 {
                    for col in 0..w as i32 {
                        draw_pixel(
                            &mut pixels,
                            size.0.x as i32,
                            size.0.y as i32,
                            x + col + padding.0.x as i32 - x_offset.0.unwrap_or((0., 0.)).0 as i32,
                            y + row + padding.0.y as i32,
                            color,
                        );
                    }
                }
            },
        );

        if let Some(prev_image) = images.get_mut(canvas) {
            prev_image.data.clear();
            prev_image.data.extend_from_slice(pixels.as_slice());
            prev_image.resize(Extent3d {
                width: size.0.x as u32,
                height: size.0.y as u32,
                depth_or_array_layers: 1,
            });
        }

        cosmic_editor.0.buffer_mut().set_redraw(false);
    }
}

pub(crate) fn new_image_from_default(
    mut query: Query<&mut Handle<Image>, Added<CosmicEditor>>,
    mut images: ResMut<Assets<Image>>,
) {
    for mut canvas in query.iter_mut() {
        *canvas = images.add(Image::default());
    }
}

pub(crate) fn set_cursor(
    mut query: Query<(
        &mut XOffset,
        &CosmicMode,
        &CosmicEditor,
        &CosmicWidgetSize,
        &CosmicPadding,
    )>,
) {
    for (mut x_offset, mode, cosmic_editor, size, padding) in query.iter_mut() {
        let mut cursor_x = 0.;
        if mode == &CosmicMode::InfiniteLine {
            if let Some(line) = cosmic_editor.0.buffer().layout_runs().next() {
                for (idx, glyph) in line.glyphs.iter().enumerate() {
                    if cosmic_editor.0.cursor().affinity == Affinity::Before {
                        if idx <= cosmic_editor.0.cursor().index {
                            cursor_x += glyph.w;
                        }
                    } else if idx < cosmic_editor.0.cursor().index {
                        cursor_x += glyph.w;
                    } else {
                        break;
                    }
                }
            }
        }

        if mode == &CosmicMode::InfiniteLine && x_offset.0.is_none() {
            *x_offset = XOffset(Some((0., size.0.x - 2. * padding.0.x)));
        }

        if let Some((x_min, x_max)) = x_offset.0 {
            if cursor_x > x_max {
                let diff = cursor_x - x_max;
                *x_offset = XOffset(Some((x_min + diff, cursor_x)));
            }
            if cursor_x < x_min {
                let diff = x_min - cursor_x;
                *x_offset = XOffset(Some((cursor_x, x_max - diff)));
            }
        }
    }
}

pub(crate) fn auto_height(
    mut query: Query<(
        Entity,
        &mut Sprite,
        &CosmicMode,
        &mut CosmicEditor,
        &CosmicWidgetSize,
    )>,
    mut style_q: Query<(&mut Style, &CosmicSource)>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if windows.iter().len() == 0 {
        return;
    }

    let scale = windows.single().scale_factor() as f32;

    for (entity, mut sprite, mode, mut cosmic_editor, size) in query.iter_mut() {
        if mode == &CosmicMode::AutoHeight {
            let text_size = get_text_size(cosmic_editor.0.buffer());
            let text_height = (text_size.1 + 30.) / scale;
            if text_height > size.0.y / scale {
                let mut new_size = sprite.custom_size.unwrap();
                new_size.y = text_height.ceil();
                // TODO this gets set automatically in UI cases but needs to be done for all other cases.
                // redundant work but easier to just set on all sprites
                sprite.custom_size = Some(new_size);

                cosmic_editor.0.buffer_mut().set_redraw(true);

                // TODO: bad loop nesting
                for (mut style, source) in style_q.iter_mut() {
                    if source.0 != entity {
                        continue;
                    }
                    style.height = Val::Px(text_height.ceil());
                }
            }
        }
    }
}

pub(crate) fn set_size_from_ui(
    mut source_q: Query<&mut Sprite, With<CosmicEditor>>,
    dest_q: Query<(&Node, &CosmicSource)>,
) {
    for (node, source) in dest_q.iter() {
        if let Ok(mut sprite) = source_q.get_mut(source.0) {
            sprite.custom_size = Some(node.size().ceil().max(Vec2::ONE));
        }
    }
}

pub(crate) fn set_size_from_transform(
    mut source_q: Query<&mut Sprite, With<CosmicEditor>>,
    dest_q: Query<(&Transform, &CosmicSource), With<Handle<Mesh>>>,
) {
    for (transform, source) in dest_q.iter() {
        if let Ok(mut sprite) = source_q.get_mut(source.0) {
            sprite.custom_size = Some(transform.scale.truncate() * 64.0);
        }
    }
}

fn draw_pixel(
    buffer: &mut [u8],
    width: i32,
    height: i32,
    x: i32,
    y: i32,
    color: cosmic_text::Color,
) {
    // TODO: perftest this fn against previous iteration
    let a_a = color.a() as u32;
    if a_a == 0 {
        // Do not draw if alpha is zero
        return;
    }

    if y < 0 || y >= height {
        // Skip if y out of bounds
        return;
    }

    if x < 0 || x >= width {
        // Skip if x out of bounds
        return;
    }

    let offset = (y as usize * width as usize + x as usize) * 4;

    let bg = Color::rgba_u8(
        buffer[offset],
        buffer[offset + 1],
        buffer[offset + 2],
        buffer[offset + 3],
    );

    // TODO: if alpha is 100% or bg is empty skip blending

    let fg = Color::rgba_u8(color.r(), color.g(), color.b(), color.a());

    let premul = fg * Vec3::splat(color.a() as f32 / 255.0);

    let out = premul + bg * (1.0 - fg.a());

    buffer[offset + 2] = (out.b() * 255.0) as u8;
    buffer[offset + 1] = (out.g() * 255.0) as u8;
    buffer[offset] = (out.r() * 255.0) as u8;
    buffer[offset + 3] = (out.a() * 255.0) as u8;
}

pub(crate) fn blink_cursor(
    mut visibility: ResMut<CursorVisibility>,
    mut timer: ResMut<CursorBlinkTimer>,
    time: Res<Time>,
    active_editor: ResMut<Focus>,
    mut cosmic_editor_q: Query<&mut CosmicEditor, Without<ReadOnly>>,
) {
    if let Some(e) = active_editor.0 {
        timer.0.tick(time.delta());
        if !timer.0.just_finished() && !active_editor.is_changed() {
            return;
        }
        visibility.0 = !visibility.0;

        // always start cursor visible on focus
        if active_editor.is_changed() {
            visibility.0 = true;
            timer.0.set_elapsed(Duration::ZERO);
        }

        let new_color = if visibility.0 {
            None
        } else {
            Some(cosmic_text::Color::rgba(0, 0, 0, 0))
        };

        if let Ok(mut editor) = cosmic_editor_q.get_mut(e) {
            let editor = &mut editor.0;
            let mut cursor = editor.cursor();
            cursor.color = new_color;
            editor.set_cursor(cursor);
            editor.buffer_mut().set_redraw(true);
        }
    }
}

pub(crate) fn freeze_cursor_blink(
    mut visibility: ResMut<CursorVisibility>,
    mut timer: ResMut<CursorBlinkTimer>,
    active_editor: Res<Focus>,
    keys: Res<Input<KeyCode>>,
    char_evr: EventReader<ReceivedCharacter>,
    mut editor_q: Query<&mut CosmicEditor, Without<ReadOnly>>,
) {
    let inputs = [
        KeyCode::Left,
        KeyCode::Right,
        KeyCode::Up,
        KeyCode::Down,
        KeyCode::Back,
        KeyCode::Return,
    ];
    if !keys.any_pressed(inputs) && char_evr.is_empty() {
        return;
    }

    if let Some(e) = active_editor.0 {
        if let Ok(mut editor) = editor_q.get_mut(e) {
            timer.0.set_elapsed(Duration::ZERO);
            visibility.0 = true;
            let mut cursor = editor.0.cursor();
            cursor.color = None;
            editor.0.set_cursor(cursor);
            editor.0.buffer_mut().set_redraw(true);
        }
    }
}

pub(crate) fn hide_inactive_or_readonly_cursor(
    mut cosmic_editor_q_readonly: Query<&mut CosmicEditor, With<ReadOnly>>,
    mut cosmic_editor_q_editable: Query<(Entity, &mut CosmicEditor), Without<ReadOnly>>,
    active_editor: Res<Focus>,
) {
    for mut editor in &mut cosmic_editor_q_readonly.iter_mut() {
        let mut cursor = editor.0.cursor();
        cursor.color = Some(cosmic_text::Color::rgba(0, 0, 0, 0));
        editor.0.set_cursor(cursor);
        editor.0.buffer_mut().set_redraw(true);
    }

    for (e, mut editor) in &mut cosmic_editor_q_editable.iter_mut() {
        if active_editor.is_none() || e != active_editor.0.unwrap() {
            let mut cursor = editor.0.cursor();
            if cursor.color == Some(cosmic_text::Color::rgba(0, 0, 0, 0)) {
                continue;
            }
            cursor.color = Some(cosmic_text::Color::rgba(0, 0, 0, 0));
            editor.0.set_cursor(cursor);
            editor.0.buffer_mut().set_redraw(true);
        }
    }
}

pub(crate) fn set_initial_scale(
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut cosmic_query: Query<&mut CosmicMetrics, Added<CosmicMetrics>>,
) {
    let scale = window_q.single().scale_factor() as f32;

    for mut metrics in &mut cosmic_query.iter_mut() {
        if metrics.scale_factor != DEFAULT_SCALE_PLACEHOLDER {
            continue;
        }

        metrics.scale_factor = scale;
    }
}

pub(crate) fn on_scale_factor_change(
    mut scale_factor_changed: EventReader<WindowScaleFactorChanged>,
    mut cosmic_query: Query<(&mut CosmicEditor, &CosmicMetrics, &mut XOffset)>,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    if !scale_factor_changed.is_empty() {
        let new_scale_factor = scale_factor_changed.read().last().unwrap().scale_factor as f32;
        for (mut editor, metrics, mut x_offset) in &mut cosmic_query.iter_mut() {
            let font_system = &mut font_system.0;
            let metrics =
                Metrics::new(metrics.font_size, metrics.line_height).scale(new_scale_factor);

            editor.0.buffer_mut().set_metrics(font_system, metrics);
            editor.0.buffer_mut().set_redraw(true);

            *x_offset = XOffset(None);
        }
    }
}

pub(crate) fn swap_target_handle(
    source_q: Query<&Handle<Image>, (Changed<Handle<Image>>, With<CosmicEditor>)>,
    mut dest_q: Query<
        (
            Option<&mut Handle<Image>>,
            Option<&mut UiImage>,
            &CosmicSource,
        ),
        Without<CosmicEditor>,
    >,
) {
    // TODO: do this once
    for (dest_handle_opt, dest_ui_opt, source_entity) in dest_q.iter_mut() {
        if let Ok(source_handle) = source_q.get(source_entity.0) {
            if let Some(mut dest_handle) = dest_handle_opt {
                *dest_handle = source_handle.clone_weak();
            }
            if let Some(mut dest_ui) = dest_ui_opt {
                dest_ui.texture = source_handle.clone_weak();
            }
        }
    }
}

pub(crate) fn hide_password_text(
    mut editor_q: Query<(Entity, &mut CosmicEditor, &CosmicAttrs, &PasswordInput)>,
    mut font_system: ResMut<CosmicFontSystem>,
    mut password_input_states: ResMut<PasswordValues>,
    active_editor: Res<Focus>,
) {
    for (entity, mut cosmic_editor, attrs, password) in editor_q.iter_mut() {
        let text = cosmic_editor.get_text();
        let select_opt = cosmic_editor.0.select_opt();
        let mut cursor = cosmic_editor.0.cursor();

        if !text.is_empty() {
            cosmic_editor.set_text(
                CosmicText::OneStyle(format!("{}", password.0).repeat(text.chars().count())),
                attrs.0.clone(),
                &mut font_system.0,
            );

            // multiply cursor idx and select_opt end point by password char length
            // the actual char length cos '●' is 3x as long as 'a'
            // This operation will need to be undone when resetting.
            //
            // Currently breaks entering multi-byte chars

            let char_len = password.0.len_utf8();

            let select_opt = match select_opt {
                Some(mut select) => {
                    select.index *= char_len;
                    Some(select)
                }
                None => None,
            };

            cursor.index *= char_len;

            cosmic_editor.0.set_select_opt(select_opt);

            // Fixes stuck cursor on password inputs
            if let Some(active) = active_editor.0 {
                if entity != active {
                    cursor.color = Some(cosmic_text::Color::rgba(0, 0, 0, 0));
                }
            }

            cosmic_editor.0.set_cursor(cursor);
        }

        let glyph_idx = match cosmic_editor.0.buffer().lines[0].layout_opt() {
            Some(_) => cosmic_editor.0.buffer().layout_cursor(&cursor).glyph,
            None => 0,
        };

        password_input_states.0.insert(entity, (text, glyph_idx));
    }
}

pub(crate) fn restore_password_text(
    mut editor_q: Query<(Entity, &mut CosmicEditor, &CosmicAttrs, &PasswordInput)>,
    mut font_system: ResMut<CosmicFontSystem>,
    password_input_states: Res<PasswordValues>,
) {
    for (entity, mut cosmic_editor, attrs, password) in editor_q.iter_mut() {
        if let Some((text, _glyph_idx)) = password_input_states.0.get(&entity) {
            if !text.is_empty() {
                let char_len = password.0.len_utf8();

                let mut cursor = cosmic_editor.0.cursor();
                let select_opt = match cosmic_editor.0.select_opt() {
                    Some(mut select) => {
                        select.index /= char_len;
                        Some(select)
                    }
                    None => None,
                };

                cursor.index /= char_len;

                cosmic_editor.set_text(
                    crate::CosmicText::OneStyle(text.clone()),
                    attrs.0.clone(),
                    &mut font_system.0,
                );

                cosmic_editor.0.set_select_opt(select_opt);
                cosmic_editor.0.set_cursor(cursor);
            }
        }
    }
}

pub(crate) fn show_placeholder(
    mut editor_q: Query<(
        Entity,
        &mut CosmicEditor,
        &PlaceholderText,
        &PlaceholderAttrs,
    )>,
    mut font_system: ResMut<CosmicFontSystem>,
    mut commands: Commands,
) {
    for (entity, mut cosmic_editor, placeholder, attrs) in editor_q.iter_mut() {
        if cosmic_editor.get_text().is_empty() {
            cosmic_editor.set_text(placeholder.0.clone(), attrs.0.clone(), &mut font_system.0);

            let mut cursor = cosmic_editor.0.cursor();
            cursor.index = 0;
            cosmic_editor.0.set_cursor(cursor);

            commands.entity(entity).insert(Placeholder);
        } else {
            commands.entity(entity).remove::<Placeholder>();
        }
    }
}

pub(crate) fn restore_placeholder_text(
    mut editor_q: Query<(&mut CosmicEditor, &CosmicAttrs), With<Placeholder>>,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    for (mut cosmic_editor, attrs) in editor_q.iter_mut() {
        cosmic_editor.set_text(
            CosmicText::OneStyle("".into()),
            attrs.0.clone(),
            &mut font_system.0,
        );
    }
}
