use bevy::{
    ecs::query::WorldQuery,
    prelude::*,
    render::camera::RenderTarget,
    time::Time,
    ui::{CalculatedClip, Node, UiStack},
    window::{PrimaryWindow, Window, WindowRef},
};
use polako_constructivism::Construct;

pub struct PolakoInputPlugin;

impl Plugin for PolakoInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PointerInput>();
        app.add_systems(PreUpdate, bypass_filter_system);
        app.add_systems(PreUpdate, pointer_input_system.after(bypass_filter_system));
    }
}

#[derive(Construct, Default, Clone)]
pub struct PointerInputPosition {
    abs: Vec2,
    rel: Vec2,
}

#[derive(Construct)]
pub struct PointerInputDrag {
    #[prop(construct)]
    position: PointerInputPosition,
    source_entities: Vec<Entity>,
}

pub enum PointerInputData {
    Up,
    Down,
    Motion,
    DragStart,
    Drag,
    DragStop,
    Hover,
    Focus,
}

#[derive(Event)]
pub struct PointerInput {
    pub entity: Entity,
    pub position: PointerInputPosition,
    pub data: PointerInputData,
}

impl PointerInput {
    pub fn up(&self) -> bool {
        match self.data {
            PointerInputData::Up => true,
            _ => false,
        }
    }
    pub fn down(&self) -> bool {
        match self.data {
            PointerInputData::Down => true,
            _ => false,
        }
    }
    pub fn motion(&self) -> bool {
        match self.data {
            PointerInputData::Motion => true,
            _ => false,
        }
    }
    pub fn hover(&self) -> bool {
        match self.data {
            PointerInputData::Hover => true,
            _ => false,
        }
    }
    pub fn focus(&self) -> bool {
        match self.data {
            PointerInputData::Focus => true,
            _ => false,
        }
    }
    pub fn drag_start(&self) -> bool {
        match self.data {
            PointerInputData::DragStart => true,
            _ => false,
        }
    }
    pub fn drag(&self) -> bool {
        matches!(self.data, PointerInputData::Drag)
    }
    pub fn drag_stop(&self) -> bool {
        matches!(self.data, PointerInputData::DragStop)
    }
}

#[derive(Default, Component, Clone, Copy, Debug)]
pub enum PointerFilter {
    #[default]
    Default,
    Ignore,
    Pass,
    Block,
}

// derive_behaviour

impl PointerFilter {
    pub fn pointer_filter(&self) -> Self {
        *self
    }

    pub fn set_pointer_filter(&mut self, value: Self) {
        *self = value
    }
}

#[derive(Component)]
pub enum ActivePointerFilter {
    Pass,
    Block,
}

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct PointerQuery {
    entity: Entity,
    node: &'static Node,
    global_transform: &'static GlobalTransform,
    filter: Option<&'static ActivePointerFilter>,
    calculated_clip: Option<&'static CalculatedClip>,
    computed_visibility: Option<&'static InheritedVisibility>,
}

#[derive(Default)]
pub struct PointerSystemState {
    pressed_entities: Vec<Entity>,
    drag_in_seconds: Option<f32>,
    dragging_from: Vec<Entity>,
    press_position: Option<Vec2>,
    last_cursor_position: Option<Vec2>,
    dragging: bool,
}

pub fn bypass_filter_system(
    nodes: Query<(Entity, &PointerFilter), Changed<PointerFilter>>,
    mut commands: Commands,
) {
    for (entity, filter) in nodes.iter() {
        info!("Bypassing {filter:?} for {entity:?}");
        match filter {
            PointerFilter::Pass => commands.entity(entity).insert(ActivePointerFilter::Pass),
            PointerFilter::Block => commands.entity(entity).insert(ActivePointerFilter::Block),
            _ => commands.entity(entity).remove::<ActivePointerFilter>(),
        };
    }
}

