use bevy::ecs::component::TableStorage;

use super::*;

pub struct XXX;
impl Component for XXX {
    type Storage = TableStorage;
}

#[derive(Element)]
#[construct(Div -> Empty)]
pub struct Div {}

impl ElementBuilder for Div {
    fn build_element(content: Vec<Entity>) -> Blueprint<Self> {
        blueprint! {
            Div::Base [[ content ]]
        }
    }
}

#[derive(Behavior)]
pub struct TextElement {
    pub text: String,
    #[param(default = format!("regular"))]
    pub font: String,
}

impl Default for TextElement {
    fn default() -> Self {
        TextElement {
            text: "".into(),
            font: "regular".into(),
        }
    }
}

impl DivDesign {
    pub fn push_text<'c, S: AsRef<str>>(
        &self,
        world: &mut World,
        content: &'c mut Vec<Entity>,
        text: S,
    ) -> Implemented {
        let entity = world
            .spawn(TextElement {
                text: text.as_ref().to_string(),
                ..default()
            })
            .id();
        content.push(entity);
        Implemented
    }
}

#[derive(Element)]
#[construct(Label -> TextElement -> Div)]
pub struct Label;

impl ElementBuilder for Label {
    fn build_element(_: Vec<Entity>) -> Blueprint<Self> {
        blueprint! {
            Label::Base
        }
    }
}

#[derive(Element)]
#[construct(Bold -> Label)]
pub struct Bold {}
impl ElementBuilder for Bold {
    fn build_element(_: Vec<Entity>) -> Blueprint<Self> {
        blueprint! {
            Bold::Base + TextElement(.font: "bold")
        }
    }
}

#[test]
fn test_div_with_text() {
    let mut app = App::new();
    let eml = eml! { Div [ "text" ] };
    eml.apply(&mut app.world);
    let world = &mut app.world;
    assert_eq!(1, world.query::<&Div>().iter(world).len());
    assert_eq!(1, world.query::<&TextElement>().iter(world).len());
    assert_eq!("text", world.query::<&TextElement>().single(world).text);
    let child = world
        .query_filtered::<Entity, With<TextElement>>()
        .single(world);
    let children = world.query_filtered::<&Children, With<Div>>().single(world);
    assert_eq!(1, children.len());
    assert_eq!(children[0], child);
}

#[test]
fn test_labels() {
    let mut app = App::new();
    let eml = eml! { Label { .text: "text" } };
    eml.apply(&mut app.world);
    app.update();
    let world = &mut app.world;
    assert_eq!(1, world.query::<&Label>().iter(world).len());
    assert_eq!(1, world.query::<&TextElement>().iter(world).len());
    assert_eq!(1, world.query::<(&TextElement, &Label)>().iter(world).len());
    assert_eq!("text", world.query::<&TextElement>().single(world).text);
}

#[test]
fn test_bold_text() {
    let mut app = App::new();
    let eml = eml! { Bold { .text: "some bold text" } };
    eml.apply(&mut app.world);
    let world = &mut app.world;
    assert_eq!(
        1,
        world
            .query::<(&Bold, &Label, &TextElement, &Div)>()
            .iter(world)
            .len()
    );
    assert_eq!(
        "some bold text",
        world.query::<&TextElement>().single(world).text
    );
    assert_eq!("bold", &world.query::<&TextElement>().single(world).font);
}

#[derive(Element)]
#[construct(UiNode -> Div)]
pub struct UiNode {}
impl ElementBuilder for UiNode {
    fn build_element(_: Vec<Entity>) -> Blueprint<Self> {
        blueprint! { UiNode::Base + NodeBundle }
    }
}
#[test]
fn test_blueprint_patch_self() {
    let mut app = App::new();
    let eml = eml! { UiNode };
    eml.apply(&mut app.world);
    let world = &mut app.world;
    assert_eq!(1, world.query::<(&UiNode, &Div, &Node)>().iter(world).len());
}
#[derive(Component, Default)]
struct TestComponent {
    value: String,
}
#[derive(Element)]
#[construct(MixPatch -> Div)]
pub struct MixPatch;
impl ElementBuilder for MixPatch {
    fn build_element(_: Vec<Entity>) -> Blueprint<Self> {
        blueprint! {
            MixPatch::Base + TestComponent(.value: "mix_patch")
        }
    }
}
#[test]
fn test_blueprint_mix_patch() {
    let mut app = App::new();
    let eml = eml! { MixPatch };
    eml.apply(&mut app.world);
    let world = &mut app.world;
    assert_eq!(
        1,
        world
            .query::<(&MixPatch, &TestComponent)>()
            .iter(world)
            .len()
    );
    assert_eq!(
        "mix_patch",
        &world.query::<&TestComponent>().single(world).value
    );
}
#[derive(Element)]
#[construct(MixConstruct -> Div)]
pub struct MixConstruct;
impl ElementBuilder for MixConstruct {
    fn build_element(_: Vec<Entity>) -> Blueprint<Self> {
        blueprint! {
            MixConstruct::Base + Name { .value: "mix_construct" }
        }
    }
}
#[test]
fn test_blueprint_mix_construct() {
    let mut app = App::new();
    let eml = eml! { MixConstruct };
    eml.apply(&mut app.world);
    let world = &mut app.world;
    assert_eq!(1, world.query::<(&MixConstruct, &Name)>().iter(world).len());
    assert_eq!(
        "mix_construct",
        world.query::<&Name>().single(world).as_str()
    );
}

#[test]
fn test_eml_mix_construct() {
    let mut app = App::new();
    let eml = eml! { Div + Name { .value: "hello" } };
    eml.apply(&mut app.world);
    let world = &mut app.world;
    assert_eq!(1, world.query::<(&Div, &Name)>().iter(world).len());
    assert_eq!("hello", world.query::<&Name>().single(world).as_str());
}

#[test]
fn test_eml_mix_patch() {
    let mut app = App::new();
    let eml = eml! { Div + TestComponent(.value: "world") };
    eml.apply(&mut app.world);
    let world = &mut app.world;
    assert_eq!(1, world.query::<(&Div, &TestComponent)>().iter(world).len());
    assert_eq!(
        "world",
        &world.query::<&TestComponent>().single(world).value
    );
}
