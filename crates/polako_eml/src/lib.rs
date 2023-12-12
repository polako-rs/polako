use std::{cell::RefCell, marker::PhantomData, rc::Rc};

use bevy::{
    ecs::{
        system::{Command, CommandQueue},
        world::EntityWorldMut,
    },
    prelude::{Resource, *},
};
use polako_constructivism::{traits::Construct, *};
use polako_flow::{EnterSignal, NotifyChange, OnDemandSignal, UpdateSignal};

#[cfg(test)]
mod tests;

pub mod msg {
    pub struct TextAsContent;
    pub struct ElementAsContent;
}

pub trait Element: ElementBuilder {
    type Signals: Singleton + 'static;
}

pub trait ElementBuilder: Component + Construct + Sized {
    fn build_element(content: Vec<Entity>) -> Blueprint<Self>;
}

/// Transforms (A, (B, (C, (D, ())))) into (A, ((), (C, ((), ())))
/// where only A & C impl Bundle (and Component implicitly)
pub trait IntoBundle {
    type Output: Bundle;
    fn into_bundle(self) -> Self::Output;
}

impl IntoBundle for () {
    type Output = ();
    fn into_bundle(self) -> Self::Output {
        ()
    }
}

impl<T: ElementBuilder> IntoBundle for T {
    type Output = Self;
    fn into_bundle(self) -> Self::Output {
        self
    }
}

impl<A, AOut, B, BOut> IntoBundle for (A, B)
where
    A: IntoBundle<Output = AOut>,
    AOut: Bundle,
    B: IntoBundle<Output = BOut>,
    BOut: Bundle,
{
    type Output = (A::Output, B::Output);
    fn into_bundle(self) -> Self::Output {
        (self.0.into_bundle(), self.1.into_bundle())
    }
}

/// Behavior is about adding shared functionality
/// to elements. Like `Pressable` in `#[construct(Button -> Pressable -> Div)]
pub trait Behavior: Segment + Component {
    type Signals<T: Singleton + 'static>: Singleton + 'static;
}

/// Constraint is about to define the rules the eml
/// tree is built. Like `AcceptOnly<T>` in `#[construct(TabView -> AcceptOnly<Tab> -> Div)]
pub trait Constraint: Segment + IntoBundle {}

pub trait PolakoType: TypeReference
where
    Self::Type: Component,
{
    fn notify_changed(&self, commands: &mut Commands, entity: Entity);
}

impl<T: TypeReference> PolakoType for T
where
    Self::Type: Component,
{
    fn notify_changed(&self, commands: &mut Commands, entity: Entity) {
        commands.add(NotifyChange::<Self::Type>::new(entity))
    }
}

pub struct EntityMark<E: Element> {
    pub entity: Entity,
    marker: PhantomData<E>,
}

impl<E: Element> EntityMark<E> {
    pub fn new(entity: Entity) -> Self {
        EntityMark {
            entity,
            marker: PhantomData,
        }
    }
    pub fn getters(&self) -> &'static E::Props<Get> {
        <<E as Construct>::Props<Get> as Singleton>::instance()
    }
    pub fn setters(&self) -> &'static E::Props<Set> {
        <<E as Construct>::Props<Set> as Singleton>::instance()
    }
    pub fn descriptor(&self) -> &'static E::Props<Describe> {
        <<E as Construct>::Props<Describe> as Singleton>::instance()
    }
}

impl<E: Element> Copy for EntityMark<E> {}
impl<E: Element> std::fmt::Debug for EntityMark<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EntityMark")
            .field("entity", &self.entity)
            .finish()
    }
}
impl<E: Element> Clone for EntityMark<E> {
    fn clone(&self) -> Self {
        Self {
            entity: self.entity,
            marker: PhantomData,
        }
    }
}

pub struct ResourceMark<R: Resource + Construct>(PhantomData<R>);

impl<R: Resource + Construct> ResourceMark<R> {
    pub fn new() -> Self {
        ResourceMark(PhantomData)
    }
    pub fn getters(&self) -> &'static R::Props<Get> {
        <<R as Construct>::Props<Get> as Singleton>::instance()
    }
    pub fn setters(&self) -> &'static R::Props<Set> {
        <<R as Construct>::Props<Set> as Singleton>::instance()
    }
    pub fn descriptor(&self) -> &'static R::Props<Describe> {
        <<R as Construct>::Props<Describe> as Singleton>::instance()
    }
}

impl<R: Resource + Construct> Copy for ResourceMark<R> {}
impl<R: Resource + Construct> std::fmt::Debug for ResourceMark<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EntityMark").finish()
    }
}
impl<R: Resource + Construct> Clone for ResourceMark<R> {
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}

pub struct Eml<Root: ElementBuilder>(Box<dyn FnOnce(&mut World, Entity)>, PhantomData<Root>);

