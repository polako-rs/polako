use std::{marker::PhantomData, rc::Rc, cell::RefCell};

use bevy::{prelude::*, ecs::{system::{Command, CommandQueue}, world::EntityMut}};
use polako_constructivism::{*, traits::Construct};


#[cfg(test)]
mod tests;

pub mod msg {
    pub struct TextAsContent;
    pub struct ElementAsContent;
}


pub trait Element: Component + Construct + Sized {
    // type Builder: ElementBuilder<Self> + Singleton;

    fn build_element(this: Model<Self>, content: Vec<Entity>) -> Blueprint<Self>;
}


pub struct Model<C: Element> {
    pub entity: Entity,
    marker: PhantomData<C>
}

impl<C: Element> Model<C> {
    pub fn new(entity: Entity) -> Self {
        Model { entity, marker: PhantomData }
    }
}

impl<C: Element> Copy for Model<C> {
    
}

impl<C: Element> std::fmt::Debug for Model<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f
            .debug_struct("Model")
            .field("entity", &self.entity)
            .finish()
    }
}

impl<C: Element> Clone for Model<C> {
    fn clone(&self) -> Self {
        Self {
            entity: self.entity,
            marker: PhantomData
        }
    }
}

pub struct Eml<Root: Element>(
    Box<dyn FnOnce(&mut World, Entity)>,
    PhantomData<Root>
);

unsafe impl<Root: Element> Send for Eml<Root> { }
unsafe impl<Root: Element> Sync for Eml<Root> { }

impl<Root: Element> Eml<Root> {
    pub fn new<F: 'static + FnOnce(&mut World, Entity)>(body: F) -> Self {
        Eml(Box::new(body), PhantomData)
    }
    pub fn write(self, world: &mut World, entity: Entity) {
        (self.0)(world, entity);
    }
}

impl<Root: Element> Command for Eml<Root> {
    fn apply(self, world: &mut World) {
        let entity = world.spawn_empty().id();
        self.write(world, entity)
    }
}

pub struct Implemented;
pub struct NotImplemented<T>(PhantomData<T>);
impl<T> NotImplemented<T> {
    pub fn new() -> Self {
        Self (PhantomData)
    }
}

#[derive(Component, Construct)]
pub struct Elem {

}

impl Element for Elem {
    fn build_element(_: Model<Self>, content: Vec<Entity>) -> Blueprint<Self> {
        Blueprint::new(Eml::new(move |world, entity| {
            world.entity_mut(entity).push_children(&content);
        }))
        
    }
}

impl elem_construct::Protocols {
    #[allow(unused_variables)]
    pub fn push_text<'c, S: AsRef<str>>(&self, world: &mut World, content: &'c mut Vec<Entity>, text: S) -> NotImplemented<msg::TextAsContent> {
        NotImplemented::new()
    }

    #[allow(unused_variables)]
    pub fn push_content<E: Element>(&self, world: &mut World, content: &mut Vec<Entity>, model: Model<E>) -> Implemented {
        content.push(model.entity);
        Implemented
    }
}

pub struct Blueprint<T: Element>(Eml<T>);
impl<T: Element> Blueprint<T> {
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
    pub fn update<C: Component + Default, F: FnOnce(&mut C) + Send + Sync + 'static>(&mut self, entity: Entity, func: F) {
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
        self.entity(entity, move |e| { e.insert(bundle); });
    }
    pub fn entity<F: FnOnce(&mut EntityMut) + Send + Sync + 'static>(&mut self, entity: Entity, func: F) {
        let mut borrow = self.stack.0.borrow_mut();
        let queue = borrow.get_mut(self.idx).unwrap();
        queue.push(move |world: &mut World| func(&mut world.entity_mut(entity)))
    }
}

#[derive(Clone, Resource, Default)]
pub struct CommandStack(Rc<RefCell<Vec<CommandQueue>>>);
unsafe impl Send for CommandStack { }
unsafe impl Sync for CommandStack { }

impl CommandStack {
    pub fn push(&mut self) -> CommandStackItem {
        let idx = self.0.borrow().len();
        self.0.borrow_mut().push(CommandQueue::default());
        CommandStackItem { idx, stack: self.clone() }
    }
}


#[derive(Component, Mixin)]
pub struct AcceptNoContent;

impl<T: Singleton> acceptnocontent_construct::Protocols<T> {
    #[allow(unused_variables)]
    pub fn push_content<E: Element>(&self, world: &mut World, content: &mut Vec<Entity>, model: Model<E>) -> NotImplemented<msg::ElementAsContent> {
        NotImplemented::new()
    }
    #[allow(unused_variables)]
    pub fn push_text<'c, S: AsRef<str>>(&self, world: &mut World, content: &'c mut Vec<Entity>, text: S) -> NotImplemented<msg::TextAsContent> {
        NotImplemented::new()
    }
}
