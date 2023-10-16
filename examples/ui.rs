use bevy::prelude::*;
use polako::eml::*;
use polako::flow::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Polako UI Sample".into(),
                resolution: (450., 400.).into(),
                position: WindowPosition::At(IVec2::new(300, 300)),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FlowPlugin)
        .add_systems(Startup, hello_world)
        .add_systems(Update, ui_text_system)
        .add_systems(Update, div_system)
        .run();
}


fn test_infer(world: &mut World) {
    let entity = world.spawn_empty().id();
    let system = move |items: Query<&_>| {
        let item = items.get(entity).unwrap();
        let data = prop!(Label.text).get(item).as_ref().clone();
        info!("data: {data}");
    };
    // system.run((), world);

    world.schedule_scope(Update, move |w, s| {
        let mut system = IntoSystem::into_system(system);
        system.initialize(w);
        s.add_systems(system);
    });

}

pub trait GetRefComponent<T: Component> {
    fn get_c(&self, entity: Entity) -> &T;
}


impl<'w, 's, T: Component> GetRefComponent<T> for Query<'w, 's, &T, ()> {
    fn get_c(&self, entity: Entity) -> &T {
        self.get(entity).unwrap()
    }
}

pub trait GetMutComponent<T: Component> {
    fn get_c<'a>(&'a mut self, entity: Entity) -> &'a mut T;
}

impl<'w, 's, T: Component> GetMutComponent<T> for Query<'w, 's, &mut T, ()> {
    fn get_c<'a>(&'a mut self, entity: Entity) -> &'a mut T{
        self.get_mut(entity).unwrap().into_inner()
    }
}


fn takes_mut(q: Query<&mut UiText>) {

}

fn takes_mut_item(item: &mut UiText) {
    
}

// impl div_construct::Props<Get> {
//     pub fn press(&self) { }
// }

