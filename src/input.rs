#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use std::time::Duration;

#[cfg(target_arch = "wasm32")]
use bevy::tasks::AsyncComputeTaskPool;

use bevy::{
    input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    prelude::*,
    window::PrimaryWindow,
};
use cosmic_text::{Action, AttrsList, BufferLine, Cursor, Edit, Shaping};

#[cfg(target_arch = "wasm32")]
use js_sys::Promise;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;

use crate::{
    get_node_cursor_pos, get_timestamp, get_x_offset_center, get_y_offset_center,
    save_edit_history, CosmicAttrs, CosmicEditHistory, CosmicEditor, CosmicFontSystem,
    CosmicMaxChars, CosmicMaxLines, CosmicSource, CosmicTextChanged, CosmicTextPosition, Focus,
    PasswordInput, ReadOnly, XOffset,
};

#[derive(Resource)]
pub struct ClickTimer(pub Timer);

// TODO: hide this behind #cfg wasm, depends on wasm having own copy/paste fn
#[allow(dead_code)]
pub struct WasmPaste {
    text: String,
    entity: Entity,
}

#[derive(Resource)]
pub struct WasmPasteAsyncChannel {
    pub tx: crossbeam_channel::Sender<WasmPaste>,
    pub rx: crossbeam_channel::Receiver<WasmPaste>,
}

pub(crate) fn input_mouse(
    windows: Query<&Window, With<PrimaryWindow>>,
    active_editor: Res<Focus>,
    keys: Res<Input<KeyCode>>,
    buttons: Res<Input<MouseButton>>,
    mut editor_q: Query<(
        &mut CosmicEditor,
        &GlobalTransform,
        &CosmicTextPosition,
        Entity,
        &XOffset,
        &mut Sprite,
    )>,
    node_q: Query<(&Node, &GlobalTransform, &CosmicSource)>,
    mut font_system: ResMut<CosmicFontSystem>,
    mut scroll_evr: EventReader<MouseWheel>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut click_timer: ResMut<ClickTimer>,
    mut click_count: Local<usize>,
    time: Res<Time>,
    evr_mouse_motion: EventReader<MouseMotion>,
) {
    click_timer.0.tick(time.delta());

    if active_editor.0.is_none() {
        return;
    }

    if click_timer.0.finished() || !evr_mouse_motion.is_empty() {
        *click_count = 0;
    }

    if buttons.just_pressed(MouseButton::Left) {
        click_timer.0.reset();
        *click_count += 1;
    }

    if *click_count > 3 {
        *click_count = 0;
    }

    let primary_window = windows.single();
    let scale_factor = primary_window.scale_factor() as f32;
    let (camera, camera_transform) = camera_q.iter().find(|(c, _)| c.is_active).unwrap();

    for (mut editor, sprite_transform, text_position, entity, x_offset, sprite) in
        &mut editor_q.iter_mut()
    {
        if active_editor.0 != Some(entity) {
            continue;
        }

        let mut is_ui_node = false;
        let mut transform = sprite_transform;
        let (mut width, mut height) =
            (sprite.custom_size.unwrap().x, sprite.custom_size.unwrap().y);

        // TODO: this is bad loop nesting, rethink system with relationships in mind
        for (node, node_transform, source) in node_q.iter() {
            if source.0 != entity {
                continue;
            }
            is_ui_node = true;
            transform = node_transform;
            width = node.size().x;
            height = node.size().y;
        }

        let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

        // if shift key is pressed
        let already_has_selection = editor.0.select_opt().is_some();
        if shift && !already_has_selection {
            let cursor = editor.0.cursor();
            editor.0.set_select_opt(Some(cursor));
        }

        let (padding_x, padding_y) = match text_position {
            CosmicTextPosition::Center => (
                get_x_offset_center(width * scale_factor, editor.0.buffer()),
                get_y_offset_center(height * scale_factor, editor.0.buffer()),
            ),
            CosmicTextPosition::TopLeft { padding } => (*padding, *padding),
            CosmicTextPosition::Left { padding } => (
                *padding,
                get_y_offset_center(height * scale_factor, editor.0.buffer()),
            ),
        };
        let point = |node_cursor_pos: (f32, f32)| {
            (
                (node_cursor_pos.0 * scale_factor) as i32 - padding_x,
                (node_cursor_pos.1 * scale_factor) as i32 - padding_y,
            )
        };

        if buttons.just_pressed(MouseButton::Left) {
            if let Some(node_cursor_pos) = get_node_cursor_pos(
                primary_window,
                transform,
                (width, height),
                is_ui_node,
                camera,
                camera_transform,
            ) {
                let (mut x, y) = point(node_cursor_pos);
                x += x_offset.0.unwrap_or((0., 0.)).0 as i32;
                if shift {
                    editor.0.action(&mut font_system.0, Action::Drag { x, y });
                } else {
                    match *click_count {
                        1 => {
                            editor.0.action(&mut font_system.0, Action::Click { x, y });
                        }
                        2 => {
                            // select word
                            editor.0.action(&mut font_system.0, Action::LeftWord);
                            let cursor = editor.0.cursor();
                            editor.0.set_select_opt(Some(cursor));
                            editor.0.action(&mut font_system.0, Action::RightWord);
                        }
                        3 => {
                            // select paragraph
                            editor.0.action(&mut font_system.0, Action::ParagraphStart);
                            let cursor = editor.0.cursor();
                            editor.0.set_select_opt(Some(cursor));
                            editor.0.action(&mut font_system.0, Action::ParagraphEnd);
                        }
                        _ => {}
                    }
                }
            }
            return;
        }

        if buttons.pressed(MouseButton::Left) && *click_count == 0 {
            if let Some(node_cursor_pos) = get_node_cursor_pos(
                primary_window,
                transform,
                (width, height),
                is_ui_node,
                camera,
                camera_transform,
            ) {
                let (mut x, y) = point(node_cursor_pos);
                x += x_offset.0.unwrap_or((0., 0.)).0 as i32;
                if active_editor.is_changed() && !shift {
                    editor.0.action(&mut font_system.0, Action::Click { x, y });
                } else {
                    editor.0.action(&mut font_system.0, Action::Drag { x, y });
                }
            }
            return;
        }

        for ev in scroll_evr.read() {
            match ev.unit {
                MouseScrollUnit::Line => {
                    editor.0.action(
                        &mut font_system.0,
                        Action::Scroll {
                            lines: -ev.y as i32,
                        },
                    );
                }
                MouseScrollUnit::Pixel => {
                    let line_height = editor
                        .0
                        .buffer()
                        .line_heights()
                        .iter()
                        .next()
                        .unwrap_or(&18.)
                        .clone();
                    editor.0.action(
                        &mut font_system.0,
                        Action::Scroll {
                            lines: -(ev.y / line_height) as i32,
                        },
                    );
                }
            }
        }
    }
}