// pointer_input_system is the rewriten bevy's ui_focus_system
// it emit PointerEvent with associated entities and data.
pub fn pointer_input_system(
    mut state: Local<PointerSystemState>,
    camera: Query<(&Camera, Option<&UiCameraConfig>)>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    windows: Query<&Window, Without<PrimaryWindow>>,
    mouse_button_input: Res<Input<MouseButton>>,
    touches_input: Res<Touches>,
    ui_stack: Res<UiStack>,
    time: Res<Time>,
    pointer_query: Query<PointerQuery>,
    mut events: EventWriter<PointerInput>,
) {
    let up =
        mouse_button_input.just_released(MouseButton::Left) || touches_input.any_just_released();
    let down =
        mouse_button_input.just_pressed(MouseButton::Left) || touches_input.any_just_pressed();

    let is_ui_disabled =
        |camera_ui| matches!(camera_ui, Some(&UiCameraConfig { show_ui: false, .. }));

    let cursor_position = camera
        .iter()
        .filter(|(_, camera_ui)| !is_ui_disabled(*camera_ui))
        .filter_map(|(camera, _)| {
            if let RenderTarget::Window(window_ref) = camera.target {
                Some(window_ref)
            } else {
                None
            }
        })
        .filter_map(|window_ref| {
            if let WindowRef::Entity(entity) = window_ref {
                windows.get(entity).ok()
            } else {
                primary_window.get_single().ok()
            }
        })
        .filter(|window| window.focused)
        .find_map(|window| window.cursor_position())
        .or_else(|| touches_input.first_pressed_position());

    if down {
        state.press_position = cursor_position;
        state.drag_in_seconds = Some(0.5);
    }
    let delta = match (cursor_position, state.last_cursor_position) {
        (Some(c), Some(l)) => c - l,
        _ => Vec2::ZERO,
    };

    state.last_cursor_position = cursor_position;
    let mut moused_over_nodes = ui_stack
        .uinodes
        .iter()
        // reverse the iterator to traverse the tree from closest nodes to furthest
        .rev()
        .filter_map(|entity| {
            if let Ok(node) = pointer_query.get(*entity) {
                // Nodes that are not rendered should not be interactable
                if let Some(computed_visibility) = node.computed_visibility {
                    if !computed_visibility.get() {
                        return None;
                    }
                }

                let position = node.global_transform.translation();
                let ui_position = position.truncate();
                let extents = node.node.size() / 2.0;
                let mut min = ui_position - extents;
                let mut max = ui_position + extents;
                if let Some(clip) = node.calculated_clip {
                    min = Vec2::max(min, clip.clip.min);
                    max = Vec2::min(max, clip.clip.max);
                }
                // if the current cursor position is within the bounds of the node, consider it for
                // emiting the event
                let contains_cursor = if let Some(cursor_position) = cursor_position {
                    (min.x..max.x).contains(&cursor_position.x)
                        && (min.y..max.y).contains(&cursor_position.y)
                } else {
                    false
                };

                if contains_cursor {
                    Some(*entity)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<Entity>>()
        .into_iter();
    let mut down_entities = vec![];
    let mut up_entities = vec![];
    let mut pressed_entities = vec![];
    let mut drag_entities = vec![];
    let mut motion_entities = vec![];
    let mut drag_start_entities = vec![];
    let delta_len = delta.length();
    if let Some(drag_in_seconds) = &mut state.drag_in_seconds {
        *drag_in_seconds -= time.delta_seconds();
        *drag_in_seconds -= delta_len / 50.;
    }
    if !state.dragging
        && !state.pressed_entities.is_empty()
        && state.drag_in_seconds.is_some()
        && state.drag_in_seconds.unwrap() <= 0.
    {
        state.dragging = true;
        drag_start_entities = state.pressed_entities.clone();
    }
    let send_drag_stop = state.dragging && up;
    let mut drag_stop_entities = vec![];
    if send_drag_stop {
        drag_stop_entities = state.dragging_from.clone();
    }

    let mut iter = pointer_query.iter_many(moused_over_nodes.by_ref());
    while let Some(node) = iter.fetch_next() {
        if node.filter.is_none() {
            continue;
        }
        let entity = node.entity;

        if down {
            state.pressed_entities.push(entity);
            down_entities.push(entity);
        }
        if up {
            up_entities.push(entity);
            let pressed_entity_idx = state.pressed_entities.iter().position(|e| *e == entity);
            if let Some(pressed_entity_idx) = pressed_entity_idx {
                state.pressed_entities.remove(pressed_entity_idx);
                pressed_entities.push(entity);
            }
        }
        if delta != Vec2::ZERO {
            if state.dragging {
                drag_entities.push(entity);
            } else {
                motion_entities.push(entity);
            };
        }
        if send_drag_stop {
            drag_stop_entities.push(entity);
        }

        match node.filter.unwrap() {
            ActivePointerFilter::Block => {
                break;
            }
            ActivePointerFilter::Pass => { /* allow the next node to be processed */ }
        }
    }

    let Some(pos) = cursor_position else { return };
    if down_entities.len() > 0 {
        // TODO: do not forget about drag_in_seconds here
        // state.was_down_at = time.elapsed_seconds();
        for entity in down_entities.iter().copied() {
            events.send(PointerInput {
                entity,
                // TODO: do not forget about calculating screen/windown/viewport/relative position
                position: PointerInputPosition { abs: pos, rel: pos },
                data: PointerInputData::Down,
            });
        }
    }

    for entity in motion_entities.iter().copied() {
        info!("sending PointerInput::hover event");
        events.send(PointerInput {
            entity,
            // TODO: do not forget about calculating screen/windown/viewport/relative position
            position: PointerInputPosition { abs: pos, rel: pos },
            // delta,
            data: PointerInputData::Motion,
        });
    }
    for entity in drag_start_entities.iter().copied() {
        // state.dragging_from = drag_start_entities.clone();
        events.send(PointerInput {
            entity,
            // TODO: do not forget about calculating screen/windown/viewport/relative position
            position: PointerInputPosition { abs: pos, rel: pos },
            // delta,
            data: PointerInputData::DragStart,
        });
    }
    if drag_stop_entities.is_empty() {
        for entity in drag_entities.iter().copied() {
            events.send(PointerInput {
                entity,
                // TODO: do not forget about calculating screen/windown/viewport/relative position
                position: PointerInputPosition { abs: pos, rel: pos },
                // delta,
                data: PointerInputData::Drag,
            });
        }
    }

    for entity in drag_stop_entities.iter().copied() {
        events.send(PointerInput {
            entity,
            // TODO: do not forget about calculating screen/windown/viewport/relative position
            position: PointerInputPosition { abs: pos, rel: pos },
            // delta,
            // entities: drag_stop_entities,
            data: PointerInputData::DragStop,
        });
    }
    for entity in up_entities.iter().copied() {
        events.send(PointerInput {
            entity,
            // TODO: do not forget about calculating screen/windown/viewport/relative position
            position: PointerInputPosition { abs: pos, rel: pos },
            // delta,
            // entities: up_entities,
            data: PointerInputData::Up,
        });
    }

    if up {
        state.pressed_entities.clear();
        state.dragging_from.clear();
        state.press_position = None;
        state.dragging = false;
    }
}
