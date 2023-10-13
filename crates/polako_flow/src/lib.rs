use std::{any::TypeId, cell::RefCell, ops::DerefMut, rc::Rc, sync::RwLock, thread::ThreadId};

use bevy::{
    ecs::system::{Command, SystemBuffer},
    prelude::*,
    utils::{HashMap, HashSet},
};
use polako_constructivism::*;

pub struct FlowPlugin;
impl Plugin for FlowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, flow_system);
        app.insert_resource(FlowResource::new());
        app.insert_resource(FlowLoop::default());
        app.insert_resource(BindTargets::new());
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum FlowSet {
    CleanupChanges,
    Cleanup,
    Read,
    Write,
}

pub fn flow_system(world: &mut World) {
    let flow = world.resource::<FlowResource>().clone();
    loop {
        let mut schedule_ref = flow.schdule.borrow_mut();
        for add_systems in flow.queue.take() {
            add_systems(schedule_ref.deref_mut());
        }
        for command in flow.commands.take() {
            command(world);
        }
        schedule_ref.run(world);
        if world.resource_mut::<FlowLoop>().should_repeat() {
            continue;
        } else {
            break;
        }
    }
}

fn cleanup_changes<T: Component, V: Bindable>(changes: Changes<T, V>) {
    changes.clear();
}

pub fn cleanup_component_readers<S: Component, T: Component, V: Bindable>(
    mut sources: Query<&mut ComponentBindSources<S, T, V>>,
    mut targets: ResMut<BindTargets>,
    mut removals: RemovedComponents<BindTarget>,
) {
    for target in removals.iter() {
        for source in targets.0.remove(&target).unwrap_or_default().iter() {
            if let Ok(mut source) = sources.get_mut(*source) {
                source.0.remove(&target);
            }
        }
    }
}

pub fn read_component_changes<S: Component, T: Component, V: Bindable>(
    components: Query<(&ComponentBindSources<S, T, V>, &S), Changed<S>>,
    changes: Changes<T, V>,
) {
    for (sources, component) in components.iter() {
        for items in sources.0.values() {
            for source in items.iter() {
                let value = source.read.read(&component).get();
                changes.send(ApplyChange {
                    value,
                    target: source.target,
                    writer: source.writer.clone(),
                })
            }
        }
    }
}

pub fn cleanup_resource_readers<S: Resource, T: Component, V: Bindable>(
    mut sources: ResMut<ResourceBindSources<S, T, V>>,
    mut removals: RemovedComponents<BindTarget>,
) {
    for target in removals.iter() {
        sources.0.remove(&target);
    }
}

pub fn read_resource_changes<S: Resource, T: Component, V: Bindable>(
    res: Res<S>,
    sources: Res<ResourceBindSources<S, T, V>>,
    changes: Changes<T, V>,
) {
    if res.is_changed() {
        for sources in sources.0.values() {
            for source in sources.iter() {
                let value = source.read.read(&res).get();
                changes.send(ApplyChange {
                    value,
                    target: source.target,
                    writer: source.writer.clone(),
                })
            }
        }
    }
}
pub fn write_component_changes<T: Component, V: Bindable>(
    mut components: Query<&mut T>,
    mut flow: Deferred<FlowControl>,
    changes: Changes<T, V>,
) {
    changes.recv(|change| {
        let Ok(mut component) = components.get_mut(change.target) else {
            return;
        };
        if change.writer.read(component.as_ref()).as_ref() == &change.value {
            return;
        }
        change
            .writer
            .write(component.as_mut(), change.value.clone());
        flow.repeat();
    });
}

#[derive(Resource, Default)]
struct FlowLoop(bool);
impl FlowLoop {
    fn repeat(&mut self) {
        self.0 = true;
    }
    fn should_repeat(&mut self) -> bool {
        let retry = self.0;
        self.0 = false;
        retry
    }
}

#[derive(Default)]
pub struct FlowControl(bool);
impl FlowControl {
    pub fn repeat(&mut self) {
        self.0 = true;
    }
}
impl SystemBuffer for FlowControl {
    fn apply(&mut self, _: &bevy::ecs::system::SystemMeta, world: &mut World) {
        if self.0 {
            self.0 = false;
            world.resource_mut::<FlowLoop>().repeat();
        }
    }
}

#[derive(Resource, Deref, Clone)]
pub struct FlowResource(Rc<Flow>);
unsafe impl Send for FlowResource {}
unsafe impl Sync for FlowResource {}
impl FlowResource {
    pub fn new() -> Self {
        FlowResource(Rc::new(Flow::new()))
    }
}

pub struct Flow {
    schdule: RefCell<Schedule>,
    queue: RefCell<Vec<Box<dyn FnOnce(&mut Schedule)>>>,
    commands: RefCell<Vec<Box<dyn FnOnce(&mut World)>>>,

