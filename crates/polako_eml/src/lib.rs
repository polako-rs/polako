use std::marker::PhantomData;

use bevy::{prelude::*, ecs::{system::Command, world::EntityMut}};
use polako_constructivism::{*, traits::Construct};


#[cfg(test)]
mod tests;

pub mod msg {
    pub struct TextAsContent;
    pub struct ElementAsContent;
}


pub trait Element: Component + Construct + {
    type Build: Build;
}


pub struct EntityComponent<C: Element> {
    pub entity: Entity,
    marker: PhantomData<C>
}

impl<C: Element> EntityComponent<C> {
    pub fn new(entity: Entity) -> Self {
        EntityComponent { entity, marker: PhantomData }
    }

    pub fn as_view(&self) -> View<C> {
        View { for_model: self.entity, marker: PhantomData }
    }
    pub fn as_model(&self) -> Model<C> {
        Model { for_view: self.entity, marker: PhantomData }
    }
}

impl<C: Element> Copy for EntityComponent<C> {
    
}

impl<C: Element> std::fmt::Debug for EntityComponent<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f
            .debug_struct("EntityComponent")
            .field("entity", &self.entity)
            .finish()
    }
}

impl<C: Element> Clone for EntityComponent<C> {
    fn clone(&self) -> Self {
        Self {
            entity: self.entity,
            marker: PhantomData
        }
    }
}

pub trait IntoBase<T> {
    fn into_base(self) -> T;
}
impl<B: Element, T: Element + Extends<B>> IntoBase<EntityComponent<B>> for EntityComponent<T> {
    fn into_base(self) -> EntityComponent<B> {
        EntityComponent::new(self.entity)
    }
}

#[derive(Component)]
pub struct Model<C: Element> {
    pub for_view: Entity,
    marker: PhantomData<C>
}

impl<C: Element> Model<C> {
    pub fn new(for_view: Entity) -> Self {
        Self {
            for_view,
            marker: PhantomData
        }
    }
}

impl<C: Element> std::fmt::Debug for Model<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f
            .debug_struct("EntityComponent")
            .field("for_view", &self.for_view)
            .finish()
    }
}

impl<C: Element> Clone for Model<C> {
    fn clone(&self) -> Self {
        Self {
            for_view: self.for_view,
            marker: PhantomData
        }
    }
}


#[derive(Component)]
pub struct View<C: Element> {
    pub for_model: Entity,
    marker: PhantomData<C>
}
impl<C: Element> View<C> {
    pub fn new(entity: Entity) -> Self {
        Self { for_model: entity, marker: PhantomData }
    }
}

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
    fn build(world: &mut World, this: Entity, model: EntityComponent<Self::Element>, content: Vec<Entity>);
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

pub struct BuildElem;
impl Build for BuildElem {
    type Element = Elem;
    fn build(world: &mut World, _this: Entity, model: EntityComponent<Self::Element>, content: Vec<Entity>) {
        world.entity_mut(model.entity).push_children(&content);
    }
}

impl Element for Elem {
    type Build = BuildElem;
}


impl elem_construct::Protocols {
    #[allow(unused_variables)]
    pub fn push_text<'c, S: AsRef<str>>(&self, world: &mut World, content: &'c mut Vec<Entity>, text: S) -> NotImplemented<msg::TextAsContent> {
        NotImplemented::new()
    }

    #[allow(unused_variables)]
    pub fn push_content<E: Element>(&self, world: &mut World, content: &mut Vec<Entity>, model: EntityComponent<E>) -> Implemented {
        content.push(model.entity);
        Implemented
    }
}

pub struct Builder<T: Element>(Eml<T>);
impl<T: Element> Builder<T> {
    pub fn new(eml: Eml<T>) -> Self {
        Self(eml)
    }
    pub fn eml(self) -> Eml<T> {
        self.0
    }
}

pub fn validate_builder<E: Element + Extends<R>, R: Element>(In(builder): In<Builder<R>>) -> Eml<R> {
    builder.eml()
}


// pub fn assign_views(
//     views: Query<&View
// )

pub struct Context<'w, E: Element> {
    pub world: &'w mut World,
    pub this: Entity,
    marker: PhantomData<E>,
}

impl<'w, E: Element> Context<'w, E> {
    pub fn insert<C: Bundle>(&mut self, bundle: C) {
        let mut entity = self.world.entity_mut(self.this);
        entity.insert(bundle);
    }
    pub fn component<'a: 'w, C: Component>(&'a mut self) -> EntityComponentMut<'w, 'a, C> {
        let entity = self.world.entity_mut(self.this);
        EntityComponentMut {
            entity, marker: PhantomData
        }
    }

}

pub struct EntityComponentMut<'w, 'a, C: Component> {
    entity: EntityMut<'w>,
    marker: PhantomData<(&'a (), C)>
}

impl<'w, 'a, C: Component> std::ops::Deref for EntityComponentMut<'w, 'a, C> {
    type Target = C;
    fn deref(&self) -> &Self::Target {
        self.entity.get().as_ref().unwrap()
    }
}

pub struct BuildArgs<'w, E: Element> {
    pub this: Entity,
    pub model: EntityComponent<E>,
    pub content: Vec<Entity>,
    pub ctx: Context<'w, E>,

}

pub trait ElementBuilder<E: Element> {
    fn build_element(&self, world: &mut World, this: Entity, model: EntityComponent<E>, content: Vec<Entity>);
}


impl<F, E: Element, R> ElementBuilder<E> for F
where
    R: Element,
    E: Element + Extends<R>,
    F: Fn(BuildArgs<E>) -> Builder<R>,
{
    fn build_element(&self, world: &mut World, this: Entity, model: EntityComponent<E>, content: Vec<Entity>) {
            let args = BuildArgs {
                this, model, content, ctx: Context { world, this, marker: PhantomData }
            };
            let eml = self(args).eml();
            eml.write(world, this);
    }
}



#[derive(Component, Mixin)]
pub struct AcceptNoContent;

impl<T: Singleton> acceptnocontent_construct::Protocols<T> {
    #[allow(unused_variables)]
    pub fn push_content<E: Element>(&self, world: &mut World, content: &mut Vec<Entity>, model: EntityComponent<E>) -> NotImplemented<msg::ElementAsContent> {
        NotImplemented::new()
    }
    #[allow(unused_variables)]
    pub fn push_text<'c, S: AsRef<str>>(&self, world: &mut World, content: &'c mut Vec<Entity>, text: S) -> NotImplemented<msg::TextAsContent> {
        NotImplemented::new()
    }
}
