use bevy::prelude::*;
use polako_constructivism::*;
use polako_eml::*;
use polako_macro::*;

#[allow(dead_code)]
#[derive(Construct, Component)]
#[extends(Elem)]
pub struct Rect {
    size: (f32, f32),
}

impl rect_construct::Methods {
    pub fn push_text<'c, S: AsRef<str>>(
        &self,
        world: &mut World,
        content: &'c mut Vec<Entity>,
        text: S,
    ) -> Valid<&'c Entity> {
        content.push(world.spawn_empty().id());
        Valid(content.last().unwrap())
    }
}

impl Element for Rect {
    type Install = InstallRect;
}
pub struct InstallRect;
impl InstallElement for InstallRect {
    type Element = Rect;
    fn install(world: &mut World, this: Model<Self::Element>, content: Vec<Entity>) {}
}
pub struct AddTextToRect;

#[allow(dead_code)]
#[derive(Construct, Component)]
#[extends(Rect)]
pub struct Div {
    background: Color,
}
impl Element for Div {
    type Install = InstallDiv;
}
pub struct InstallDiv;
impl InstallElement for InstallDiv {
    type Element = Div;
    fn install(world: &mut World, this: Model<Self::Element>, content: Vec<Entity>) {
        world.entity_mut(this.entity).push_children(&content);
    }
}

fn main() {
    let x = eml! {
        // Styles;
        Div:root { background: Color::WHITE } [
            Div,
            Div:dude { background: Color::RED, size: (100., 100.) } [
                "With some text!"
            ],
            Div [
                Rect,
            ],
            Div { size: (100., 50.) }
        ]
        // Div [ "with text" ]
    };
    // let x =
}

mod test {
    use bevy::prelude::Component;

    fn test<C: Component>(c: C) {}
    #[derive(Component)]
    struct X {}
}