// TODO: split copy/paste into own fn, separate fn for wasm
pub(crate) fn input_kb(
    active_editor: Res<Focus>,
    keys: Res<Input<KeyCode>>,
    mut char_evr: EventReader<ReceivedCharacter>,
    mut cosmic_edit_query: Query<(
        &mut CosmicEditor,
        &mut CosmicEditHistory,
        &CosmicAttrs,
        &CosmicMaxLines,
        &CosmicMaxChars,
        Entity,
        Option<&ReadOnly>,
        Option<&PasswordInput>,
    )>,
    mut evw_changed: EventWriter<CosmicTextChanged>,
    mut font_system: ResMut<CosmicFontSystem>,
    mut is_deleting: Local<bool>,
    mut edits_duration: Local<Option<Duration>>,
    _channel: Option<Res<WasmPasteAsyncChannel>>,
) {
    for (
        mut editor,
        mut edit_history,
        attrs,
        max_lines,
        max_chars,
        entity,
        readonly_opt,
        password_opt,
    ) in &mut cosmic_edit_query.iter_mut()
    {
        if active_editor.0 != Some(entity) {
            continue;
        }

        let readonly = readonly_opt.is_some();

        let attrs = &attrs.0;

        let now_ms = get_timestamp();

        let command = keypress_command(&keys);

        #[cfg(target_arch = "wasm32")]
        let command = if web_sys::window()
            .unwrap()
            .navigator()
            .user_agent()
            .unwrap_or("NoUA".into())
            .contains("Macintosh")
        {
            keys.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight])
        } else {
            command
        };

        let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

        #[cfg(target_os = "macos")]
        let option = keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]);

        // if shift key is pressed
        let already_has_selection = editor.0.select_opt().is_some();
        if shift && !already_has_selection {
            let cursor = editor.0.cursor();
            editor.0.set_select_opt(Some(cursor));
        }

        #[cfg(target_os = "macos")]
        let should_jump = command && option;
        #[cfg(not(target_os = "macos"))]
        let should_jump = command;

        if should_jump && keys.just_pressed(KeyCode::Left) {
            editor.0.action(&mut font_system.0, Action::PreviousWord);
            if !shift {
                editor.0.set_select_opt(None);
            }
            return;
        }
        if should_jump && keys.just_pressed(KeyCode::Right) {
            editor.0.action(&mut font_system.0, Action::NextWord);
            if !shift {
                editor.0.set_select_opt(None);
            }
            return;
        }
        if should_jump && keys.just_pressed(KeyCode::Home) {
            editor.0.action(&mut font_system.0, Action::BufferStart);
            // there's a bug with cosmic text where it doesn't update the visual cursor for this action
            // TODO: fix upstream
            editor.0.buffer_mut().set_redraw(true);
            if !shift {
                editor.0.set_select_opt(None);
            }
            return;
        }
        if should_jump && keys.just_pressed(KeyCode::End) {
            editor.0.action(&mut font_system.0, Action::BufferEnd);
            // there's a bug with cosmic text where it doesn't update the visual cursor for this action
            // TODO: fix upstream
            editor.0.buffer_mut().set_redraw(true);
            if !shift {
                editor.0.set_select_opt(None);
            }
            return;
        }

        if keys.just_pressed(KeyCode::Left) {
            editor.0.action(&mut font_system.0, Action::Left);
            if !shift {
                editor.0.set_select_opt(None);
            }
            return;
        }
        if keys.just_pressed(KeyCode::Right) {
            editor.0.action(&mut font_system.0, Action::Right);
            if !shift {
                editor.0.set_select_opt(None);
            }
            return;
        }
        if keys.just_pressed(KeyCode::Up) {
            editor.0.action(&mut font_system.0, Action::Up);
            if !shift {
                editor.0.set_select_opt(None);
            }
            return;
        }
        if keys.just_pressed(KeyCode::Down) {
            editor.0.action(&mut font_system.0, Action::Down);
            if !shift {
                editor.0.set_select_opt(None);
            }
            return;
        }

        if keys.just_pressed(KeyCode::Back) & !readonly {
            // fix for issue #8
            if let Some(select) = editor.0.select_opt() {
                if editor.0.cursor().line == select.line && editor.0.cursor().index == select.index
                {
                    editor.0.set_select_opt(None);
                }
            }
            *is_deleting = true;
            editor.0.action(&mut font_system.0, Action::Backspace);
        }

        if keys.just_released(KeyCode::Back) {
            *is_deleting = false;
        }
        if keys.just_pressed(KeyCode::Delete) && !readonly {
            editor.0.action(&mut font_system.0, Action::Delete);
        }
        if keys.just_pressed(KeyCode::Escape) {
            editor.0.action(&mut font_system.0, Action::Escape);
        }
        if command && keys.just_pressed(KeyCode::A) {
            editor.0.action(&mut font_system.0, Action::BufferEnd);
            let current_cursor = editor.0.cursor();
            editor.0.set_select_opt(Some(Cursor {
                line: 0,
                index: 0,
                affinity: current_cursor.affinity,
                color: current_cursor.color,
            }));
            return;
        }
        if keys.just_pressed(KeyCode::Home) {
            editor.0.action(&mut font_system.0, Action::Home);
            if !shift {
                editor.0.set_select_opt(None);
            }
            return;
        }
        if keys.just_pressed(KeyCode::End) {
            editor.0.action(&mut font_system.0, Action::End);
            if !shift {
                editor.0.set_select_opt(None);
            }
            return;
        }
        if keys.just_pressed(KeyCode::PageUp) {
            editor.0.action(&mut font_system.0, Action::PageUp);
            if !shift {
                editor.0.set_select_opt(None);
            }
            return;
        }
        if keys.just_pressed(KeyCode::PageDown) {
            editor.0.action(&mut font_system.0, Action::PageDown);
            if !shift {
                editor.0.set_select_opt(None);
            }
            return;
        }

        let mut is_clipboard = false;
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                if command && keys.just_pressed(KeyCode::C) {
                    if password_opt.is_some() {
                        return;
                    }
                    if let Some(text) = editor.0.copy_selection() {
                        clipboard.set_text(text).unwrap();
                        return;
                    }
                }
                if command && keys.just_pressed(KeyCode::X) && !readonly {
                    if password_opt.is_some() {
                        return;
                    }
                    if let Some(text) = editor.0.copy_selection() {
                        clipboard.set_text(text).unwrap();
                        editor.0.delete_selection();
                    }
                    is_clipboard = true;
                }
                if command && keys.just_pressed(KeyCode::V) && !readonly {
                    if let Ok(text) = clipboard.get_text() {
                        for c in text.chars() {
                            if max_chars.0 == 0 || editor.get_text().len() < max_chars.0 {
                                if c == 0xA as char {
                                    if max_lines.0 == 0
                                        || editor.0.buffer().lines.len() < max_lines.0
                                    {
                                        editor.0.action(&mut font_system.0, Action::Insert(c));
                                    }
                                } else {
                                    if password_opt.is_some() && c.len_utf8() > 1 {
                                        println!("Cannot input multi-byte char '{}' to password field! See https://github.com/StaffEngineer/bevy_cosmic_edit/pull/99#issuecomment-1782607486",c);
                                        continue;
                                    }
                                    editor.0.action(&mut font_system.0, Action::Insert(c));
                                }
                            }
                        }
                    }
                    is_clipboard = true;
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            if command && keys.just_pressed(KeyCode::C) {
                if password_opt.is_some() {
                    return;
                }
                if let Some(text) = editor.0.copy_selection() {
                    write_clipboard_wasm(text.as_str());
                    return;
                }
            }

            if command && keys.just_pressed(KeyCode::X) && !readonly {
                if password_opt.is_some() {
                    return;
                }
                if let Some(text) = editor.0.copy_selection() {
                    write_clipboard_wasm(text.as_str());
                    editor.0.delete_selection();
                }
                is_clipboard = true;
            }
            if command && keys.just_pressed(KeyCode::V) && !readonly {
                let tx = _channel.unwrap().tx.clone();
                let _task = AsyncComputeTaskPool::get().spawn(async move {
                    let promise = read_clipboard_wasm();

                    let result = JsFuture::from(promise).await;

                    if let Ok(js_text) = result {
                        if let Some(text) = js_text.as_string() {
                            let _ = tx.try_send(WasmPaste { text, entity });
                        }
                    }
                });

                return;
            }
        }

        let mut is_edit = is_clipboard;
        let mut is_return = false;
        if keys.just_pressed(KeyCode::Return) && !readonly {
            is_return = true;
            if (max_lines.0 == 0 || editor.0.buffer().lines.len() < max_lines.0)
                && (max_chars.0 == 0 || editor.get_text().len() < max_chars.0)
            {
                // to have new line on wasm rather than E
                is_edit = true;
                editor.0.action(&mut font_system.0, Action::Insert('\n'));
            }
        }

        if !(is_clipboard || is_return || readonly) {
            for char_ev in char_evr.read() {
                is_edit = true;
                if *is_deleting {
                    editor.0.action(&mut font_system.0, Action::Backspace);
                } else if !command && (max_chars.0 == 0 || editor.get_text().len() < max_chars.0) {
                    if password_opt.is_some() && char_ev.char.len_utf8() > 1 {
                        println!("Cannot input multi-byte char '{}' to password field! See https://github.com/StaffEngineer/bevy_cosmic_edit/pull/99#issuecomment-1782607486",char_ev.char);
                        continue;
                    }
                    editor
                        .0
                        .action(&mut font_system.0, Action::Insert(char_ev.char));
                }
            }
        }

        // skip event + history if undo/redo keys pressed

        let requested_redo = keypress_redo(&keys);
        let requested_undo = command && keys.just_pressed(KeyCode::Z);

        if !is_edit || readonly || requested_redo || requested_undo {
            return;
        }

        evw_changed.send(CosmicTextChanged((entity, editor.get_text())));

        if let Some(last_edit_duration) = *edits_duration {
            if Duration::from_millis(now_ms as u64) - last_edit_duration
                > Duration::from_millis(150)
            {
                save_edit_history(&mut editor.0, attrs, &mut edit_history);
                *edits_duration = Some(Duration::from_millis(now_ms as u64));
            }
        } else {
            save_edit_history(&mut editor.0, attrs, &mut edit_history);
            *edits_duration = Some(Duration::from_millis(now_ms as u64));
        }
    }
}