fn test_infer_mutability(world: &mut World) {
    let entity = world.spawn_empty().id();
    let system = move |mut items: Query<_>| {
        // let item = items.get(entity).unwrap();
        // let c = items.get_c(entity);
        takes_mut(items)
        // <<Label as Construct>::Props<Lookup> as Singleton>::instance().setters().text(items.get_c(entity), "hello".to_string());
        // items.get_component(entity), |c| {
        //     let 
        //     <<Label as Construct>::Props<Lookup> as Singleton>::instance().setters().text(c, "hello".to_string());
        // })
        // let data = <<Label as Construct>::Props as Singleton>::instance().setters().text(this__, value__)
        //  prop!(Label.text).get(item).as_ref().clone();
        // info!("data: {data}");
    };
    // system.run((), world);

    // world.schedule_scope(Update, move |w, s| {
    //     let mut system = IntoSystem::into_system(system);
    //     system.initialize(w);
    //     s.add_systems(system);
    // });



}
fn hello_world(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.add(
        eml! {
            Body [
                Div {
                    .on.hover: () => {
                        label.text = source.text.fmt("{}, world!")
                    },
                    .bg: #2d2d2d,
                },
                source: Label { .text: "Hello, "},
                label: Label,
                "world",
            ]
            // resource(time, Time);
            // bind(time.elapsed_seconds.fmt("{:0.2}") => elapsed.text);
            // bind(time.elapsed_seconds * 0.5 - 0.5 => content.bg.r);
            // bind(content.bg.hex => color.text);
            // Body + Name { .value: "body" } [
            //     content: Column { .bg: #9d9d9d, .s.padding: [25, 50] }[
            //         Div { .bg: #dedede, .s.padding: 50 } [
            //             "Hello world!"
            //         ],
            //         Row [
            //             "Elapsed: ", elapsed: Label { .text: "0.00" }
            //         ],
            //         Row [
            //             "Color: ", color: Label
            //         ]
            //     ]
            // ]
        }

        // Recursive expansion of eml! macro
// ==================================

// ::polako::eml::Eml:: <Body> ::new(move|world: &mut::bevy::prelude::World,__root__: ::bevy::prelude::Entity|{
//     let __this__ = __root__;
//     let label = world.spawn_empty().id();
//     let label: ::polako::eml::Model<Label>  =  ::polako::eml::Model::new(label);
//     let __root_model__ =  ::polako::eml::Model:: <Body> ::new(__root__);
//     {
//       let __model__ = {
//         world.entity_mut(__root__).insert(::polako::eml::IntoBundle::into_bundle({
//           use::polako::constructivism::traits:: * ;
//           let fields =  <<Body as ::polako::constructivism::Construct> ::Params as ::polako::constructivism::Singleton> ::instance();
//           let params =  <<Body as ::polako::constructivism::Construct> ::ExpandedParams as ::polako::constructivism::Extractable> ::as_params();
//           let defined_params = params.defined();
//           <Body as ::polako::constructivism::Construct> ::construct(defined_params)
//         }));
//         __root_model__
//       };
//       let __content__ = {
//         let mut __content__ =  ::std::vec::Vec:: <_> ::new();
//         __content__.reserve(2usize);
//         let __content_item__ = {
//           {
//             let __model__ = {
//               let __entity__ = world.spawn(::polako::eml::IntoBundle::into_bundle({
//                 use::polako::constructivism::traits:: * ;
//                 let fields =  <<Div as ::polako::constructivism::Construct> ::Params as ::polako::constructivism::Singleton> ::instance();
//                 let params =  <<Div as ::polako::constructivism::Construct> ::ExpandedParams as ::polako::constructivism::Extractable> ::as_params();
//                 let defined_params = params.defined();
//                 <Div as ::polako::constructivism::Construct> ::construct(defined_params)
//               })).id();
//               ::polako::eml::Model:: <Div> ::new(__entity__)
//             };
//             let __content__ = {
//               let mut __content__ =  ::std::vec::Vec:: <_> ::new();
//               __content__.reserve(0usize);
//               __content__
//             };
//             <Div as ::polako::eml::ElementBuilder> ::build_element(__content__).eml().write(world,__model__.entity);
//             {
//               let __entity__ = world.entity_mut(__model__.entity);
//               {
//                 let __ext__ =  <<Div as ::polako::constructivism::Construct> ::Design as ::polako::constructivism::Singleton> ::instance().on().hover();
//                 __ext__.assign(__entity__, ::polako::flow::Hand::new(move|_params: &mut (::polako::bevy::prelude::Query< &mut _, ()> ,)| -> () {
//                   let (_q0,) = _params;
//                   {
//                     let _value = ("hello!").into();
//                     let mut _host = _q0.get_mut(label.entity).unwrap();
//                     if<<Label as ::polako::constructivism::Construct> ::Props< ::polako::constructivism::Lookup>as ::polako::constructivism::Singleton> ::instance().getters().text(&_host).into_value().as_ref()!= &_value {
//                       <<Label as ::polako::constructivism::Construct> ::Props< ::polako::constructivism::Lookup>as ::polako::constructivism::Singleton> ::instance().setters().set_text(_host.as_mut(),_value)
//                     }
//                   }
//                 }));
//               }
//             }__model__
//           }
//         };
//         let _:Implemented =  <<Body as ::polako::constructivism::Construct> ::Design as ::polako::constructivism::Singleton> ::instance().push_content(world, &mut __content__,__content_item__);
//         let __content_item__ = {
//           {
//             let __model__ = {
//               world.entity_mut(label.entity).insert(::polako::eml::IntoBundle::into_bundle({
//                 use::polako::constructivism::traits:: * ;
//                 let fields =  <<Label as ::polako::constructivism::Construct> ::Params as ::polako::constructivism::Singleton> ::instance();
//                 let params =  <<Label as ::polako::constructivism::Construct> ::ExpandedParams as ::polako::constructivism::Extractable> ::as_params();
//                 let defined_params = params.defined();
//                 <Label as ::polako::constructivism::Construct> ::construct(defined_params)
//               }));
//               label
//             };
//             let __content__ = {
//               let mut __content__ =  ::std::vec::Vec:: <_> ::new();
//               __content__.reserve(0usize);
//               __content__
//             };
//             <Label as ::polako::eml::ElementBuilder> ::build_element(__content__).eml().write(world,__model__.entity);
//             {
//               let __entity__ = world.entity_mut(__model__.entity);
//             }__model__
//           }
//         };
//         let _:Implemented =  <<Body as ::polako::constructivism::Construct> ::Design as ::polako::constructivism::Singleton> ::instance().push_content(world, &mut __content__,__content_item__);
//         __content__
//       };
//       <Body as ::polako::eml::ElementBuilder> ::build_element(__content__).eml().write(world,__model__.entity);
//       {
//         let __entity__ = world.entity_mut(__model__.entity);
//       }__model__
//     };
//   })
    )
}

