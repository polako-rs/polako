// 0. Include polako eml & bevy
use bevy::prelude::*;
use polako::eml::*;

// 1. Define element.
#[derive(Component, Element)] // - You need to derive at least Component and Element
#[extends(Elem)]
// - You have to extend element from another element
//   Polako comes single `Elem` element out of the box
#[build(my_element)] // - You have to provide build function for Element
pub struct MyElement {}

fn my_element(BuildArgs { content, .. }: BuildArgs<MyElement>) -> Builder<Elem> {
    build! {
        Elem [[ content ]]      // bypass `Vec<Entity>` to the `Elem` builder
    }
}

// 2. You can add a tree of elements to the world.
fn step_a(mut commands: Commands) {
    commands.add(eml! {
        MyElement [             // MyElement can accept any other elements as content
            MyElement, MyElement, MyElement,
                                // But MyElement can't accept string literals as children
                                // the next won't compile:
            // "Hello world!"
        ]
    })
}

// 3. Polako eml uses constructivism inheritance and defines set of protocols
// to staticaly constraint the tree hierarchy. Protocols - are just static
// methods implemented for Element metadata. You can implement `push_text`
// protocol and element will accept string literals as content:

#[derive(Component)]
pub struct TextComponent {
    pub text: String,
}

#[derive(Component, Element)]
#[extends(Elem)]
#[build(my_node)]
pub struct MyNode {}

fn my_node(BuildArgs { content, .. }: BuildArgs<MyNode>) -> Builder<Elem> {
    build! {
        Elem [[ content ]]
    }
}

impl mynode_construct::Protocols {
    // `push_text` takes mut world, mut content and string.
    // You can spawn new entity and push it to content.
    pub fn push_text<'c, S: AsRef<str>>(
        &self,
        world: &mut World,
        content: &'c mut Vec<Entity>,
        text: S,
    ) -> Valid<()> {
        let entity = world
            .spawn(TextComponent {
                text: text.as_ref().to_string(),
            })
            .id();
        content.push(entity);
        Valid(())
    }
}

fn main() {
    let mut app = App::new();
    app.add_systems(Startup, step_a);
}
