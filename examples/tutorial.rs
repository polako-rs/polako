// // 0. Include polako eml & bevy
// use bevy::prelude::*;
// use polako::eml::*;

// // 1. Define element.
// #[rustfmt::skip]
// #[derive(Component, Element)]   // - You need to derive at least Component and Element
// #[extend(Elem)]                // - You have to extend element from another element
//                                 //   Polako comes single `Elem` element out of the box
// #[build(my_element)]            // - You have to provide build function for Element
// pub struct MyElement {}

// fn my_element(BuildArgs { content, .. }: BuildArgs<MyElement>) -> Builder<Elem> {
//     build! {
//         Elem [[ content ]]      // bypass `Vec<Entity>` to the `Elem` builder
//     }
// }

// // 2. You can add a tree of elements to the world.
// #[allow(dead_code)]
// fn step_a(mut commands: Commands) {
//     commands.add(eml! {
//         MyElement [             // MyElement can accept any other elements as content
//             MyElement, MyElement, MyElement,
//                                 // But MyElement can't accept string literals as children
//                                 // the next won't compile with error message
//                                 // expected `Implemented`, found `NotImplemented<TextAsChild>`
//             // "Hello world!"
//         ]
//     })
// }

// // 3. Polako eml uses constructivism inheritance and defines set of protocols
// // to staticaly constraint the tree hierarchy. Protocols - are just static
// // methods implemented for Element metadata. You can implement `push_text`
// // protocol and element will accept string literals as content:

// #[derive(Component)]
// pub struct TextComponent {
//     pub text: String,
// }

// #[derive(Component, Element)]
// #[extend(Elem)]
// #[build(my_node)]
// pub struct MyNode {}

// fn my_node(BuildArgs { content, .. }: BuildArgs<MyNode>) -> Builder<Elem> {
//     build! {
//         Elem [[ content ]]
//     }
// }

// impl mynode_construct::Protocols {
//     // `push_text` takes mut world, mut content and string.
//     // You can spawn new entity and push it to content.
//     pub fn push_text<'c, S: AsRef<str>>(
//         &self,
//         world: &mut World,
//         content: &'c mut Vec<Entity>,
//         text: S,
//     ) -> Implemented {
//         let entity = world
//             .spawn(TextComponent {
//                 text: text.as_ref().to_string(),
//             })
//             .id();
//         content.push(entity);
//         Implemented
//     }
// }

// #[allow(dead_code)]
// fn step_3(mut commands: Commands) {
//     commands.add(eml! {
//         MyNode [
//             "With text and..",
//             MyElement,
//             "..ther nodes."
//         ]
//     })
// }

// // 4. The ability to accept elements as content contolled by `push_content`
// // protocol. You can override it and forbid any content for example. Or use
// // `AcceptNoContent` mixin provided by polako
// #[derive(Component, Element)]
// #[extend(Elem)]
// #[mix(AcceptNoContent)]
// #[build(dead_end)]
// pub struct DeadEnd;

// fn dead_end(BuildArgs { .. }: BuildArgs<DeadEnd>) -> Builder<Elem> {
//     build! { Elem }
// }

// #[allow(dead_code)]
// fn step_4(mut commands: Commands) {
//     commands.add(eml! {
//         MyNode [
//             DeadEnd [
//                 // uncomment to get 'expected `Implemented`, found `NotImplemented<ElementAsContent>`'
//                 // MyNode
//             ]
//         ]
//     })
// }

// // 5. You can assign values to the fields from the `eml!`. Becouse of constructivism, it is possible
// // to pass values to every component fields from the single definition.
// #[derive(Component, Element)]
// #[extend(MyElement)]
// #[build(rect)]
// pub struct Rect {
//     pub position: Vec2,
//     pub size: Vec2
// }

// fn rect(BuildArgs { content, .. }: BuildArgs<Rect>) -> Builder<Elem> {
//     build! { Elem [[ content ]] }
// }
// #[derive(Component, Element)]
// #[extend(Rect)]
// #[build(div)]
// pub struct Div {
//     pub background: Color
// }
// fn div(BuildArgs { content, .. }: BuildArgs<Div>) -> Builder<Rect> {
//     build! { Rect [[ content ]] }
// }

// #[allow(dead_code)]
// fn step_5(mut commands: Commands) {
//     commands.add(eml! {
//         Div { background: Color::RED, size: Vec2::splat(50.)} [
//             Div { background: Color::WHITE, position: Vec2::splat(50.)},
//             Rect { size: Vec2::splat(10.) }
//         ]
//     })
// }

// fn main() {
//     // let mut app = App::new();

// }
fn main() {}