#[derive(Component, Construct)]
#[construct(Div -> Empty)]
pub struct Div {
    #[prop(construct)]
    /// The background color of element
    bg: Color,
}
impl ElementBuilder for Div {
    fn build_element(content: Vec<Entity>) -> Blueprint<Self> {
        blueprint! {
            Div::Base
            + NodeBundle
            [[ content ]]
        }
    }
}

#[derive(Component, Behaviour)]
pub struct UiText {
    /// The text value of UiText element.
    pub text: String,
    #[param(default = Color::hex("2f2f2f").unwrap())]
    pub text_color: Color,
}

#[derive(Component, Construct)]
#[construct(Label -> UiText -> Div)]
pub struct Label;
impl ElementBuilder for Label {
    fn build_element(_: Vec<Entity>) -> Blueprint<Self> {
        blueprint! {
            Label::Base + TextBundle
        }
    }
}

#[derive(Component, Construct)]
#[construct(Body -> Div)]
pub struct Body;
impl ElementBuilder for Body {
    fn build_element(content: Vec<Entity>) -> Blueprint<Self> {
        blueprint! {
            Body::Base
            + Style(
                .width: Val::Percent(100.),
                .height: Val::Percent(100.),
                .justify_content: JustifyContent::Center,
                .align_content: AlignContent::Center,
                .align_items: AlignItems::Center,
            )
            [[ content ]]
        }
    }
}

#[derive(Component, Construct)]
#[construct(Column -> Div)]
pub struct Column;
impl ElementBuilder for Column {
    fn build_element(content: Vec<Entity>) -> Blueprint<Self> {
        blueprint! {
            Column::Base
            + Style (
                .display: Display::Flex,
                .flex_direction: FlexDirection::Column,
                .align_items: AlignItems::Center,
                .align_content: AlignContent::Center,
                .row_gap: Val::Px(3.),
            )
            [[ content ]]
        }
    }
}

#[derive(Component, Construct)]
#[construct(Row -> Div)]
pub struct Row;
impl ElementBuilder for Row {
    fn build_element(content: Vec<Entity>) -> Blueprint<Self> {
        blueprint! {
            Row::Base
            + Style (
                .display: Display::Flex,
                .flex_direction: FlexDirection::Row,
                .align_items: AlignItems::Center,
                .align_content: AlignContent::Center,
                .column_gap: Val::Px(3.),
            )
            [[ content ]]
        }
    }
}

