

use bevy::prelude::*;
use polako::eml::*;
use polako_constructivism::Is;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, ui_text_system)
        .add_systems(Update, div_system)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    let primary = Color::hex("9f9f9f").unwrap();
    let secondary = Color::hex("dfdfdf").unwrap();
    commands.add(eml! {
        Body [ 
            Column { background: primary, padding: 10. } [  
                Div { background: secondary, padding: 5. } [
                    "Hello world!"
                ],
                "This is awesome!",
                Row [ "T", "h", "i", "s", " ", "i", "s"],
                Row [ "A", "W", "E", "S", "O", "M", "E", "!"]
            ]
        ]
    });
}

// #[element(Elem)]
#[derive(Component, Construct)]
#[extend(Elem)]
pub struct Div {
    #[default(Color::NONE)]
    background: Color,
    padding: f32,
}
impl Element for Div {
    fn build_element(content: Vec<Entity>) -> Blueprint<Self> {
        blueprint! {
            Div::Super
            + NodeBundle
            [[ content ]]
        }
    }
}

#[derive(Component, Mixin)]
pub struct UiText {
    pub text: String,
    #[default(Color::hex("2f2f2f").unwrap())]
    pub text_color: Color
}

// #[element(Div + UiText)]
#[derive(Component, Construct)]
#[extend(Div)]
#[mix(UiText)]
pub struct Label;
impl Element for Label {
    fn build_element(_: Vec<Entity>) -> Blueprint<Self> {
        blueprint! {
            Label::Super + TextBundle
        }
    }
}

#[derive(Component, Construct)]
#[extend(Div)]
pub struct Body;
impl Element for Body {
    fn build_element(content: Vec<Entity>) -> Blueprint<Self> {
        blueprint! { 
            Body::Super
            + Style(
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
            )
            [[ content ]]
        }
    }
}

#[derive(Component, Construct)]
#[extend(Div)]
pub struct Column;
impl Element for Column {
    fn build_element(content: Vec<Entity>) -> Blueprint<Self> {
        blueprint! {
            Column::Super
            + Style (
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                align_content: AlignContent::Center,
                row_gap: Val::Px(3.),
            )
            [[ content ]]
        }
    }
}

#[derive(Component, Construct)]
#[extend(Div)]
pub struct Row;
impl Element for Row {
    fn build_element(content: Vec<Entity>) -> Blueprint<Self> {
        blueprint! { 
            Row::Super
            + Style (
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                align_content: AlignContent::Center,
                column_gap: Val::Px(3.),
            )
            [[ content ]]
        }
    }
}

impl div_construct::Protocols {
    // Div can accpet string literals as content
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
    // Only Div and elements extends Div can be content of the Div
    pub fn push_content<E: Element + Is<Div>>(&self, _: &mut World, content: &mut Vec<Entity>, model: Model<E>) -> Implemented {
        content.push(model.entity);
        Implemented
    }
}

// helpers for spawning text bundle
impl Default for UiText {
    fn default() -> Self {
        UiText { text: "".into(), text_color: Color::hex("2f2f2f").unwrap() }
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

/// bypase Div.background to BackgroundColor.0 when changed
/// and Div.padding to Style.padding
fn div_system(
    mut colors: Query<(&Div, &mut Style, &mut BackgroundColor,), Changed<Div>>
) {
    colors.for_each_mut(|(div, mut style, mut bg)| {
        bg.0 = div.background;
        style.padding = UiRect::all(Val::Px(div.padding));
    });
}

/// bypass UiText text value & color to Text.sections[0] when changed
fn ui_text_system(
    mut texts: Query<(&UiText, &mut Text), Changed<UiText>>
) {
    for (ui_text, mut text) in texts.iter_mut() {
        if text.sections.is_empty() {
            *text = Text::with_text("");
        }
        text.sections[0].value = ui_text.text.clone();
        text.sections[0].style.color = ui_text.text_color;
    }
}