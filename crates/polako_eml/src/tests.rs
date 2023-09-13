
use super::*;


#[derive(Component, Element)]
#[build(div)]
#[extends(Elem)]
pub struct Div {

}


fn div(In((_model, content)): BuildArgs<Div>) -> Eml<Elem> {
    eml! {
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

fn bold(In((model, content)): BuildArgs<Bold>) -> Eml<Div> {
    eml! {
        Div {{ model }} [[ content ]]
    }
}

#[derive(Component, Element)]
#[build(label)]
#[extends(Div)]
pub struct Label { text: String }

fn label(In((model, _content)): BuildArgs<Label>, mut commands: Commands) -> Eml<Div> {
    commands.entity(model.entity).insert(TextElement { text: "".to_string() });
    eml! { 
        Div {{ model }}
    }
}

fn update_label_system(
    models: Query<&Label, Changed<Label>>,
    mut views: Query<(&mut TextElement, &View<Label>)>
) {
    for (mut text, view) in views.iter_mut() {
        if let Ok(label) = models.get(view.entity) {
            text.text = label.text.clone();
        }
    }
}


#[derive(Component, Element)]
#[extends(Div)]
#[build(quote)]
pub struct Quote {

}

fn quote(In((model, content)): BuildArgs<Quote>) -> Eml<Div> {
    eml! {
        Div {{ model }} [
            "Quote:",
            Bold [[ content ]]
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
    eml.apply(&mut app.world);
    app.update();
    let world = &mut app.world;
    assert_eq!(1, world.query::<(&Quote, &Div)>().iter(world).len());
    // assert_eq!(1, world.query::<&TextElement>().iter(world).len());

}