use bevy::ecs::world::EntityMut;
use polako_constructivism::Construct;
use polako_constructivism::Get;
use polako_constructivism::Is;
use polako_constructivism::Lookup;
use polako_constructivism::Singleton;
impl DivDesign {
    // Div can accept string literals as content
    pub fn push_text<'c, S: AsRef<str>>(
        &self,
        world: &mut World,
        content: &'c mut Vec<Entity>,
        text: S,
    ) -> Implemented {
        let entity = world.spawn(TextBundle::with_text(text)).id();
        content.push(entity);
        Implemented
    }
    // Only Div and elements based on Div can be content of the Div
    pub fn push_content<E: ElementBuilder + Is<Div>>(
        &self,
        _: &mut World,
        content: &mut Vec<Entity>,
        model: Model<E>,
    ) -> Implemented {
        content.push(model.entity);
        Implemented
    }
    /// Everything based on Div can access the styles using param extensions: `Row { .s.padding: 25 }`
    pub fn s(&self) -> &'static Styles {
        &Styles
    }
    pub fn on(&self) -> &'static Signals {
        &Signals
    }
}

use polako_flow::input::HoverSignal;
use polako_flow::Signal;
pub struct Signals;
impl Signals {
    pub fn hover(&self) -> &'static <HoverSignal as Signal>::Descriptor {
        <<HoverSignal as Signal>::Descriptor as Singleton>::instance()
    }
}
pub struct Styles;
impl Styles {
    /// The amount of space between the edges of a node and its contents in pixels.
    pub fn padding<T: IntoRect>(&self) -> StyleProperty<T> {
        StyleProperty(|mut entity, padding| {
            let rect = padding.into_rect();
            if !entity.contains::<Style>() {
                entity.insert(Style::default());
            }
            entity.get_mut::<Style>().unwrap().padding = rect;
        })
    }
}
pub struct StyleProperty<T>(fn(EntityMut, T));
impl<T> StyleProperty<T> {
    pub fn assign<'w>(&self, entity: EntityMut<'w>, value: T) {
        (self.0)(entity, value)
    }
}

// helpers for spawning text bundle
impl Default for UiText {
    fn default() -> Self {
        UiText {
            text: "".into(),
            text_color: Color::hex("2f2f2f").unwrap(),
        }
    }
}
pub trait WithText {
    fn with_text<T: AsRef<str>>(text: T) -> Self;
}
impl WithText for TextBundle {
    fn with_text<T: AsRef<str>>(text: T) -> TextBundle {
        let mut text = TextBundle::from_section(text.as_ref(), Default::default());
        text.text.sections[0].style.font_size = 24.;
        text.text.sections[0].style.color = Color::hex("2f2f2f").unwrap();
        text
    }
}
impl WithText for Text {
    fn with_text<T: AsRef<str>>(text: T) -> Self {
        let mut text = Text::from_section(text.as_ref().to_string(), Default::default());
        text.sections[0].style.font_size = 24.;
        text.sections[0].style.color = Color::hex("2f2f2f").unwrap();
        text
    }
}

/// bypass Div.background to BackgroundColor.0 when changed
/// and Div.padding to Style.padding
fn div_system(mut colors: Query<(&Div, &mut BackgroundColor), Changed<Div>>) {
    colors.for_each_mut(|(div, mut bg)| {
        bg.0 = div.bg.into();
    });
}

/// bypass UiText text value & color to Text.sections[0] when changed
fn ui_text_system(mut texts: Query<(&UiText, &mut Text), Changed<UiText>>) {
    for (ui_text, mut text) in texts.iter_mut() {
        if text.sections.is_empty() {
            *text = Text::with_text("");
        }
        text.sections[0].value = ui_text.text.clone();
        text.sections[0].style.color = ui_text.text_color.into();
    }
}

pub trait IntoRect {
    fn into_rect(self) -> UiRect;
}

impl IntoRect for i32 {
    fn into_rect(self) -> UiRect {
        UiRect::all(Val::Px(self as f32))
    }
}
impl IntoRect for f32 {
    fn into_rect(self) -> UiRect {
        UiRect::all(Val::Px(self))
    }
}

impl IntoRect for [i32; 2] {
    fn into_rect(self) -> UiRect {
        UiRect {
            left: Val::Px(self[0] as f32),
            right: Val::Px(self[0] as f32),
            top: Val::Px(self[1] as f32),
            bottom: Val::Px(self[1] as f32),
        }
    }
}
