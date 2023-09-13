use std::marker::PhantomData;

use bevy::{prelude::*, ecs::system::Command};
use polako_constructivism::{*, traits::Construct};


#[cfg(test)]
mod tests;

pub mod msg {
    pub struct TextAsChild;
}


pub trait Element: Component + Construct + {
    type Build: Build;
}

pub struct Model<C: Element> {
    pub entity: Entity,
    marker: PhantomData<C>
}

impl<C: Element> Model<C> {
    pub fn new(entity: Entity) -> Self {
        Model { entity, marker: PhantomData }
    }

    pub fn view(&self) -> View<C> {
        View { entity: self.entity, marker: PhantomData }
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

#[derive(Component)]
pub struct View<C: Element> {
    pub entity: Entity,
    marker: PhantomData<C>
}

pub type BuildArgs<T> = In<(Model<T>, Vec<Entity>)>;

pub struct Eml<Root: Element>(Box<dyn FnOnce(&mut World, Entity)>, PhantomData<Root>);

unsafe impl<Root: Element> Send for Eml<Root> { }
unsafe impl<Root: Element> Sync for Eml<Root> { }

impl<Root: Element> Eml<Root> {
    pub fn new<F: 'static + FnOnce(&mut World, Entity)>(body: F) -> Self {
        Eml(Box::new(body), PhantomData)
    }
    pub fn write(self, world: &mut World, entity: Entity) {
        (self.0)(world, entity)
    }
}

impl<Root: Element> Command for Eml<Root> {
    fn apply(self, world: &mut World) {
        let entity = world.spawn_empty().id();
        (self.0)(world, entity)
    }
}


pub trait Build {
    type Element: Element;
    fn build(world: &mut World, this: Model<Self::Element>, content: Vec<Entity>);
}


pub struct Valid<T>(pub T);
pub struct NotSupported<T>(pub T);

#[derive(Component, Construct)]
pub struct Elem {

}

pub struct BuildElem;
impl Build for BuildElem {
    type Element = Elem;
    fn build(world: &mut World, this: Model<Self::Element>, content: Vec<Entity>) {
        world.entity_mut(this.entity).push_children(&content);
    }
}

impl Element for Elem {
    type Build = BuildElem;
}


impl elem_construct::Methods {
    #[allow(unused_variables)]
    pub fn push_text<'c, S: AsRef<str>>(&self, world: &mut World, content: &'c mut Vec<Entity>, text: S) -> NotSupported<msg::TextAsChild> {
        NotSupported(msg::TextAsChild)
    }

    #[allow(unused_variables)]
    pub fn push_model<E: Element>(&self, world: &mut World, content: &mut Vec<Entity>, model: Model<E>) -> Valid<()> {
        content.push(model.entity);
        Valid(())
    }

}

pub fn validate_builder<E: Element + Extends<R>, R: Element>(In(eml): In<Eml<R>>) -> Eml<R> {
    eml
}