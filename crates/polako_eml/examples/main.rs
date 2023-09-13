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
    ) -> Valid<()> {
        content.push(world.spawn_empty().id());
        Valid(())
    }
}

impl Element for Rect {
    type Build = InstallRect;
}
pub struct InstallRect;
impl Build for InstallRect {
    type Element = Rect;
    fn build(world: &mut World, this: Entity, model: EntityComponent<Self::Element>, content: Vec<Entity>) {}
}

#[allow(dead_code)]
#[derive(Component, Element)]
#[build(div)]
#[extends(Rect)]
pub struct Div {
    background: Color,
}


fn div(
    In((this, model, content)): BuildArgs<Div>
) -> Builder<Rect> {
    Builder::new(Eml::new(|_, _| { }))
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
