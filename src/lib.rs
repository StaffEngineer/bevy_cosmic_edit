#![allow(clippy::type_complexity)]

mod buffer;
mod cursor;
pub mod focus;
mod input;
mod layout;
mod render;

use std::{collections::VecDeque, path::PathBuf, time::Duration};

use bevy::{prelude::*, transform::TransformSystem};
use buffer::{add_font_system, set_initial_scale, set_redraw, swap_target_handle};
pub use buffer::{get_x_offset_center, get_y_offset_center, CosmicBuffer};
pub use cosmic_text::{
    Action, Attrs, AttrsOwned, Color as CosmicColor, Cursor, Edit, Family, Style as FontStyle,
    Weight as FontWeight,
};
use cosmic_text::{
    AttrsList, Buffer, BufferLine, Editor, FontSystem, Metrics, Shaping, SwashCache,
};
use cursor::{change_cursor, hover_sprites, hover_ui};
pub use cursor::{TextHoverIn, TextHoverOut};
use focus::{add_editor_to_focused, drop_editor_unfocused, FocusedWidget};
use input::{input_kb, input_mouse, ClickTimer};
#[cfg(target_arch = "wasm32")]
use input::{poll_wasm_paste, WasmPaste, WasmPasteAsyncChannel};
use layout::{
    auto_height, new_image_from_default, on_scale_factor_change, reshape, set_buffer_size,
    set_cursor, set_padding, set_sprite_size_from_ui, set_widget_size, CosmicPadding,
    CosmicWidgetSize,
};
use render::{blink_cursor, render_texture, SwashCacheState};

#[cfg(feature = "multicam")]
#[derive(Component)]
pub struct CosmicPrimaryCamera;

#[derive(Clone, PartialEq, Debug)]
pub enum CosmicText {
    OneStyle(String),
    MultiStyle(Vec<Vec<(String, AttrsOwned)>>),
}

impl Default for CosmicText {
    fn default() -> Self {
        Self::OneStyle(String::new())
    }
}

#[derive(Clone, Component, PartialEq, Default)]
pub enum CosmicMode {
    InfiniteLine,
    AutoHeight,
    #[default]
    Wrap,
}

#[derive(Default)]
pub enum CursorConfig {
    #[default]
    Default,
    Events,
    None,
}

/// Enum representing the position of the cosmic text.
#[derive(Clone, Component, Default)]
pub enum CosmicTextPosition {
    #[default]
    Center,
    TopLeft {
        padding: i32,
    },
    Left {
        padding: i32,
    },
}

#[derive(Event, Debug)]
pub struct CosmicTextChanged(pub (Entity, String));

#[derive(Resource, Deref, DerefMut)]
pub struct CosmicFontSystem(pub FontSystem);

#[derive(Component)]
pub struct ReadOnly; // tag component

#[derive(Component, Debug)]
pub struct XOffset(Option<(f32, f32)>);

#[derive(Component, Deref, DerefMut)]
pub struct CosmicEditor {
    #[deref]
    pub editor: Editor<'static>,
    pub cursor_visible: bool,
    pub cursor_timer: Timer,
}

impl CosmicEditor {
    fn new(editor: Editor<'static>) -> Self {
        Self {
            editor,
            cursor_visible: true,
            cursor_timer: Timer::new(Duration::from_millis(530), TimerMode::Repeating),
        }
    }
}

#[derive(Component)]
pub struct CosmicAttrs(pub AttrsOwned);

impl Default for CosmicAttrs {
    fn default() -> Self {
        CosmicAttrs(AttrsOwned::new(Attrs::new()))
    }
}

#[derive(Component, Default)]
pub struct CosmicBackground(pub Option<Handle<Image>>);

#[derive(Component, Default)]
pub struct FillColor(pub Color);

#[derive(Component, Default)]
pub struct CosmicMaxLines(pub usize);

#[derive(Component, Default)]
pub struct CosmicMaxChars(pub usize);

#[derive(Component)]
pub struct CosmicSource(pub Entity);

#[derive(Bundle)]
pub struct CosmicEditBundle {
    // cosmic bits
    pub buffer: CosmicBuffer,
    // render bits
    pub fill_color: FillColor,
    pub attrs: CosmicAttrs,
    pub background_image: CosmicBackground,
    pub sprite_bundle: SpriteBundle,
    // restriction bits
    pub max_lines: CosmicMaxLines,
    pub max_chars: CosmicMaxChars,
    // layout bits
    pub x_offset: XOffset,
    pub mode: CosmicMode,
    pub text_position: CosmicTextPosition,
    pub padding: CosmicPadding,
    pub widget_size: CosmicWidgetSize,
}

