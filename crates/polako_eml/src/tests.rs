
use super::*;


#[derive(Component, Construct)]
#[extends(Elem)]
pub struct Div {

}

impl Element for Div {
    fn build_element(_: Model<Self>, content: Vec<Entity>) -> Blueprint<Self> {
        blueprint! {
            Div::Super [[ content ]]
        }
    }
}


#[derive(Component, Mixin)]
pub struct TextElement {
    pub text: String,
    #[default(format!("regular"))]
    pub font: String,
}

impl Default for TextElement {
    fn default() -> Self {
        TextElement { text: "".into(), font: "regular".into() }
    }
}

impl div_construct::Protocols {
    pub fn push_text<'c, S: AsRef<str>>(&self, world: &mut World, content: &'c mut Vec<Entity>, text: S) -> Implemented {
        let entity = world.spawn(TextElement { text: text.as_ref().to_string(), ..default() }).id();
        content.push(entity);
        Implemented
    }
}


#[derive(Component, Construct)]
#[extends(Div)]
#[mixin(TextElement)]
pub struct Label;

impl Element for Label {
    fn build_element(_: Model<Self>, _: Vec<Entity>) -> Blueprint<Self> {
        blueprint! { 
            Label::Super
        }
    }
}

#[derive(Component, Construct)]
#[extends(Label)]
pub struct Bold { }
impl Element for Bold {
    fn build_element(_: Model<Self>, _: Vec<Entity>) -> Blueprint<Self> {
        blueprint! {
            Bold::Super + TextElement(font: "bold")
        }
    }
}

// #[derive(Component, Element)]
// #[extends(Div)]
// #[build(quote)]
// pub struct Quote {

// }

// fn quote(BuildArgs { content, .. }: BuildArgs<Quote>) -> Builder<Div> {
//     build! {
//         Div [
//             Bold ["Quote:"],
//             Div [[ content ]]
//         ]
//     }
// }

// #[derive(Component, Element)]
// #[extends(Label)]
// #[build(field)]
// pub struct Field {
//     label: String
// }

// fn field(BuildArgs { model, mut ctx, .. }: BuildArgs<Field>) -> Builder<Div> {
//     let label = ctx.model().component::<Field>().label.clone();
//     build! {
//         Div [
//             Label { text: label },
//             Label {{ model }}
//         ]
//     }
// }

#[test]
fn test_div_with_text() {
    let mut app = App::new();
    let eml = eml! { Div [ "text" ] };
    eml.apply(&mut app.world);
    let world = &mut app.world;
    assert_eq!(1, world.query::<&Div>().iter(world).len());
    assert_eq!(1, world.query::<&TextElement>().iter(world).len());
    assert_eq!("text", world.query::<&TextElement>().single(world).text);
    let child = world.query_filtered::<Entity, With<TextElement>>().single(world);
    let children = world.query_filtered::<&Children, With<Div>>().single(world);
    assert_eq!(1, children.len());
    assert_eq!(children[0], child);
}


#[test]
fn test_labels() {
    let mut app = App::new();
    let eml = eml! { Label { text: "text" } };
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
    let eml = eml! { Bold { text: "some bold text" } };
    eml.apply(&mut app.world);
    let world = &mut app.world;
    assert_eq!(1, world.query::<(&Bold, &Label, &TextElement, &Div)>().iter(world).len());
    assert_eq!("some bold text", world.query::<&TextElement>().single(world).text);
    assert_eq!("bold", &world.query::<&TextElement>().single(world).font);
}