    registered_cleanup_changes_systems: RefCell<HashSet<(TypeId, TypeId)>>,
    registered_read_component_systems: RefCell<HashSet<(TypeId, TypeId, TypeId)>>,
    registered_read_resource_systems: RefCell<HashSet<(TypeId, TypeId, TypeId)>>,
    registered_wite_systems: RefCell<HashSet<(TypeId, TypeId)>>,
}

impl Flow {
    pub fn new() -> Self {
        let mut schedule = Schedule::new();
        schedule.configure_sets((
            FlowSet::Cleanup.after(FlowSet::CleanupChanges),
            FlowSet::Read.after(FlowSet::Cleanup),
            FlowSet::Write.after(FlowSet::Read),
        ));
        // schedule.
        Self {
            schdule: RefCell::new(schedule),
            queue: RefCell::new(vec![]),
            commands: RefCell::new(vec![]),

            registered_cleanup_changes_systems: RefCell::new(HashSet::new()),
            registered_read_component_systems: RefCell::new(HashSet::new()),
            registered_read_resource_systems: RefCell::new(HashSet::new()),
            registered_wite_systems: RefCell::new(HashSet::new()),
        }
    }

    pub fn register_component_read_systems<S: Component, T: Component, V: Bindable>(&self) {
        let source_type = TypeId::of::<S>();
        let target_type = TypeId::of::<T>();
        let value_type = TypeId::of::<V>();
        if !self
            .registered_cleanup_changes_systems
            .borrow()
            .contains(&(target_type, value_type))
        {
            self.registered_cleanup_changes_systems
                .borrow_mut()
                .insert((target_type, value_type));
            self.queue.borrow_mut().push(Box::new(|schedule| {
                schedule.add_systems(cleanup_changes::<T, V>.in_set(FlowSet::CleanupChanges));
            }))
        }
        if self.registered_read_component_systems.borrow().contains(&(
            source_type,
            target_type,
            value_type,
        )) {
            return;
        }
        self.registered_read_component_systems.borrow_mut().insert((
            source_type,
            target_type,
            value_type,
        ));
        self.queue.borrow_mut().push(Box::new(|schedule| {
            schedule.add_systems(cleanup_component_readers::<S, T, V>.in_set(FlowSet::Cleanup));
            schedule.add_systems(read_component_changes::<S, T, V>.in_set(FlowSet::Read));
        }));
        // self.commands.borrow_mut().push(Box::new(|world| {

        // }))
    }

    pub fn register_resource_read_systems<S: Resource, T: Component, V: Bindable>(&self) {
        let source_type = TypeId::of::<S>();
        let target_type = TypeId::of::<T>();
        let value_type = TypeId::of::<V>();
        if !self
            .registered_cleanup_changes_systems
            .borrow()
            .contains(&(target_type, value_type))
        {
            self.registered_cleanup_changes_systems
                .borrow_mut()
                .insert((target_type, value_type));
            self.queue.borrow_mut().push(Box::new(|schedule| {
                schedule.add_systems(cleanup_changes::<T, V>.in_set(FlowSet::CleanupChanges));
            }))
        }
        if self.registered_read_resource_systems.borrow().contains(&(
            source_type,
            target_type,
            value_type,
        )) {
            return;
        }
        self.registered_read_resource_systems.borrow_mut().insert((
            source_type,
            target_type,
            value_type,
        ));
        self.queue.borrow_mut().push(Box::new(|schedule| {
            schedule.add_systems(cleanup_resource_readers::<S, T, V>.in_set(FlowSet::Cleanup));
            schedule.add_systems(read_resource_changes::<S, T, V>.in_set(FlowSet::Read));
        }));
    }

    pub fn register_component_write_systems<T: Component, V: Bindable>(&self) {
        let host_type = TypeId::of::<T>();
        let value_type = TypeId::of::<V>();
        if self
            .registered_wite_systems
            .borrow()
            .contains(&(host_type, value_type))
        {
            return;
        }
        self.registered_wite_systems
            .borrow_mut()
            .insert((host_type, value_type));
        self.queue.borrow_mut().push(Box::new(|schedule| {
            schedule.add_systems(write_component_changes::<T, V>.in_set(FlowSet::Write));
        }));
        self.commands.borrow_mut().push(Box::new(|world| {
            world.insert_resource(Channel::<ApplyChange<T, V>>::new());
        }));
    }
}

pub trait WorldFlow {
    fn bind_component_to_component<S: Component, T: Component, V: Bindable>(
        &mut self,
        from: ComponentReader<S, V>,
        to: ComponentWriter<T, V>,
    );