fn keypress_command(keys: &Input<KeyCode>) -> bool {
    #[cfg(target_os = "macos")]
    let command = keys.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight]);

    #[cfg(not(target_os = "macos"))]
    let command = keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);

    #[cfg(target_arch = "wasm32")]
    let command = if web_sys::window()
        .unwrap()
        .navigator()
        .user_agent()
        .unwrap_or("NoUA".into())
        .contains("Macintosh")
    {
        keys.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight])
    } else {
        command
    };

    command
}

fn keypress_redo(keys: &Input<KeyCode>) -> bool {
    let command = keypress_command(keys);
    let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

    #[cfg(not(target_os = "windows"))]
    let requested_redo = command && shift && keys.just_pressed(KeyCode::Z);

    // TODO: windows OS detection for wasm here
    #[cfg(target_os = "windows")]
    let requested_redo = command && keys.just_pressed(KeyCode::Y);

    requested_redo
}

pub(crate) fn undo_redo(
    active_editor: Res<Focus>,
    keys: Res<Input<KeyCode>>,
    mut editor_q: Query<
        (&mut CosmicEditor, &CosmicAttrs, &mut CosmicEditHistory),
        Without<ReadOnly>,
    >,
    mut evw_changed: EventWriter<CosmicTextChanged>,
) {
    let entity = match active_editor.0 {
        Some(entity) => entity,
        None => return,
    };

    let (mut editor, attrs, mut edit_history) = match editor_q.get_mut(entity) {
        Ok(components) => components,
        Err(_) => return,
    };

    let command = keypress_command(&keys);

    let attrs = &attrs.0;

    let requested_redo = keypress_redo(&keys);
    let requested_undo = command & keys.just_pressed(KeyCode::Z);

    if !(requested_redo || requested_undo) {
        return;
    }

    let edits = &edit_history.edits;

    if edits.is_empty() {
        return;
    }

    // use not redo rather than undo, cos undo will be true when redo is
    if !requested_redo && edit_history.current_edit == 0 {
        return;
    }

    if requested_redo && edit_history.current_edit == edits.len() - 1 {
        return;
    }

    let index = if requested_redo {
        edit_history.current_edit + 1
    } else {
        edit_history.current_edit - 1
    };

    if let Some(current_edit) = edits.get(index) {
        editor.0.buffer_mut().lines.clear();
        for line in current_edit.lines.iter() {
            let mut line_text = String::new();
            let mut attrs_list = AttrsList::new(attrs.as_attrs());
            for (text, attrs) in line.iter() {
                let start = line_text.len();
                line_text.push_str(text);
                let end = line_text.len();
                attrs_list.add_span(start..end, attrs.as_attrs());
            }
            editor.0.buffer_mut().lines.push(BufferLine::new(
                line_text,
                attrs_list,
                Shaping::Advanced,
            ));
        }
        editor.0.set_cursor(current_edit.cursor);
        editor.0.set_select_opt(None); // prevent auto selection of redo-inserted text
        editor.0.buffer_mut().set_redraw(true);
        edit_history.current_edit = index;
        evw_changed.send(CosmicTextChanged((entity, editor.get_text())));
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn write_clipboard_wasm(text: &str) {
    let clipboard = web_sys::window()
        .unwrap()
        .navigator()
        .clipboard()
        .expect("Clipboard not found!");
    let _result = clipboard.write_text(text);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn read_clipboard_wasm() -> Promise {
    let clipboard = web_sys::window()
        .unwrap()
        .navigator()
        .clipboard()
        .expect("Clipboard not found!");
    clipboard.read_text()
}

#[cfg(target_arch = "wasm32")]
pub fn poll_wasm_paste(
    channel: Res<WasmPasteAsyncChannel>,
    mut editor_q: Query<
        (
            &mut CosmicEditor,
            &CosmicAttrs,
            &CosmicMaxChars,
            &CosmicMaxChars,
            &mut CosmicEditHistory,
            Option<&PasswordInput>,
        ),
        Without<ReadOnly>,
    >,
    mut evw_changed: EventWriter<CosmicTextChanged>,
    mut font_system: ResMut<CosmicFontSystem>,
) {
    let inlet = channel.rx.try_recv();
    match inlet {
        Ok(inlet) => {
            let entity = inlet.entity;
            if let Ok((mut editor, attrs, max_chars, max_lines, mut edit_history, password_opt)) =
                editor_q.get_mut(entity)
            {
                let text = inlet.text;
                let attrs = &attrs.0;
                for c in text.chars() {
                    if max_chars.0 == 0 || editor.get_text().len() < max_chars.0 {
                        if c == 0xA as char {
                            if max_lines.0 == 0 || editor.0.buffer().lines.len() < max_lines.0 {
                                editor.0.action(&mut font_system.0, Action::Insert(c));
                            }
                        } else {
                            if password_opt.is_some() && c.len_utf8() > 1 {
                                // TODO: console.log here instead
                                println!("Cannot input multi-byte char '{}' to password field! See https://github.com/StaffEngineer/bevy_cosmic_edit/pull/99#issuecomment-1782607486",c);
                                continue;
                            }
                            editor.0.action(&mut font_system.0, Action::Insert(c));
                        }
                    }
                }

                evw_changed.send(CosmicTextChanged((entity, editor.get_text())));
                save_edit_history(&mut editor.0, attrs, &mut edit_history);
            }
        }
        Err(_) => {}
    }
}
