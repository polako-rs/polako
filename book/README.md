eml behaviour
-------------

Lets say we have this elements definition:

```rust
// the Div itself
#[derive(Component, Element)]
#[extend(Elem)]
#[build(div)]
pub struct Div {
    #[default(Color::NONE)]
    background: Color
}

// the way we build div element
fn div(BuildArgs { content, mut ctx, ..}: BuildArgs<Div>) -> Builder<Elem> {
    // TODO: write and notice model-view relation
    // `ctx.view()` methods works with `view` entity, the one is adding to hierarchy at the moment
    // I cant insert other compoenents for `view` entity:
    ctx.view().insert(NodeBundle::default());
    build! {
        Elem [[ content ]]
    }
}

// everytime Div.background prop get changed =>
// bypass Div.background to BackgroundColor.0
fn div_system(
    mut colors: Query<(&mut BackgroundColor, &Div), Changed<Div>>
) {
    colors.for_each_mut(|(mut bg, div)| bg.0 = div.background);
}


// allow the Div to accept string literlas as children,
// thay becomes `TextBundle` at the right place.
impl div_construct::Protocols {
    pub fn push_text<'c, S: AsRef<str>>(
        &self,
        world: &mut World,
        content: &'c mut Vec<Entity>,
        text: S,
    ) -> Implemented {
        let entity = world.spawn(TextBundle::from_section(
            text.as_ref().to_string(),
            TextStyle { font_size: 24., color: Color::hex("2f2f2f").unwrap(), ..default() }
        )).id();
        content.push(entity);
        Implemented
    }
}


```