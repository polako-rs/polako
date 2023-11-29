use bevy::prelude::{Entity, Event};
use polako_constructivism::{Construct, Singleton};

pub trait Signal {
    type Event: Event;
    type Descriptor: Singleton;
    type Args: Construct;
    fn filter(event: &Self::Event) -> Option<Entity>;
    // fn emit(world: &mut World, entity: Entity, args: Self::Args);
    // fn props(&self) -> &'static <Self::Args as Construct>::Props<Lookup> {
    //     <<Self::Args as Construct>::Props<Lookup> as Singleton>::instance()
    // }
    // fn params(&self) -> &'static <Self::Args as Construct>::Params {
    //     <<Self::Args as Construct>::Params as Singleton>::instance()
    // }
}

pub trait Sig {
    type Marker;
}

pub struct Pressed {}

pub struct PressedSignalMarker;

impl Sig for Pressed {
    type Marker = PressedSignalMarker;
}
