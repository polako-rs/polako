use polako_constructivism::Singleton;
use polako_input::{PointerInput, PointerInputData, PointerInputPosition};
use bevy::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::ecs::world::EntityMut;
use super::{Signal, Hand, Handler};
macro_rules! impl_signal {
    ($variant:ident, $name:ident, $marker:ident) => {
        pub struct $name;
        impl Signal for $name {
            type Event = PointerInput;
            type Args = PointerInputPosition;
            type Descriptor = $marker;
            fn filter(event: &Self::Event) -> Option<Entity> {
                matches!(event.data, PointerInputData::$variant).then_some(event.entity)
            }
        }
        pub struct $marker;
        impl Singleton for $marker {
            fn instance() -> &'static $marker {
                &$marker
            }
        }
        impl $marker {
            pub fn emit(&self, world: &mut World, entity: Entity, position: <$name as Signal>::Args) {
                world
                    .get_resource_or_insert_with(Events::<PointerInput>::default)
                    .send(PointerInput {
                        entity,
                        position,
                        data: PointerInputData::$variant,
                    })
            }

            pub fn assign<'w, S: SystemParam>(
                &self,
                entity: EntityMut<'w>,
                value: Hand<$name, S>,
            ) {
                
            }
        }
    };
}

impl_signal!(Up, UpSignal, UpSignalMarker);
impl_signal!(Down, DownSignal, DownSignalMarker);
impl_signal!(Motion, MotionSignal, MotionSignalMarker);
impl_signal!(DragStart, DragStSignal, DragStartSignalMarker);
impl_signal!(Drag, DragSignal, DragSignalMarker);
impl_signal!(DragStop, DragStopSignal, DragStopSignalMarker);
impl_signal!(Hover, HoverSignal, HoverSignalMarker);
impl_signal!(Focus, FocusSignal, FocusSignalMarker);