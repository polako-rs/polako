use bevy::{prelude::{DerefMut, Deref}, math::Vec2, ecs::{component::Component, query::{WorldQuery, Without, With, Changed, Or}, system::{Query, Resource, Commands, ResMut, Local, Res}, entity::{Entity, Entities}, removal_detection::RemovedComponents}, hierarchy::{Children, Parent}, utils::{HashMap, HashSet}, window::{Window, PrimaryWindow}, log::warn};
use polako_channel::Channel;
// use taffy::

#[derive(Component)]
pub enum StyleProp<T> {
    Undefined(T),
    Defined(T),
    Managed(T),
}

pub trait StylePropData: Default + Send + Sync + ApplyChange + 'static { }
impl<T: Default + Send + Sync + ApplyChange + 'static> StylePropData for T { }

#[derive(Component)]
pub struct Inline;

pub enum StylePropValue<'p, T> {
    Ref(&'p StyleProp<T>),
    Val(StyleProp<T>),
}


impl<'p, T> std::ops::Deref for StylePropValue<'p, T> {
    type Target = StyleProp<T>;
    fn deref(&self) -> &Self::Target {
        match self {
            StylePropValue::Ref(prop) => prop,
            StylePropValue::Val(prop) => prop,
        }
    }
}

pub struct StyleChange(Box<dyn FnOnce(&mut taffy::style::Style)>);
unsafe impl Send for StyleChange { }
unsafe impl Sync for StyleChange { }
// pub struct StyleChanges(Vec<StyleChange>)

#[derive(Component)]
pub enum Owner {
    Undefined,
    Direct(Entity),
    Page(Entity),
    Space(Entity),
}

impl Owner {
    pub fn space(&mut self, space: Entity) {
        *self = Owner::Space(space)
    }
    pub fn page(&mut self, page: Entity) {
        *self = Owner::Page(page)
    }
    pub fn direct(&mut self, parent: Entity) {
        *self = Owner::Direct(parent)
    }
    pub fn undefined(&mut self) {
        *self = Owner::Undefined;
    }
}

pub trait ApplyChange {
    fn apply_change(&self) -> StyleChange;
}

pub trait AsStylePropValue<T> {
    fn as_style_prop(&self) -> StylePropValue<T>;
}

impl<'a, T: Default> AsStylePropValue<T> for Option<&'a StyleProp<T>> {
    fn as_style_prop(&self) -> StylePropValue<T> {
        match self {
            Some(prop) => {
                StylePropValue::Ref(prop)
            },
            None => StylePropValue::Val(StyleProp::Undefined(T::default()))
        }
    }
}

impl<T> std::ops::Deref for StyleProp<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match self {
            StyleProp::Undefined(p) => p,
            StyleProp::Defined(p) => p,
            StyleProp::Managed(p) => p,
        }
    }
}

#[derive(WorldQuery)]
pub struct StyleProps {
    container: &'static StyleProp<Container>,
    padding: Option<&'static StyleProp<Padding>>,
    width: Option<&'static StyleProp<Width>>,
    height: Option<&'static StyleProp<Height>>,
}


pub enum Container {
    Stack,
    Flex,
    Space(TargetSpace),
    Page,
}

impl Container {
    pub fn is_stack(&self) -> bool {
        matches!(self, Container::Stack)
    }
}

pub enum TargetSpace {
    Nearest,
    Exact(Entity),
}

#[derive(Component)]
pub struct Space {
    pub width: u16,
    pub height: u16,
}

#[derive(Component)]
pub struct ComputedSpace(Entity);

pub fn update_windows_space(
    windows: Query<(Entity, &Window), Changed<Window>>,
    mut spaces: Query<&mut Space>,
    mut commands: Commands,
) {
    for (entity, window) in &windows {
        let width = window.resolution.physical_height() as u16;
        let height = window.resolution.physical_height() as u16;
        if let Ok(mut space) = spaces.get_mut(entity) {
            if space.width != width {
                space.width = width;
            }
            if space.height != height {
                space.height = height;
            }
        } else {
            commands.entity(entity).insert(Space { width, height });
        }
    }
}

#[derive(Deref, DerefMut)]
pub struct Padding(UiRect);
impl Padding {
    pub fn all(size: Val) -> Self {
        Padding(UiRect { left: size, right: size, top: size, bottom: size })
    }
}
impl Default for Padding {
    fn default() -> Self {
        Padding::all(Val::Px(0))
    }
}
impl ApplyChange for Padding {
    fn apply_change(&self) -> StyleChange {
        let padding = self.0.clone();
        StyleChange(Box::new(move |style| {
            style.padding.left = padding.left.as_length_percentage();
            style.padding.right = padding.right.as_length_percentage();
            style.padding.top = padding.top.as_length_percentage();
            style.padding.bottom = padding.bottom.as_length_percentage();
        }))
    }
}


