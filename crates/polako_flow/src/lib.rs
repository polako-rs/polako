use std::{any::TypeId, cell::RefCell, rc::Rc, ops::DerefMut};

use bevy::{prelude::*, utils::{HashMap, HashSet}, ecs::system::{SystemBuffer, Command}};
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

pub fn cleanup_component_readers<H: Component, T: Bindable>(
    mut sources: Query<&mut BindSources<H, T>>,
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

pub fn read_component_changes<H: Component, T: Bindable>(
    components: Query<(&BindSources<H, T>, &H), Changed<H>>,
    mut commands: Commands,
) {
    for (sources, component) in components.iter() {
        for items in sources.0.values() {
            for source in items.iter() {
                let value = source.read.read(&component).get();
                (source.notify_changed)(&mut commands, value)
            }
        }
    }
}

pub fn write_component_changes<H: Component, V: Bindable>(
    mut components: Query<&mut H>,
    mut changes: EventReader<ApplyChange<H, V>>,
    mut flow: Deferred<FlowControl>,
) {
    for change in changes.iter() {
        let Ok(mut component) = components.get_mut(change.target) else {
            continue;
        };
        if change.writer.read(component.as_ref()).as_ref() == &change.value {
            continue;
        }
        change.writer.write(component.as_mut(), change.value.clone());
        flow.repeat();
    }
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
unsafe impl Send for FlowResource { }
unsafe impl Sync for FlowResource { }
impl FlowResource {
    pub fn new() -> Self {
        FlowResource(Rc::new(Flow::new()))
    }
}

pub struct Flow {
    schdule: RefCell<Schedule>,
    queue: RefCell<Vec<Box<dyn FnOnce(&mut Schedule)>>>,
    commands: RefCell<Vec<Box<dyn FnOnce(&mut World)>>>,
    
    registered_read_systems: RefCell<HashSet<(TypeId, TypeId)>>,
    registered_wite_systems: RefCell<HashSet<(TypeId, TypeId)>>,
}


impl Flow {
    pub fn new() -> Self {
        Self {
            schdule: RefCell::new(Schedule::new()),
            queue: RefCell::new(vec![]),
            commands: RefCell::new(vec![]),

            registered_read_systems: RefCell::new(HashSet::new()),
            registered_wite_systems: RefCell::new(HashSet::new()),
        }
    }



    pub fn register_component_read_systems<H: Component, V: Bindable>(
        &self
    ) {
        let host_type = TypeId::of::<H>();
        let value_type = TypeId::of::<V>();
        if self.registered_read_systems.borrow().contains(&(host_type, value_type)) {
            return;
        }
        self.registered_read_systems.borrow_mut().insert((host_type, value_type));
        self.queue.borrow_mut().push(Box::new(|schedule| {
            schedule.add_systems(cleanup_component_readers::<H, V>);
            schedule.add_systems(read_component_changes::<H, V>
                .after(cleanup_component_readers::<H, V>)
            );
        }));
    }

    pub fn register_component_write_systems<H: Component, V: Bindable>(
        &self
    ) {
        let host_type = TypeId::of::<H>();
        let value_type = TypeId::of::<V>();
        if self.registered_wite_systems.borrow().contains(&(host_type, value_type)) {
            return;
        }
        self.registered_wite_systems.borrow_mut().insert((host_type, value_type));
        self.queue.borrow_mut().push(Box::new(|schedule| {
            schedule.add_systems(write_component_changes::<H, V>);
        }));
        self.commands.borrow_mut().push(Box::new(|world| {
            world.insert_resource(Events::<ApplyChange<H, V>>::default());
        }));
    }

}

pub trait WorldFlow {
    fn bind_component_to_component<
        S: Component,
        T: Component,
        V: Send + Sync + Clone + PartialEq + 'static,
    >(
        &mut self,
        from: ComponentReader<S, V>,
        to: ComponentWriter<T, V>,
    );
}
impl WorldFlow for World {
    fn bind_component_to_component<
        S: Component,
        T: Component,
        V: Send + Sync + Clone + PartialEq + 'static,
    >(
        &mut self,
        from: ComponentReader<S, V>,
        to: ComponentWriter<T, V>,
    ) {
        // let to: Writer<T, V> = to.writer;
        // let from: Reader<S, V> = from.reader;
        // setup source

        // this component will be added to `from.entity`, all required generic systems will be
        // added to the `Flow` if needed
        let bind_source = BindSource {
            read: from.reader,
            notify_changed: Box::new(move |cmd, value| {
                let target = to.entity;
                let prop = to.writer.clone();
                cmd.add(move |w: &mut World| {
                    w.get_resource_or_insert_with(Events::<ApplyChange<T, V>>::default).send(
                        ApplyChange { writer: prop, target, value }
                    )
                })
            })
        };
        {
            let mut e = self.entity_mut(from.entity);
            if !e.contains::<BindSources<S, V>>() {
                e.insert(BindSources::<S, V>(HashMap::new()));
            }
            e.get_mut::<BindSources<S, V>>()
                .unwrap().0
                .entry(to.entity)
                .or_default()
                .push(bind_source);
        }
        
        
        
        // setup target
        self.entity_mut(to.entity).insert(BindTarget);
        self.resource_mut::<BindTargets>().0
            .entry(to.entity)
            .or_default()
            .insert(from.entity);
    
        let flow = self.resource::<FlowResource>().clone();
        flow.register_component_read_systems::<S, V>();
        flow.register_component_write_systems::<T, V>();    
    }
}

pub trait Bindable: Send + Sync + Clone + PartialEq + 'static { }
impl<T: Send + Sync + Clone + PartialEq + 'static> Bindable for T { }

pub struct BindSource<H: Component, T: Bindable> {
    read: Reader<H, T>,
    notify_changed: Box<dyn Fn(&mut Commands, T) + Send + Sync>
}
#[derive(Component)]
pub struct BindSources<H: Component, T: Bindable>(
    HashMap<Entity, Vec<BindSource<H, T>>>
);


#[derive(Component)]
pub struct BindTarget;

#[derive(Resource)]
pub struct BindTargets(HashMap<Entity, HashSet<Entity>>);
impl BindTargets {
    pub fn new() -> Self {
        BindTargets(HashMap::new())
    }
}

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
        ComponentReader { entity: self.clone(), reader: value.into() }
    }
    fn set(&self, value: impl Into<Writer<H, V>>) -> ComponentWriter<H, V> {
        ComponentWriter { entity: self.clone(), writer: value.into() }
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

pub struct ComponentReader<C: Component, V: Bindable> {
    entity: Entity,
    reader: Reader<C, V>,
}

pub enum Reader<H, V: Bindable> {
    Func(fn(&H) -> Value<V>),
    Closure(Rc<dyn Fn(&H) -> Value<V>>)
}

unsafe impl<H, V: Bindable> Send for Reader<H, V> { }
unsafe impl<H, V: Bindable> Sync for Reader<H, V> { }
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
unsafe impl<H, V: Bindable> Send for Writer<H, V> { }
unsafe impl<H, V: Bindable> Sync for Writer<H, V> { }
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