#[derive(Component, Construct)]
#[extends(Div)]
pub struct UiNode { }
impl Element for UiNode {
    fn build_element(_: Model<Self>, _: Vec<Entity>) -> Blueprint<Self> {
        blueprint!{ UiNode::Super + NodeBundle }
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
    value: String
}
#[derive(Component, Construct)]
#[extends(Div)]
struct MixPatch;
impl Element for MixPatch {
    fn build_element(_: Model<Self>, _: Vec<Entity>) -> Blueprint<Self> {
        blueprint! {
            MixPatch::Super + TestComponent(value: "mix_patch")
        }
    }
}
#[test]
fn test_blueprint_mix_patch() {
    let mut app = App::new();
    let eml = eml! { MixPatch };
    eml.apply(&mut app.world);
    let world = &mut app.world;
    assert_eq!(1, world.query::<(&MixPatch, &TestComponent)>().iter(world).len());
    assert_eq!("mix_patch", &world.query::<&TestComponent>().single(world).value);
}
#[derive(Component, Construct)]
#[extends(Div)]
struct MixConstruct;
impl Element for MixConstruct {
    fn build_element(_: Model<Self>, _: Vec<Entity>) -> Blueprint<Self> {
        blueprint! {
            MixConstruct::Super + Name { value: "mix_construct" }
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
    assert_eq!("mix_construct", world.query::<&Name>().single(world).as_str());

}

#[test]
fn test_eml_mix_construct() {
    let mut app = App::new();
    let eml = eml! { Div + Name { value: "hello" } };
    eml.apply(&mut app.world);
    let world = &mut app.world;
    assert_eq!(1, world.query::<(&Div, &Name)>().iter(world).len());
    assert_eq!("hello", world.query::<&Name>().single(world).as_str());
}


#[test]
fn test_eml_mix_patch() {
    let mut app = App::new();
    let eml = eml! { Div + TestComponent(value: "world") };
    eml.apply(&mut app.world);
    let world = &mut app.world;
    assert_eq!(1, world.query::<(&Div, &TestComponent)>().iter(world).len());
    assert_eq!("world", &world.query::<&TestComponent>().single(world).value);
}


// #[test]
// fn test_quote() {
//     let mut app = App::new();
//     app.add_systems(Update, update_label_system);
//     let eml = eml! { Quote [ "War never changes" ] };
//     let root = app.world.spawn_empty().id();
//     eml.write(&mut app.world, root);
//     app.update();
//     let world = &mut app.world;
//     assert_eq!(1, world.query::<(&Quote, &Div)>().iter(world).len());
//     assert_eq!(0, world.query::<(&Bold, &Div, &TextElement)>().iter(world).len());
//     assert_eq!(0, world.query::<(&Div, &TextElement)>().iter(world).len());

//     let root = world.entity(root);
//     let children = root.get::<Children>().unwrap();
//     assert_eq!(2, children.len());
//     let bold = world.entity(children[0]);
//     let bold_children = bold.get::<Children>().unwrap();
//     assert_eq!(1, bold_children.len());
//     let quote_label = world.entity(bold_children[0]).get::<TextElement>().unwrap();
//     assert_eq!(&quote_label.text, "Quote:");
//     let body_children = world.entity(children[1]).get::<Children>().unwrap();
//     assert_eq!(1, body_children.len());
//     let quote_content = world.entity(body_children[0]).get::<TextElement>().unwrap();
//     assert_eq!(&quote_content.text, "War never changes");
// }

// #[test]
// fn test_field() {
//     let mut app = App::new();
//     app.add_systems(Update, update_label_system);
//     let eml = eml! { Field { label: "hello", text: "world" } };
//     let root = app.world.spawn_empty().id();
//     eml.write(&mut app.world, root);
//     app.update();
//     let world = &mut app.world;
//     assert_eq!(2, world.query::<&TextElement>().iter(world).len());
//     assert_eq!(2, world.query::<&View<Label>>().iter(world).len());
//     let children = world.entity(root).get::<Children>().unwrap();
//     println!("children: {children:?}");
//     assert_eq!(2, children.len());
//     let t0 = world.entity(children[0]).get::<TextElement>();
//     let t1 = world.entity(children[1]).get::<TextElement>();
//     assert!(t0.is_some());
//     assert!(t1.is_some());
//     assert_eq!(t0.unwrap().text, "hello");
//     assert_eq!(t1.unwrap().text, "world");
//     let root_text = world.entity(root).get::<TextElement>();
//     assert!(root_text.is_none());

// }