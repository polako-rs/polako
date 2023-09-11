use std::marker::PhantomData;

use bevy::prelude::*;
use polako_constructivism::{*, traits::Construct};


pub trait Element: Component + Construct + {
    type Install: InstallElement;
    type PushText: PushText;
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

pub trait PushText {
    fn push_text<'c, S: AsRef<str>>(world: &mut World, content: &'c mut Vec<Entity>, text: S) -> &'c Entity;
}

pub trait IntoContent {
    fn into_content(world: &mut World, this: Self) -> Entity;
}
impl<T: Bundle> IntoContent for T {
    fn into_content(world: &mut World, this: Self) -> Entity {
        world.spawn(this).id()
    }
}