unsafe impl<Root: ElementBuilder> Send for Eml<Root> {}
unsafe impl<Root: ElementBuilder> Sync for Eml<Root> {}

impl<Root: ElementBuilder> Eml<Root> {
    pub fn new<F: 'static + FnOnce(&mut World, Entity)>(body: F) -> Self {
        Eml(Box::new(body), PhantomData)
    }
    pub fn write(self, world: &mut World, entity: Entity) {
        (self.0)(world, entity);
    }
}

impl<Root: ElementBuilder> Command for Eml<Root> {
    fn apply(self, world: &mut World) {
        let entity = world.spawn_empty().id();
        self.write(world, entity)
    }
}

pub struct Implemented;
pub struct NotImplemented<T>(PhantomData<T>);
impl<T> NotImplemented<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

#[derive(Component, Construct)]
#[construct(Empty -> Nothing)]
pub struct Empty {}
impl Element for Empty {
    type Signals = EmptySignals;
}

impl ElementBuilder for Empty {
    fn build_element(content: Vec<Entity>) -> Blueprint<Self> {
        Blueprint::new(Eml::new(move |world, entity| {
            world.entity_mut(entity).push_children(&content);
        }))
    }
}

pub struct EmptySignals;
impl Singleton for EmptySignals {
    fn instance() -> &'static Self {
        &EmptySignals
    }
}
impl EmptySignals {
    pub fn enter(&self) -> &'static OnDemandSignal<EnterSignal> {
        OnDemandSignal::instance()
    }

    pub fn update(&self) -> &'static OnDemandSignal<UpdateSignal> {
        OnDemandSignal::instance()
    }
}

impl EmptyDesign {
    #[allow(unused_variables)]
    pub fn push_text<'c, S: AsRef<str>>(
        &self,
        world: &mut World,
        content: &'c mut Vec<Entity>,
        text: S,
    ) -> NotImplemented<msg::TextAsContent> {
        NotImplemented::new()
    }

    #[allow(unused_variables)]
    pub fn push_content<E: Element>(
        &self,
        world: &mut World,
        content: &mut Vec<Entity>,
        model: EntityMark<E>,
    ) -> Implemented {
        content.push(model.entity);
        Implemented
    }
}

pub struct Blueprint<T: ElementBuilder>(Eml<T>);
impl<T: ElementBuilder> Blueprint<T> {
    pub fn new(eml: Eml<T>) -> Self {
        Self(eml)
    }
    pub fn eml(self) -> Eml<T> {
        self.0
    }
}

#[derive(Clone)]
pub struct CommandStackItem {
    stack: CommandStack,
    idx: usize,
}

impl CommandStackItem {
    pub fn update<C: Component + Default, F: FnOnce(&mut C) + Send + Sync + 'static>(
        &mut self,
        entity: Entity,
        func: F,
    ) {
        self.entity(entity, move |e| {
            if e.contains::<C>() {
                func(e.get_mut().as_mut().unwrap());
            } else {
                let mut component = C::default();
                func(&mut component);
                e.insert(component);
            }
        })
    }
    pub fn insert<B: Bundle>(&mut self, entity: Entity, bundle: B) {
        self.entity(entity, move |e| {
            e.insert(bundle);
        });
    }
    pub fn entity<F: FnOnce(&mut EntityWorldMut) + Send + Sync + 'static>(
        &mut self,
        entity: Entity,
        func: F,
    ) {
        let mut borrow = self.stack.0.borrow_mut();
        let queue = borrow.get_mut(self.idx).unwrap();
        queue.push(move |world: &mut World| func(&mut world.entity_mut(entity)))
    }
}

#[derive(Clone, Resource, Default)]
pub struct CommandStack(Rc<RefCell<Vec<CommandQueue>>>);
unsafe impl Send for CommandStack {}
unsafe impl Sync for CommandStack {}

impl CommandStack {
    pub fn push(&mut self) -> CommandStackItem {
        let idx = self.0.borrow().len();
        self.0.borrow_mut().push(CommandQueue::default());
        CommandStackItem {
            idx,
            stack: self.clone(),
        }
    }
}

#[derive(Component, Constraint)]
pub struct AcceptNoContent;

impl<T> AcceptNoContentDesign<T> {
    #[allow(unused_variables)]
    pub fn push_content<E: Element>(
        &self,
        world: &mut World,
        content: &mut Vec<Entity>,
        model: EntityMark<E>,
    ) -> NotImplemented<msg::ElementAsContent> {
        NotImplemented::new()
    }
    #[allow(unused_variables)]
    pub fn push_text<'c, S: AsRef<str>>(
        &self,
        world: &mut World,
        content: &'c mut Vec<Entity>,
        text: S,
    ) -> NotImplemented<msg::TextAsContent> {
        NotImplemented::new()
    }
}
