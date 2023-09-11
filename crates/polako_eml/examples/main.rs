use polako_constructivism::*;
use polako_macro::*;
use polako_eml::*;
use bevy::prelude::*;

#[allow(dead_code)]
#[derive(Construct, Component)]
pub struct Rect { 
    size: (f32, f32),
}

impl Element for Rect {
    type Install = InstallRect;
    type PushText = AddTextToRect;
}
pub struct InstallRect;
impl InstallElement for InstallRect {
    type Element = Div;
    fn install(world: &mut World, this: Model<Self::Element>, content: Vec<Entity>) {
        
    }
}
pub struct AddTextToRect;
impl PushText for AddTextToRect {
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
    type Install = InstallDiv;
    type PushText = <Self::Extends as Element>::PushText;
}
pub struct InstallDiv;
impl InstallElement for InstallDiv {
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



fn main() {
    let x = eml! {
        // Styles;
        Div:root { background: Color::WHITE } [
            Div,
            Div:dude { background: Color::RED, size: (100., 100.) } [
                "With some text!"
            ]
        ]
        // Div [ "with text" ]
    };
    // let x = 
    
}