impl Default for CosmicEditBundle {
    fn default() -> Self {
        CosmicEditBundle {
            buffer: Default::default(),
            fill_color: Default::default(),
            text_position: Default::default(),
            attrs: Default::default(),
            background_image: Default::default(),
            max_lines: Default::default(),
            max_chars: Default::default(),
            mode: Default::default(),
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::ONE * 128.0),
                    ..default()
                },
                visibility: Visibility::Hidden,
                ..default()
            },
            x_offset: XOffset(None),
            padding: Default::default(),
            widget_size: Default::default(),
        }
    }
}

#[derive(Clone)]
pub struct EditHistoryItem {
    pub cursor: Cursor,
    pub lines: Vec<Vec<(String, AttrsOwned)>>,
}

#[derive(Component, Default)]
pub struct CosmicEditHistory {
    pub edits: VecDeque<EditHistoryItem>,
    pub current_edit: usize,
}

/// Resource struct that holds configuration options for cosmic fonts.
#[derive(Resource, Clone)]
pub struct CosmicFontConfig {
    pub fonts_dir_path: Option<PathBuf>,
    pub font_bytes: Option<Vec<&'static [u8]>>,
    pub load_system_fonts: bool, // caution: this can be relatively slow
}

impl Default for CosmicFontConfig {
    fn default() -> Self {
        let fallback_font = include_bytes!("./font/FiraMono-Regular-subset.ttf");
        Self {
            load_system_fonts: false,
            font_bytes: Some(vec![fallback_font]),
            fonts_dir_path: None,
        }
    }
}

/// Plugin struct that adds systems and initializes resources related to cosmic edit functionality.
#[derive(Default)]
pub struct CosmicEditPlugin {
    pub font_config: CosmicFontConfig,
    pub change_cursor: CursorConfig,
}

impl Plugin for CosmicEditPlugin {
    fn build(&self, app: &mut App) {
        let font_system = create_cosmic_font_system(self.font_config.clone());

        let layout_systems = (
            (new_image_from_default, set_sprite_size_from_ui),
            (set_widget_size, set_buffer_size),
            auto_height,
            set_padding,
            set_cursor,
        )
            .chain();

        app.add_systems(
            First,
            (
                add_font_system,
                set_initial_scale,
                set_redraw,
                swap_target_handle,
                on_scale_factor_change,
            )
                .chain(),
        )
        .add_systems(PreUpdate, (input_mouse,).chain())
        .add_systems(
            Update,
            (
                drop_editor_unfocused,
                add_editor_to_focused,
                input_kb,
                reshape,
                blink_cursor,
            )
                .chain(),
        )
        .add_systems(
            PostUpdate,
            (layout_systems, render_texture)
                .chain()
                .after(TransformSystem::TransformPropagate),
        )
        .init_resource::<FocusedWidget>()
        .insert_resource(SwashCacheState {
            swash_cache: SwashCache::new(),
        })
        .insert_resource(CosmicFontSystem(font_system))
        .insert_resource(ClickTimer(Timer::from_seconds(0.5, TimerMode::Once)))
        .add_event::<CosmicTextChanged>();

        match self.change_cursor {
            CursorConfig::Default => {
                app.add_systems(Update, (hover_sprites, hover_ui, change_cursor))
                    .add_event::<TextHoverIn>()
                    .add_event::<TextHoverOut>();
            }
            CursorConfig::Events => {
                app.add_systems(Update, (hover_sprites, hover_ui))
                    .add_event::<TextHoverIn>()
                    .add_event::<TextHoverOut>();
            }
            CursorConfig::None => {}
        }

        #[cfg(target_arch = "wasm32")]
        {
            let (tx, rx) = crossbeam_channel::bounded::<WasmPaste>(1);
            app.insert_resource(WasmPasteAsyncChannel { tx, rx })
                .add_systems(Update, poll_wasm_paste);
        }
    }
}

fn create_cosmic_font_system(cosmic_font_config: CosmicFontConfig) -> FontSystem {
    let locale = sys_locale::get_locale().unwrap_or_else(|| String::from("en-US"));
    let mut db = cosmic_text::fontdb::Database::new();
    if let Some(dir_path) = cosmic_font_config.fonts_dir_path.clone() {
        db.load_fonts_dir(dir_path);
    }
    if let Some(custom_font_data) = &cosmic_font_config.font_bytes {
        for elem in custom_font_data {
            db.load_font_data(elem.to_vec());
        }
    }
    if cosmic_font_config.load_system_fonts {
        db.load_system_fonts();
    }
    cosmic_text::FontSystem::new_with_locale_and_db(locale, db)
}

