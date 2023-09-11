use polako_constructivism::*;
use polako_macro::*;
use polako_eml::*;
use bevy::prelude::*;

#[allow(dead_code)]
#[derive(Construct, Component)]
pub struct Rect { 
    hello: String
}

impl Element for Rect {
    type ContentType = Entity;
    type Install = InstallRect;
    type PushText = AddTextToRect;
}
pub struct InstallRect;
impl InstallElement<Entity> for InstallRect {
    type Element = Div;
    fn install(world: &mut World, this: Model<Self::Element>, content: Vec<Entity>) {
        
    }
}
pub struct AddTextToRect;
impl PushText<Entity> for AddTextToRect {
    fn push_text<'c, S: AsRef<str>>(world: &mut World, content: &'c mut Vec<Entity>, text: S) -> &'c Entity {
        content.push(world.spawn_empty().id());
        content.last().unwrap()
    }
}

#[allow(dead_code)]
#[derive(Construct, Component)]
#[extends(Rect)]
pub struct Div {
    background: Color
}
impl Element for Div {
    type ContentType = <Self::Extends as Element>::ContentType;
    type Install = InstallDiv;
    type PushText = <Self::Extends as Element>::PushText;
}
pub struct InstallDiv;
impl InstallElement<Entity> for InstallDiv {
    type Element = Div;
    fn install(world: &mut World, this: Model<Self::Element>, content: Vec<Entity>) {
        let mut func = IntoSystem::into_system(div);
        func.initialize(world);
        let command = func.run((this, content), world);
        command.apply(world, this.entity);
    }
}

fn div(
    In((this, content)):In<(Model<Div>,Vec<Entity>)>
    
) -> Eml {
    Eml::new(|_, _| { })
}


pub struct StyleSheet;

impl IntoContent<StyleSheet> for StyleSheet {
    fn into_content(_world: &mut World, this: Self) -> StyleSheet {
        this
    }
}

#[derive(Component)]
pub struct Styles(Vec<StyleSheet>);
constructable!(Styles() Self(vec![]) );

pub struct InvalidOperation;

impl NewContent for StyleSheet {
    type Output = InvalidOperation;
    fn new_content(_: &mut World) -> Self::Output {
        InvalidOperation
    }
}

impl Element for Styles {
    type ContentType = StyleSheet;
    type Install = InstallStyles;
    type PushText = AddTextToStylles;
}

pub struct InstallStyles;
impl InstallElement<StyleSheet> for InstallStyles {
    type Element = Styles;
    fn install(world: &mut World, this: Model<Self::Element>, content: Vec<StyleSheet>) {
        let mut func = IntoSystem::into_system(styles);
        func.initialize(world);
        let command = func.run((this, content), world);
        command.apply(world, this.entity);
    }
}
pub struct AddTextToStylles;
impl PushText<StyleSheet> for AddTextToStylles {
    fn push_text<'c, S: AsRef<str>>(_world: &mut World, content: &'c mut Vec<StyleSheet>, text: S) -> &'c StyleSheet {
        content.push(StyleSheet);
        content.last().unwrap()
    }
}

fn styles(
    In((this, content)):In<(Model<Styles>,Vec<StyleSheet>)>
) -> Eml {
    Eml::new(|_,_|{ })
}

fn main() {
    let x = eml! {
        // Styles;
        Div { background: Color::WHITE } [
            Div,
            Div { background: Color::RED }
        ]
    };
    // let x = 
    
}