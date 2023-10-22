use std::ops::DerefMut;

use bevy::ecs::system::StaticSystemParam;
use bevy::prelude::*;
use polako::eml::*;
use polako::flow::*;

#[derive(Signal)]
pub struct Pressed {
    entity: Entity
}

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



fn hello_world(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.add(
        eml! {
            resource(time, Time);
            Body {
                .on.enter: () => {
                    hello.text = "Hello, ";
                },
                .on.update: (e) => {
                    delta.text = e.delta.fmt("Frame time: {:0.4}");
                    elapsed.text = time.elapsed.fmt("Elapsed time: {:0.2}");
                    elapsed.bg.g = (time.elapsed - 2.) * 0.5;
                },
            } [
                Column [
                    Row [ hello: Label { .text: "..., " }, "world!" ],
                    delta: Label { .text: "0.0000" },
                    elapsed: Label { .text: "0.00" },
                ]
            ]
        }
    );


        //     // resource(time, Time);
        //     // bind(time.elapsed_seconds.fmt("{:0.2}") => elapsed.text);
        //     // bind(time.elapsed_seconds * 0.5 - 0.5 => content.bg.r);
        //     // bind(content.bg.hex => color.text);
        //     // Body + Name { .value: "body" } [
        //     //     content: Column { .bg: #9d9d9d, .s.padding: [25, 50] }[
        //     //         Div { .bg: #dedede, .s.padding: 50 } [
        //     //             "Hello world!"
        //     //         ],
        //     //         Row [
        //     //             "Elapsed: ", elapsed: Label { .text: "0.00" }
        //     //         ],
        //     //         Row [
        //     //             "Color: ", color: Label
        //     //         ]
        //     //     ]
        //     // ]
        // }
}

#[derive(Element)]
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

#[derive(Component, Behavior)]
pub struct UiText {
    /// The text value of UiText element.
    pub text: String,
    #[param(default = Color::hex("2f2f2f").unwrap())]
    pub text_color: Color,
}

#[derive(Element)]
#[construct(Label -> UiText -> Div)]
pub struct Label;
impl ElementBuilder for Label {
    fn build_element(_: Vec<Entity>) -> Blueprint<Self> {
        blueprint! {
            Label::Base + TextBundle
        }
    }
}

#[derive(Element)]
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

#[derive(Element)]
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

#[derive(Element)]
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
use polako_constructivism::Is;
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
    pub fn push_content<E: Element + Is<Div>>(
        &self,
        _: &mut World,
        content: &mut Vec<Entity>,
        model: EntityMark<E>,
    ) -> Implemented {
        content.push(model.entity);
        Implemented
    }
    /// Everything based on Div can access the styles using param extensions: `Row { .s.padding: 25 }`
    pub fn s(&self) -> &'static Styles {
        &Styles
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
