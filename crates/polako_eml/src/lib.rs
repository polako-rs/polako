use std::marker::PhantomData;

use bevy::prelude::*;
use polako_constructivism::{*, traits::Construct};

pub mod msg {
    pub struct TextAsChild;
}


pub trait Element: Component + Construct + {
    type Install: InstallElement;
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

impl<C: Element> Clone for Model<C> {
    fn clone(&self) -> Self {
        Self {
            entity: self.entity,
            marker: PhantomData
        }
    }
}

pub struct Eml(Box<dyn Fn(&mut World, Entity)>);

impl Eml {
    pub fn new<F: 'static + Fn(&mut World, Entity)>(body: F) -> Self {
        Eml(Box::new(body))
    }
    pub fn apply(self, world: &mut World, entity: Entity) {

    }
}

pub trait InstallElement {
    type Element: Element;
    fn install(world: &mut World, this: Model<Self::Element>, content: Vec<Entity>);
}


pub struct Valid<T>(pub T);
pub struct NotSupported<T>(pub T);

#[derive(Component, Construct)]
pub struct Elem {

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