    fn bind_resource_to_component<R: Resource, T: Component, V: Bindable>(
        &mut self,
        from: Reader<R, V>,
        to: ComponentWriter<T, V>,
    );
}
impl WorldFlow for World {
    fn bind_component_to_component<S: Component, T: Component, V: Bindable>(
        &mut self,
        from: ComponentReader<S, V>,
        to: ComponentWriter<T, V>,
    ) {
        // setup source

        // this component will be added to `from.entity`, all required generic systems will be
        // added to the `Flow` if needed
        let bind_source = BindSource {
            target: to.entity,
            read: from.reader,
            writer: to.writer,
            // maybe_changed: Box::new(move |cmd, value| {
            //     let target = to.entity;
            //     let prop = to.writer.clone();
            //     cmd.add(move |w: &mut World| {
            //         w.get_resource_or_insert_with(Events::<ApplyChange<T, V>>::default)
            //             .send(ApplyChange {
            //                 writer: prop,
            //                 target,
            //                 value,
            //             })
            //     })
            // }),
        };
        {
            let mut e = self.entity_mut(from.entity);
            if !e.contains::<ComponentBindSources<S, T, V>>() {
                e.insert(ComponentBindSources::<S, T, V>(HashMap::new()));
            }
            e.get_mut::<ComponentBindSources<S, T, V>>()
                .unwrap()
                .0
                .entry(to.entity)
                .or_default()
                .push(bind_source);
        }

        // setup target
        self.entity_mut(to.entity).insert(BindTarget);
        self.resource_mut::<BindTargets>()
            .0
            .entry(to.entity)
            .or_default()
            .insert(from.entity);

        let flow = self.resource::<FlowResource>().clone();
        flow.register_component_read_systems::<S, T, V>();
        flow.register_component_write_systems::<T, V>();
    }

    fn bind_resource_to_component<S: Resource, T: Component, V: Bindable>(
        &mut self,
        from: Reader<S, V>,
        to: ComponentWriter<T, V>,
    ) {
        let bind_source = BindSource {
            target: to.entity,
            read: from,
            writer: to.writer,
            // maybe_changed: Box::new(move |cmd, value| {
            //     let target = to.entity;
            //     let prop = to.writer.clone();
            //     cmd.add(move |w: &mut World| {
            //         w.get_resource_or_insert_with(Events::<ApplyChange<T, V>>::default)
            //             .send(ApplyChange {
            //                 writer: prop,
            //                 target,
            //                 value,
            //             })
            //     })
            // }),
        };

        self.entity_mut(to.entity).insert(BindTarget);
        self.get_resource_or_insert_with(ResourceBindSources::<S, T, V>::new)
            .0
            .entry(to.entity)
            .or_default()
            .push(bind_source);

        let flow = self.resource::<FlowResource>().clone();
        flow.register_resource_read_systems::<S, T, V>();
        flow.register_component_write_systems::<T, V>();
    }
}

pub trait Bindable: Send + Sync + Clone + std::fmt::Debug + PartialEq + 'static {}
impl<T: Send + Sync + Clone + PartialEq + std::fmt::Debug + 'static> Bindable for T {}

pub type Changes<'w, T, V> = Res<'w, Channel<ApplyChange<T, V>>>;
#[derive(Resource)]
pub struct Channel<T>(RwLock<HashMap<ThreadId, Rc<RefCell<Vec<T>>>>>);
unsafe impl<T> Send for Channel<T> {}
unsafe impl<T> Sync for Channel<T> {}
impl<T> Channel<T> {
    pub fn new() -> Self {
        Self(RwLock::new(HashMap::new()))
    }
    pub fn clear(&self) {
        for cell in self.0.read().unwrap().values() {
            cell.borrow_mut().clear()
        }
    }
    pub fn send(&self, event: T) {
        let id = std::thread::current().id();
        {
            let read = self.0.read().unwrap();
            let item = read.get(&id);
            if let Some(events) = item {
                events.borrow_mut().push(event);
                return;
            }
        }
        {
            self.0
                .write()
                .unwrap()
                .insert(id, Rc::new(RefCell::new(vec![event])));
        }
    }
    pub fn recv<F: FnMut(&T)>(&self, mut recv: F) {
        for cell in self.0.read().unwrap().values() {
            let borrow = cell.borrow();
            for item in borrow.iter() {
                recv(item)
            }
        }
    }
}

pub struct BindSource<S, T, V: Bindable> {
    target: Entity,
    read: Reader<S, V>,
    writer: Writer<T, V>,
}
#[derive(Component)]
pub struct ComponentBindSources<S: Component, T: Component, V: Bindable>(
    HashMap<Entity, Vec<BindSource<S, T, V>>>,
);

#[derive(Resource)]
pub struct ResourceBindSources<S: Resource, T: Component, V: Bindable>(
    HashMap<Entity, Vec<BindSource<S, T, V>>>,
);
impl<S: Resource, T: Component, V: Bindable> ResourceBindSources<S, T, V> {
    fn new() -> Self {
        Self(HashMap::new())
    }
}

