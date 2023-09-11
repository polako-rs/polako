use std::marker::PhantomData;

use bevy::prelude::*;
use polako_constructivism::{*, traits::Construct};


pub trait Element: Component + Construct + {
    type ContentType: NewContent;
    type Install: InstallElement<Self::ContentType>;
    type PushText: PushText<Self::ContentType>;
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

pub trait InstallElement<Content> {
    type Element: Element<ContentType = Content>;
    fn install(world: &mut World, this: Model<Self::Element>, content: Vec<Content>);
}

pub trait PushText<Content> {
    fn push_text<'c, S: AsRef<str>>(world: &mut World, content: &'c mut Vec<Content>, text: S) -> &'c Content;
}

pub trait IntoContent<Content> {
    fn into_content(world: &mut World, this: Self) -> Content;
}
impl<T: Bundle> IntoContent<Entity> for T {
    fn into_content(world: &mut World, this: Self) -> Entity {
        world.spawn(this).id()
    }
}

pub struct Valid<T>(T);

pub trait NewContent {
    type Output;
    fn new_content(world: &mut World) -> Self::Output;
}

impl NewContent for Entity {
    type Output = Valid<Entity>;
    fn new_content(world: &mut World) -> Self::Output {
        Valid(world.spawn_empty().id())
    }
}

pub trait AssignContent<Content> {
    fn assign_content(world: &mut World, content: Content, value: Self);
}

impl<T: Bundle> AssignContent<Entity> for T {
    fn assign_content(world: &mut World, content: Entity, value: Self) {
        world.entity_mut(content).insert(value);
    }
}