use polako_constructivism::Singleton;
use polako_input::{PointerInput, PointerInputData, PointerInputPosition};
use bevy::prelude::*;
use super::Signal;

macro_rules! impl_signal {
    ($variant:ident, $name:ident, $marker:ident) => {
        // #[derive(Event)]
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

            pub fn assign<'w, S: ::bevy::ecs::system::SystemParam + 'static, F: Fn(&<$name as $crate::Signal>::Event, &mut ::bevy::ecs::system::StaticSystemParam<S>) + 'static>(
                &self,
                entity: &mut ::bevy::ecs::world::EntityMut<'w>,
                func: F
            ) {
                let hand = $crate::Hand::new(func);
                if !entity.contains::<$crate::Hands<<$name as $crate::Signal>::Event , S>>() {
                    entity.insert((
                        $crate::Hands(vec![hand]),
                        $crate::FlowItem
                    ));
                } else {
                    entity.get_mut::<$crate::Hands<<$name as $crate::Signal>::Event, S>>().unwrap().push(hand);
                }
                entity.insert(::polako_input::PointerFilter::Pass);
                entity.world_scope(|world| {
                    world.resource::<$crate::FlowResource>().register_handle_signals_systems::<$name, S>();
                });
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