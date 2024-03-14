use bevy::{prelude::*, render::render_resource::Extent3d};
use cosmic_text::{Color, Edit, SwashCache};
use image::{imageops::FilterType, GenericImageView};

use crate::{
    layout::{CosmicPadding, CosmicWidgetSize},
    CosmicAttrs, CosmicBackground, CosmicBuffer, CosmicEditor, CosmicFontSystem, FillColor,
    XOffset,
};

#[derive(Resource)]
pub(crate) struct SwashCacheState {
    pub swash_cache: SwashCache,
}

pub fn blink_cursor(mut q: Query<&mut CosmicEditor>, time: Res<Time>) {
    for mut e in q.iter_mut() {
        e.cursor_timer.tick(time.delta());
        if e.cursor_timer.just_finished() {
            e.cursor_visible = !e.cursor_visible;
            e.set_redraw(true);
        }
    }
}

fn draw_pixel(buffer: &mut [u8], width: i32, height: i32, x: i32, y: i32, color: Color) {
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

    let bg = bevy::prelude::Color::rgba_u8(
        buffer[offset],
        buffer[offset + 1],
        buffer[offset + 2],
        buffer[offset + 3],
    );

    // TODO: if alpha is 100% or bg is empty skip blending

    let fg = bevy::prelude::Color::rgba_u8(color.r(), color.g(), color.b(), color.a());

    let premul = fg * Vec3::splat(color.a() as f32 / 255.0);

    let out = premul + bg * (1.0 - fg.a());

    buffer[offset + 2] = (out.b() * 255.0) as u8;
    buffer[offset + 1] = (out.g() * 255.0) as u8;
    buffer[offset] = (out.r() * 255.0) as u8;
    buffer[offset + 3] = (out.a() * 255.0) as u8;
}

pub(crate) fn render_texture(
    mut query: Query<(
        Option<&mut CosmicEditor>,
        &mut CosmicBuffer,
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
    for (
        editor,
        mut buffer,
        attrs,
        background_image,
        fill_color,
        canvas,
        size,
        padding,
        x_offset,
    ) in query.iter_mut()
    {
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

        let draw_closure = |x, y, w, h, color| {
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
        };

        // Draw glyphs
        if let Some(mut editor) = editor {
            if !editor.redraw() {
                continue;
            }
            let cursor_opacity = if editor.cursor_visible { 255 } else { 0 };
            editor.draw(
                &mut font_system.0,
                &mut swash_cache_state.swash_cache,
                font_color,
                Color::rgba(0, 0, 255, cursor_opacity),
                Color::rgba(255, 0, 255, 255),
                draw_closure,
            );
            editor.set_redraw(false);
        } else {
            if !buffer.redraw() {
                continue;
            }
            buffer.draw(
                &mut font_system.0,
                &mut swash_cache_state.swash_cache,
                font_color,
                draw_closure,
            );
            buffer.set_redraw(false);
        }

        if let Some(prev_image) = images.get_mut(canvas) {
            prev_image.data.clear();
            prev_image.data.extend_from_slice(pixels.as_slice());
            prev_image.resize(Extent3d {
                width: size.0.x as u32,
                height: size.0.y as u32,
                depth_or_array_layers: 1,
            });
        }
    }
}
