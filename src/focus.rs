use std::sync::Arc;

use crate::*;
use bevy::prelude::*;
use cosmic_text::{Edit, Editor};

/// System set for focus systems. Runs in `PostUpdate`
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FocusSet;

pub(crate) struct FocusPlugin;

impl Plugin for FocusPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (drop_editor_unfocused, add_editor_to_focused)
                .chain()
                .in_set(FocusSet)
                .after(WidgetSet),
        )
        .init_resource::<FocusedWidget>();
    }
}

/// Resource struct that keeps track of the currently active editor entity.
#[derive(Resource, Default, Deref, DerefMut)]
pub struct FocusedWidget(pub Option<Entity>);

pub(crate) fn add_editor_to_focused(
    mut commands: Commands,
    active_editor: Res<FocusedWidget>,
    q: Query<&CosmicBuffer, Without<CosmicEditor>>,
) {
    if let Some(e) = active_editor.0 {
        let Ok(b) = q.get(e) else {
            return;
        };
        let mut editor = Editor::new(Arc::clone(&b.0));
        editor.set_redraw(true);
        commands.entity(e).insert(CosmicEditor::new(editor));
    }
}

pub(crate) fn drop_editor_unfocused(
    mut commands: Commands,
    active_editor: Res<FocusedWidget>,
    mut q: Query<(Entity, &mut CosmicBuffer, &CosmicEditor)>,
) {
    if active_editor.0.is_none() {
        for (e, mut b, ed) in q.iter_mut() {
            let buffer = Arc::make_mut(&mut b.0);
            buffer.lines = ed.with_buffer(|buf| buf.lines.clone());
            buffer.set_redraw(true);
            commands.entity(e).remove::<CosmicEditor>();
        }
    } else if let Some(focused) = active_editor.0 {
        for (e, mut b, ed) in q.iter_mut() {
            if e != focused {
                let buffer = Arc::make_mut(&mut b.0);
                buffer.lines = ed.with_buffer(|buf| buf.lines.clone());
                buffer.set_redraw(true);
                commands.entity(e).remove::<CosmicEditor>();
            }
        }
    }
}