#[derive(Component)]
pub struct BindTarget;

#[derive(Resource)]
pub struct BindTargets(HashMap<Entity, HashSet<Entity>>);
impl BindTargets {
    pub fn new() -> Self {
        BindTargets(HashMap::new())
    }
}

// pub struct HandleChange<W: W V: Bindable> {
//     handler:
// }

#[derive(Event)]
pub struct ApplyChange<H: Component, V: Bindable> {
    target: Entity,
    writer: Writer<H, V>,
    value: V,
}

pub trait EntityProp<H: Component, V: Bindable> {
    fn get(&self, value: impl Into<Reader<H, V>>) -> ComponentReader<H, V>;
    fn set(&self, value: impl Into<Writer<H, V>>) -> ComponentWriter<H, V>;
}

impl<H: Component, V: Bindable> EntityProp<H, V> for Entity {
    fn get(&self, value: impl Into<Reader<H, V>>) -> ComponentReader<H, V> {
        ComponentReader {
            entity: self.clone(),
            reader: value.into(),
        }
    }
    fn set(&self, value: impl Into<Writer<H, V>>) -> ComponentWriter<H, V> {
        ComponentWriter {
            entity: self.clone(),
            writer: value.into(),
        }
    }
}

pub struct BindComponentToComponent<S: Component, T: Component, V: Bindable> {
    pub from: ComponentReader<S, V>,
    pub to: ComponentWriter<T, V>,
}

impl<S: Component, T: Component, V: Bindable> Command for BindComponentToComponent<S, T, V> {
    fn apply(self, world: &mut World) {
        world.bind_component_to_component(self.from, self.to)
    }
}

pub struct BindResourceToComponent<S: Resource, T: Component, V: Bindable> {
    pub from: Reader<S, V>,
    pub to: ComponentWriter<T, V>,
}

impl<S: Resource, T: Component, V: Bindable> Command for BindResourceToComponent<S, T, V> {
    fn apply(self, world: &mut World) {
        world.bind_resource_to_component(self.from, self.to);
    }
}

pub struct ComponentReader<C: Component, V: Bindable> {
    entity: Entity,
    reader: Reader<C, V>,
}

pub enum Reader<H, V: Bindable> {
    Func(fn(&H) -> Value<V>),
    Closure(Rc<dyn Fn(&H) -> Value<V>>),
}

unsafe impl<H, V: Bindable> Send for Reader<H, V> {}
unsafe impl<H, V: Bindable> Sync for Reader<H, V> {}
impl<H, V: Bindable> Clone for Reader<H, V> {
    fn clone(&self) -> Self {
        match self {
            Self::Func(f) => Self::Func(f.clone()),
            Self::Closure(c) => Self::Closure(c.clone()),
        }
    }
}

impl<H, V: Bindable> From<Prop<H, V>> for Reader<H, V> {
    fn from(value: Prop<H, V>) -> Self {
        Reader::Func(value.getter())
    }
}
impl<H, V: Bindable> Reader<H, V> {
    pub fn read<'a>(&self, host: &'a H) -> Value<'a, V> {
        match self {
            Reader::Func(f) => f(host),
            Reader::Closure(c) => c(host),
        }
    }
}

pub struct ComponentWriter<C: Component, V: Bindable> {
    entity: Entity,
    writer: Writer<C, V>,
}

pub struct Writer<H, V> {
    get: fn(&H) -> Value<V>,
    set: fn(&mut H, V),
}
impl<H, V: Bindable> Writer<H, V> {
    pub fn read<'a>(&self, host: &'a H) -> Value<'a, V> {
        (self.get)(host)
    }
    pub fn write(&self, host: &mut H, value: V) {
        (self.set)(host, value)
    }
}
unsafe impl<H, V: Bindable> Send for Writer<H, V> {}
unsafe impl<H, V: Bindable> Sync for Writer<H, V> {}
impl<H, V: Bindable> Clone for Writer<H, V> {
    fn clone(&self) -> Self {
        Self {
            get: self.get.clone(),
            set: self.set.clone(),
        }
    }
}
impl<H, T: Bindable> From<Prop<H, T>> for Writer<H, T> {
    fn from(prop: Prop<H, T>) -> Self {
        Writer {
            get: prop.getter(),
            set: prop.setter(),
        }
    }
}

pub trait MapProp<H, V: Bindable> {
    fn map<F: Fn(&V) -> T + 'static, T: Bindable>(self, map: F) -> Reader<H, T>;
}

impl<H: 'static, V: Bindable> MapProp<H, V> for Prop<H, V> {
    fn map<F: Fn(&V) -> T + 'static, T: Bindable>(self, map: F) -> Reader<H, T> {
        Reader::Closure(Rc::new(move |host| {
            let val = self.get(host);
            Value::Val(map(val.as_ref()))
        }))
    }
}
