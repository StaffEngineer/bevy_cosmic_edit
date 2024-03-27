use crate::*;
use bevy::{prelude::*, window::PrimaryWindow};

#[derive(Component, Deref, DerefMut)]
pub struct CosmicBuffer(pub Buffer);

impl Default for CosmicBuffer {
    fn default() -> Self {
        CosmicBuffer(Buffer::new_empty(Metrics::new(20., 20.)))
    }
}

impl<'s, 'r> CosmicBuffer {
    pub fn new(font_system: &mut FontSystem, metrics: Metrics) -> Self {
        Self(Buffer::new(font_system, metrics))
    }

    // Das a lotta boilerplate just to hide the shaping argument
    pub fn with_text(
        mut self,
        font_system: &mut FontSystem,
        text: &'s str,
        attrs: Attrs<'r>,
    ) -> Self {
        self.0.set_text(font_system, text, attrs, Shaping::Advanced);
        self
    }

    pub fn with_rich_text<I>(
        mut self,
        font_system: &mut FontSystem,
        spans: I,
        attrs: Attrs<'r>,
    ) -> Self
    where
        I: IntoIterator<Item = (&'s str, Attrs<'r>)>,
    {
        self.0
            .set_rich_text(font_system, spans, attrs, Shaping::Advanced);
        self
    }

    pub fn set_text(
        &mut self,
        font_system: &mut FontSystem,
        text: &'s str,
        attrs: Attrs<'r>,
    ) -> &mut Self {
        self.0.set_text(font_system, text, attrs, Shaping::Advanced);
        self.set_redraw(true);
        self
    }

    pub fn set_rich_text<I>(
        &mut self,
        font_system: &mut FontSystem,
        spans: I,
        attrs: Attrs<'r>,
    ) -> &mut Self
    where
        I: IntoIterator<Item = (&'s str, Attrs<'r>)>,
    {
        self.0
            .set_rich_text(font_system, spans, attrs, Shaping::Advanced);
        self.set_redraw(true);
        self
    }

    /// Retrieves the cosmic text content from a buffer.
    ///
    /// # Arguments
    ///
    /// * none, takes the rust magic ref to self
    ///
    /// # Returns
    ///
    /// A `String` containing the cosmic text content.
    pub fn get_text(&self) -> String {
        let mut text = String::new();
        let line_count = self.lines.len();

        for (i, line) in self.lines.iter().enumerate() {
            text.push_str(line.text());

            if i < line_count - 1 {
                text.push('\n');
            }
        }

        text
    }
    /// Returns texts from a MultiStyle buffer
    pub fn get_text_spans(&self, default_attrs: AttrsOwned) -> Vec<Vec<(String, AttrsOwned)>> {
        // TODO: untested!

        let buffer = self;

        let mut spans = Vec::new();
        for line in buffer.lines.iter() {
            let mut line_spans = Vec::new();
            let line_text = line.text();
            let line_attrs = line.attrs_list();
            if line_attrs.spans().is_empty() {
                line_spans.push((line_text.to_string(), default_attrs.clone()));
            } else {
                let mut current_pos = 0;
                for span in line_attrs.spans() {
                    let span_range = span.0;
                    let span_attrs = span.1.clone();
                    let start_index = span_range.start;
                    let end_index = span_range.end;
                    if start_index > current_pos {
                        // Add the text between the current position and the start of the span
                        let non_span_text = line_text[current_pos..start_index].to_string();
                        line_spans.push((non_span_text, default_attrs.clone()));
                    }
                    let span_text = line_text[start_index..end_index].to_string();
                    line_spans.push((span_text.clone(), span_attrs));
                    current_pos = end_index;
                }
                if current_pos < line_text.len() {
                    // Add the remaining text after the last span
                    let remaining_text = line_text[current_pos..].to_string();
                    line_spans.push((remaining_text, default_attrs.clone()));
                }
            }
            spans.push(line_spans);
        }
        spans
    }
}

pub fn add_font_system(
    mut font_system: ResMut<CosmicFontSystem>,
    mut q: Query<&mut CosmicBuffer, Added<CosmicBuffer>>,
) {
    for mut b in q.iter_mut() {
        if !b.lines.is_empty() {
            continue;
        }
        b.0.set_text(&mut font_system, "", Attrs::new(), Shaping::Advanced);
        b.set_redraw(true);
    }
}

pub fn set_initial_scale(
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut cosmic_query: Query<&mut CosmicBuffer, Added<CosmicBuffer>>,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    let w_scale = window_q.single().scale_factor();

    for mut b in &mut cosmic_query.iter_mut() {
        let m = b.metrics().scale(w_scale);
        b.set_metrics(&mut font_system, m);
    }
}

pub fn set_redraw(mut q: Query<&mut CosmicBuffer, Added<CosmicBuffer>>) {
    for mut b in q.iter_mut() {
        b.set_redraw(true);
    }
}

pub fn set_editor_redraw(mut q: Query<&mut CosmicEditor, Added<CosmicEditor>>) {
    for mut b in q.iter_mut() {
        b.set_redraw(true);
    }
}

pub(crate) fn swap_target_handle(
    source_q: Query<&Handle<Image>, With<CosmicBuffer>>,
    mut dest_q: Query<
        (
            Option<&mut Handle<Image>>,
            Option<&mut UiImage>,
            &CosmicSource,
        ),
        Without<CosmicBuffer>,
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

// TODO put this on impl CosmicBuffer

pub fn get_text_size(buffer: &Buffer) -> (f32, f32) {
    if buffer.layout_runs().count() == 0 {
        return (0., buffer.metrics().line_height);
    }
    let width = buffer
        .layout_runs()
        .map(|run| run.line_w)
        .reduce(f32::max)
        .unwrap();
    let height = buffer.layout_runs().count() as f32 * buffer.metrics().line_height;
    (width, height)
}

pub fn get_y_offset_center(widget_height: f32, buffer: &Buffer) -> i32 {
    let (_, text_height) = get_text_size(buffer);
    ((widget_height - text_height) / 2.0) as i32
}

pub fn get_x_offset_center(widget_width: f32, buffer: &Buffer) -> i32 {
    let (text_width, _) = get_text_size(buffer);
    ((widget_width - text_width) / 2.0) as i32
}