pub fn get_node_cursor_pos(
    window: &Window,
    node_transform: &GlobalTransform,
    size: (f32, f32),
    is_ui_node: bool,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<(f32, f32)> {
    let (x_min, y_min, x_max, y_max) = (
        node_transform.affine().translation.x - size.0 / 2.,
        node_transform.affine().translation.y - size.1 / 2.,
        node_transform.affine().translation.x + size.0 / 2.,
        node_transform.affine().translation.y + size.1 / 2.,
    );

    window.cursor_position().and_then(|pos| {
        if is_ui_node {
            if x_min < pos.x && pos.x < x_max && y_min < pos.y && pos.y < y_max {
                Some((pos.x - x_min, pos.y - y_min))
            } else {
                None
            }
        } else {
            camera
                .viewport_to_world_2d(camera_transform, pos)
                .and_then(|pos| {
                    if x_min < pos.x && pos.x < x_max && y_min < pos.y && pos.y < y_max {
                        Some((pos.x - x_min, y_max - pos.y))
                    } else {
                        None
                    }
                })
        }
    })
}

fn _trim_text(text: CosmicText, max_chars: usize, max_lines: usize) -> CosmicText {
    if max_chars == 0 && max_lines == 0 {
        // no limits, no work to do
        return text;
    }

    match text {
        CosmicText::OneStyle(mut string) => {
            if max_chars != 0 {
                string.truncate(max_chars);
            }

            if max_lines == 0 {
                return CosmicText::OneStyle(string);
            }

            let mut line_acc = 0;
            let mut char_pos = 0;
            for c in string.chars() {
                char_pos += 1;
                if c == 0xA as char {
                    line_acc += 1;
                    if line_acc >= max_lines {
                        // break text to pos
                        string.truncate(char_pos);
                        break;
                    }
                }
            }

            CosmicText::OneStyle(string)
        }
        CosmicText::MultiStyle(lines) => {
            let mut char_acc = 0;
            let mut line_acc = 0;

            let mut trimmed_styles = vec![];

            for line in lines.iter() {
                line_acc += 1;
                char_acc += 1; // count newlines for consistent behaviour

                if (line_acc >= max_lines && max_lines > 0)
                    || (char_acc >= max_chars && max_chars > 0)
                {
                    break;
                }

                let mut strs = vec![];

                for (string, attrs) in line.iter() {
                    if char_acc >= max_chars && max_chars > 0 {
                        break;
                    }

                    let mut string = string.clone();

                    if max_chars > 0 {
                        string.truncate(max_chars - char_acc);
                        char_acc += string.len();
                    }

                    if max_lines > 0 {
                        for c in string.chars() {
                            if c == 0xA as char {
                                line_acc += 1;
                                char_acc += 1; // count newlines for consistent behaviour
                                if line_acc >= max_lines {
                                    break;
                                }
                            }
                        }
                    }

                    strs.push((string, attrs.clone()));
                }
                trimmed_styles.push(strs);
            }
            CosmicText::MultiStyle(trimmed_styles)
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub fn get_timestamp() -> f64 {
    js_sys::Date::now()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_timestamp() -> f64 {
    use std::time::SystemTime;
    use std::time::UNIX_EPOCH;
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    duration.as_millis() as f64
}

#[cfg(test)]
mod tests {
    use crate::*;

    use self::buffer::CosmicBuffer;

    fn test_spawn_cosmic_edit_system(mut commands: Commands) {
        commands.spawn(CosmicEditBundle {
            ..Default::default()
        });
    }

    #[test]
    fn test_spawn_cosmic_edit() {
        let mut app = App::new();
        app.add_plugins(TaskPoolPlugin::default());
        app.add_plugins(AssetPlugin::default());
        app.insert_resource(CosmicFontSystem(create_cosmic_font_system(
            CosmicFontConfig::default(),
        )));
        app.add_systems(Update, test_spawn_cosmic_edit_system);

        let input = ButtonInput::<KeyCode>::default();
        app.insert_resource(input);
        let mouse_input: ButtonInput<MouseButton> = ButtonInput::<MouseButton>::default();
        app.insert_resource(mouse_input);

        app.add_event::<ReceivedCharacter>();

        app.update();

        let mut text_nodes_query = app.world.query::<&CosmicBuffer>();
        for cosmic_editor in text_nodes_query.iter(&app.world) {
            insta::assert_debug_snapshot!(cosmic_editor
                .lines
                .iter()
                .map(|line| line.text())
                .collect::<Vec<_>>());
        }
    }
}
