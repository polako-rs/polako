
use super::*;


#[derive(Component, Element)]
#[build(div)]
#[extends(Elem)]
pub struct Div {

}


fn div(BuildArgs { content, .. }: BuildArgs<Div>) -> Builder<Elem> {
    build! {
        Elem [[ content ]]
    }
}


impl div_construct::Methods {
    pub fn push_text<'c, S: AsRef<str>>(&self, world: &mut World, content: &'c mut Vec<Entity>, text: S) -> Valid<()> {
        let entity = world.spawn(TextElement { text: text.as_ref().to_string() }).id();
        content.push(entity);
        Valid(())
    }
}

#[derive(Component)]
pub struct TextElement {
    pub text: String
}

#[derive(Component, Element)]
#[build(bold)]
#[extends(Div)]
pub struct Bold { }

fn bold(BuildArgs { content, .. }:  BuildArgs<Bold>) -> Builder<Div> {
    build! {
        Div [[ content ]]
    }
}

#[derive(Component, Element)]
#[build(label)]
#[extends(Div)]
pub struct Label { text: String }

fn label(BuildArgs { model, mut ctx, .. }: BuildArgs<Label>) -> Builder<Div> {
    ctx.insert(TextElement { text: "".to_string() });
    build! { 
        Div(model)
    }
}

fn update_label_system(
    models: Query<(&Label, &Model<Label>), Changed<Label>>,
    mut views: Query<&mut TextElement>
) {
    for (label, model) in models.iter() {
        if let Ok(mut elem) = views.get_mut(model.for_view) {
            elem.text = label.text.clone();
        }
    }
}


#[derive(Component, Element)]
#[extends(Div)]
#[build(quote)]
pub struct Quote {

}

fn quote(BuildArgs { content, .. }: BuildArgs<Quote>) -> Builder<Div> {
    build! {
        Div [
            Bold ["Quote:"],
            Div [[ content ]]
        ]
    }
}

#[derive(Component, Element)]
#[extends(Label)]
#[build(field)]
pub struct Field {
    label: String
}

fn field(BuildArgs { model, mut ctx, .. }: BuildArgs<Field>) -> Builder<Div> {
    let label = ctx.component::<Field>().label.clone();
    build! {
        Div [
            Label { text: label },
            Label {{ model }}
        ]
    }
}

#[test]
fn test_div_with_text() {
    let mut app = App::new();
    app.add_systems(Update, update_label_system);
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
fn test_bold_with_text() {
    let mut app = App::new();
    app.add_systems(Update, update_label_system);
    let eml = eml! { Bold [ "text" ] };
    eml.apply(&mut app.world);
    let world = &mut app.world;
    assert_eq!(1, world.query::<&Div>().iter(world).len());
    assert_eq!(1, world.query::<&Bold>().iter(world).len());
    assert_eq!(1, world.query::<&TextElement>().iter(world).len());
    assert_eq!("text", world.query::<&TextElement>().single(world).text);
    let child = world.query_filtered::<Entity, With<TextElement>>().single(world);
    let children = world.query_filtered::<&Children, With<Bold>>().single(world);
    assert_eq!(1, children.len());
    assert_eq!(children[0], child);
}

#[test]
fn test_labels() {
    let mut app = App::new();
    app.add_systems(Update, update_label_system);
    let eml = eml! { Label { text: "text" } };
    eml.apply(&mut app.world);
    app.update();
    let world = &mut app.world;
    assert_eq!(1, world.query::<&Label>().iter(world).len());
    assert_eq!(1, world.query::<&TextElement>().iter(world).len());
    assert_eq!(1, world.query::<(&TextElement, &Label)>().iter(world).len());
    assert_eq!("text", world.query::<&TextElement>().single(world).text);
    assert_eq!("text", world.query::<&Label>().single(world).text);    
}

#[test]
fn test_quote() {
    let mut app = App::new();
    app.add_systems(Update, update_label_system);
    let eml = eml! { Quote [ "War never changes" ] };
    let root = app.world.spawn_empty().id();
    eml.write(&mut app.world, root);
    app.update();
    let world = &mut app.world;
    assert_eq!(1, world.query::<(&Quote, &Div)>().iter(world).len());
    assert_eq!(0, world.query::<(&Bold, &Div, &TextElement)>().iter(world).len());
    assert_eq!(0, world.query::<(&Div, &TextElement)>().iter(world).len());

    let root = world.entity(root);
    let children = root.get::<Children>().unwrap();
    assert_eq!(2, children.len());
    let bold = world.entity(children[0]);
    let bold_children = bold.get::<Children>().unwrap();
    assert_eq!(1, bold_children.len());
    let quote_label = world.entity(bold_children[0]).get::<TextElement>().unwrap();
    assert_eq!(&quote_label.text, "Quote:");
    let body_children = world.entity(children[1]).get::<Children>().unwrap();
    assert_eq!(1, body_children.len());
    let quote_content = world.entity(body_children[0]).get::<TextElement>().unwrap();
    assert_eq!(&quote_content.text, "War never changes");
}

#[test]
fn test_field() {
    let mut app = App::new();
    app.add_systems(Update, update_label_system);
    let eml = eml! { Field { label: "hello", text: "world" } };
    let root = app.world.spawn_empty().id();
    eml.write(&mut app.world, root);
    app.update();
    let world = &mut app.world;
    assert_eq!(2, world.query::<&TextElement>().iter(world).len());
    assert_eq!(2, world.query::<&View<Label>>().iter(world).len());
    let children = world.entity(root).get::<Children>().unwrap();
    println!("children: {children:?}");
    assert_eq!(2, children.len());
    let t0 = world.entity(children[0]).get::<TextElement>();
    let t1 = world.entity(children[1]).get::<TextElement>();
    assert!(t0.is_some());
    assert!(t1.is_some());
    assert_eq!(t0.unwrap().text, "hello");
    assert_eq!(t1.unwrap().text, "world");
    let root_text = world.entity(root).get::<TextElement>();
    assert!(root_text.is_none());

}