#[derive(Clone, Copy, Deref, DerefMut)]
pub struct Width(Val);
impl Default for Width {
    fn default() -> Self {
        Width(Val::Auto)
    }
}

impl ApplyChange for Width {
    fn apply_change(&self) -> StyleChange {
        let width = self.0.clone();
        StyleChange(Box::new(move |style| {
            style.size.width = width.as_dimension();
        }))
    }
}

#[derive(Clone, Copy, Deref, DerefMut)]
pub struct Height(Val);
impl Default for Height {
    fn default() -> Self {
        Height(Val::Auto)
    }
}

impl ApplyChange for Height {
    fn apply_change(&self) -> StyleChange {
        let height = self.0.clone();
        StyleChange(Box::new(move |style| {
            style.size.height = height.as_dimension();
        }))
    }
}


#[derive(Clone, Copy, Debug)]
pub enum Val {
    Auto,
    Px(i32),
    Percent(f32)
}

impl Val {
    pub fn as_length_percentage(&self) -> taffy::style::LengthPercentage {
        match self {
            Val::Auto => taffy::style::LengthPercentage::Points(0.),
            Val::Percent(p) => taffy::style::LengthPercentage::Percent(*p),
            Val::Px(p) => taffy::style::LengthPercentage::Points(*p as f32),
        }
    }
    pub fn as_dimension(&self) -> taffy::style::Dimension {
        match self {
            Val::Auto => taffy::style::Dimension::Auto,
            Val::Percent(p) => taffy::style::Dimension::Percent(*p),
            Val::Px(p) => taffy::style::Dimension::Points(*p as f32),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct UiRect {
    pub left: Val,
    pub right: Val,
    pub top: Val,
    pub bottom: Val,
}

#[derive(Clone, Copy, Debug)]
pub struct UiSize {
    pub width: Val,
    pub height: Val,
}

#[derive(Component)]
pub struct Node {
    pub size: Vec2
}

#[derive(Resource)]
pub struct UiSurface {
    tree: taffy::Taffy,
    entity_to_taffy: HashMap<Entity, taffy::node::Node>,
}

impl UiSurface {
    pub fn update_space(&mut self, entity: Entity, space: &Space) {
        let style = taffy::style::Style {
            size: taffy::geometry::Size::from_points(space.width as f32, space.height as f32),
            ..Default::default()
        };
        if let Some(node) = self.entity_to_taffy.get(&entity) {
            self.tree.set_style(*node, style).unwrap();
        } else {
            let node = self.tree.new_leaf(style).unwrap();
            self.entity_to_taffy.insert(entity, node);
        }
    }

    pub fn edit_style<F: FnOnce(&mut taffy::style::Style)>(&mut self, entity: Entity, edit: F) {
        if let Some(node) = self.entity_to_taffy.get(&entity) {
            let mut style = self.tree.style(*node).unwrap().clone();
            edit(&mut style);
            self.tree.set_style(*node, style);
        } else {
            let mut style = taffy::style::Style::default();
            edit(&mut style);
            let node = self.tree.new_leaf(style).unwrap();
            self.entity_to_taffy.insert(entity, node);
        }
    }

    pub fn compute(&mut self) {
        let node = self.tree.new_leaf(taffy::style::Style::default()).unwrap();
        let x = self.tree.layout(node).unwrap();
    }
}

impl Default for UiSurface {
    fn default() -> Self {
        UiSurface {
            tree: taffy::Taffy::new(),
            entity_to_taffy: HashMap::new(),
        }
    }
}

pub fn collect_prop_changes<T: StylePropData>(
    changed: Query<(Entity, &StyleProp<T>), Changed<StyleProp<T>>>,
    changes: Res<Channel<(Entity, StyleChange)>>,
) {
    for (entity, prop) in &changed {
        changes.send((entity, prop.apply_change()))
    }
}

pub fn collect_prop_removals<T: StylePropData>(
    mut removed: RemovedComponents<StyleProp<T>>,
    entities: Entities,
    changes: Res<Channel<(Entity, StyleChange)>>,
) {
    for entity in removed.read() {
        if entities.contains(entity) {
            let change = T::default().apply_change();
            changes.send((entity, change))
        }
    }
}

pub fn apply_changes(
    mut changes: ResMut<Channel<(Entity, StyleChange)>>,
    mut changed_entities: Local<HashMap<Entity, Vec<StyleChange>>>,
    mut surface: ResMut<UiSurface>,
) {
    changes.consume(|(entity, change)| {
        changed_entities.entry(entity).or_default().push(change)
    });
    for (entity, changes) in changed_entities.drain() {
        surface.edit_style(entity, move |style| {
            for change in changes {
                change.0(style)
            }
        })
    }
}

pub fn compute_owners(
    space_tree: Query<(Option<&Parent>, Option<&Space>)>,
    page_tree: Query<(Option<&Parent>, &StyleProp<Container>)>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    mut owners: Query<&mut Owner>,
    changed: Query<
        (Entity, Option<&Parent>, &StyleProp<Container>, Option<&Inline>),
        Or<(Changed<Parent>, Changed<StyleProp<Container>>)>
    >
) {
    fn nearest_space(
        this: Entity,
        primary_window: Entity,
        tree: &Query<(Option<&Parent>, Option<&Space>)>
    ) -> Entity {
        let (parent, space) = tree.get(this).unwrap();
        if space.is_some() {
            this
        } else if parent.is_none() {
            primary_window
        } else {
            nearest_space(parent.unwrap().get(), primary_window, tree)
        }
    }
    fn nearest_page(
        this: Entity,
        tree: &Query<(Option<&Parent>, &StyleProp<Container>)>
    ) -> Entity {
        let (parent, cnt) = tree.get(this).unwrap();
        if matches!(**cnt, Container::Page) {
            this
        } else if let Some(parent) = parent {
            nearest_page(parent.get(), tree)
        } else {
            panic!("Can't find nearest page.")
        }
    }
    let primary_window = primary_window.single();
    for (entity, parent, container, inline) in &changed {
        match **container {
            Container::Space(TargetSpace::Nearest) => {
                owners.get_mut(entity).unwrap().space(nearest_space(entity, primary_window, &space_tree))
            },
            Container::Space(TargetSpace::Exact(space)) => {
                owners.get_mut(entity).unwrap().space(space)
            },
            _ if inline.is_some() => {
                owners.get_mut(entity).unwrap().page(nearest_page(entity, &page_tree))
            },
            _ if parent.is_some() => {
                owners.get_mut(entity).unwrap().direct(parent.unwrap().get())
            },
            _ => {
                warn!("Can't detect owner");
                owners.get_mut(entity).unwrap().undefined();
            }
        }
    }
}

pub fn compute_spaces_ineffective(
    q_primary_window: Query<Entity, With<PrimaryWindow>>,
    q_roots: Query<(Entity, &StyleProp<Container>), Without<Parent>>,
    q_containers: Query<&StyleProp<Container>>,
    mut q_computed: Query<&mut ComputedSpace>,
    q_children: Query<&Children>,
    mut commands: Commands,
) {
    let primary = q_primary_window.single();
    fn set_space(
        entity: Entity,
        space: Entity,
        containers: &Query<&StyleProp<Container>>,
        computed: &mut Query<&mut ComputedSpace>,
        children: &Query<&Children>,
        commands: &mut Commands,
    ) {
        if let Ok(mut computed) = computed.get_mut(entity) {
            if computed.0 != space {
                computed.0 = space;
            }
        } else {
            commands.entity(entity).insert(ComputedSpace(space));
        }
        let Ok(container) = containers.get(entity) else {
            return;
        };
        let space = match **container {
            Container::Space(TargetSpace::Exact(space)) => space,
            _ => space
        };
        if let Ok(entities) = children.get(entity) {
            for child in entities.iter() {
                set_space(*child, space, containers, computed, children, commands)
            }
        }
    }
    for (entity, root) in &q_roots {
        let space = match **root {
            Container::Space(TargetSpace::Exact(entity)) => entity,
            _ => primary
        };
        set_space(entity, space, &q_containers, &mut q_computed, &q_children, &mut commands)
    }

}

pub fn process_spaces(
    q_spaces: Query<(Entity, &Space), Changed<Space>>,
    mut surface: ResMut<UiSurface>    
) {
    for (entity, space) in &q_spaces {
        surface.update_space(entity, space)
    }
}


pub fn process_ui(
    q_roots: Query<(Entity, &Children), (With<Node>, Without<Parent>)>,
    q_children: Query<&Children>,
    q_props: Query<StyleProps>,
) {
    // for (root, children) in &q_roots {
    //     let props = q_props.get(root).unwrap();
    //     let padding = props.padding.as_style_prop();
    //     let l = taffy::tree::layout::Layout::new();
    //     // l.
        
    // }

}