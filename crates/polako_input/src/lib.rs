use bevy::render::camera::NormalizedRenderTarget;
use bevy::utils::HashMap;
use bevy::{
    ecs::query::{QueryData, WorldQuery},
    prelude::*,
    render::camera::RenderTarget,
    time::Time,
    ui::{CalculatedClip, Node, UiStack},
    window::{PrimaryWindow, Window, WindowRef},
};
use polako_constructivism::bridge::ReadOnly;
use polako_constructivism::derive_construct;
use polako_constructivism::{Construct, Get, Singleton};

pub struct PolakoInputPlugin;

impl Plugin for PolakoInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PointerInput>();
        app.add_systems(PreUpdate, bypass_filter_system);
        app.add_systems(PreUpdate, pointer_input_system.after(bypass_filter_system));
    }
}

#[derive(Construct, Default, Clone, Copy)]
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
    pub fn position(&self) -> PointerInputPosition {
        self.position
    }
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

impl ReadOnly for PointerInput {}

derive_construct! {
    seq => PointerInput -> Nothing;
    construct => (entity: Entity, position: PointerInputPosition, data: PointerInputData) -> {
        PointerInput { entity, position, data }
    };
    props => {
        position: PointerInputPosition = construct;
        motion: bool = [motion, readonly];
    };
}
// TODO: implement getters(&self) in constructivism
impl PointerInput {
    pub fn getters(&self) -> &'static pointerinput_construct::Props<Get> {
        pointerinput_construct::Props::instance()
    }
}

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

#[derive(QueryData)]
#[query_data(mutable)]
pub struct PointerQuery {
    entity: Entity,
    node: &'static Node,
    global_transform: &'static GlobalTransform,
    filter: Option<&'static ActivePointerFilter>,
    calculated_clip: Option<&'static CalculatedClip>,
    view_visibility: Option<&'static ViewVisibility>,
    target_camera: Option<&'static TargetCamera>,
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
    camera_query: Query<(Entity, &Camera)>,
    default_ui_camera: DefaultUiCamera,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    windows: Query<&Window>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    touches_input: Res<Touches>,
    ui_scale: Res<UiScale>,
    ui_stack: Res<UiStack>,
    time: Res<Time>,
    pointer_query: Query<PointerQuery>,
    mut events: EventWriter<PointerInput>,
) {
    let mouse_released =
        mouse_button_input.just_released(MouseButton::Left) || touches_input.any_just_released();
    let mouse_clicked =
        mouse_button_input.just_pressed(MouseButton::Left) || touches_input.any_just_pressed();

    let primary_window = primary_window.iter().next();

    let camera_cursor_positions: HashMap<Entity, Vec2> = camera_query
        .iter()
        .filter_map(|(entity, camera)| {
            let Some(NormalizedRenderTarget::Window(window_ref)) =
                camera.target.normalize(primary_window)
            else {
                return None;
            };

            let viewport_position = camera
                .logical_viewport_rect()
                .map(|rect| rect.min)
                .unwrap_or_default();

            windows
                .get(window_ref.entity())
                .ok()
                .and_then(|window| window.cursor_position())
                .or_else(|| touches_input.first_pressed_position())
                .map(|cursor_position| (entity, cursor_position - viewport_position))
        })
        .map(|(entity, cursor_position)| (entity, cursor_position / ui_scale.0))
        .collect();

    let first_camera_cursor_position = camera_cursor_positions
        .iter()
        .next()
        .map(|position| *position.1);

    if mouse_clicked {
        state.press_position = first_camera_cursor_position;
        state.drag_in_seconds = Some(0.5);
    }
    let delta = match (first_camera_cursor_position, state.last_cursor_position) {
        (Some(c), Some(l)) => c - l,
        _ => Vec2::ZERO,
    };

    state.last_cursor_position = first_camera_cursor_position;
    let mut moused_over_nodes = ui_stack
        .uinodes
        .iter()
        // reverse the iterator to traverse the tree from closest nodes to furthest
        .rev()
        .filter_map(|entity| {
            let Ok(node) = pointer_query.get(*entity) else {
                return None;
            };

            let view_visibility = node.view_visibility?;

            if !view_visibility.get() {
                return None;
            }

            let camera_entity = node
                .target_camera
                .map(TargetCamera::entity)
                .or(default_ui_camera.get())?;

            let node_rect = node.node.logical_rect(node.global_transform);

            let visible_rect = node
                .calculated_clip
                .map(|clip| node_rect.intersect(clip.clip))
                .unwrap_or(node_rect);

            let relative_cursor_position = camera_cursor_positions
                .get(&camera_entity)
                .map(|cursor_position| (*cursor_position - node_rect.min) / node_rect.size());

            let normalized_visible_node_rect = visible_rect.normalize(node_rect);

            let contains_cursor = relative_cursor_position
                .map(|position| normalized_visible_node_rect.contains(position))
                .unwrap_or(false);

            if contains_cursor {
                Some(*entity)
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
    let send_drag_stop = state.dragging && mouse_released;
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

        if mouse_clicked {
            state.pressed_entities.push(entity);
            down_entities.push(entity);
        }
        if mouse_released {
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

    let Some(pos) = first_camera_cursor_position else {
        return;
    };
    if !down_entities.is_empty() {
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

    if mouse_released {
        state.pressed_entities.clear();
        state.dragging_from.clear();
        state.press_position = None;
        state.dragging = false;
    }
}
