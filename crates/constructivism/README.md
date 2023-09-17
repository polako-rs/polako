### About
`constructivism_macro` is a crate that allows to construct complex structures within single call. It is also charged with simple compile-time meta-inheritance model with mixins.

It is at very early proof-of-cocept stage for now. But with bright future.

### [Tutorial](./examples/tutorial.rs)
0. Use constructivism_macro
```rust
use constructivism_macro::*;
```

1. You can derive `Construct` now
```rust
#[derive(Construct)]
pub struct Node {
    // You can provide custom default values.
    #[default(true)]
    visible: bool,
    position: (f32, f32),
}
```

2. You can use `construct!` macro for instancing the Node.
```rust
fn step_01() {
    let node = construct!(Node {
        position: (10., 10.),
        visible: true
    });
    assert_eq!(node.position.0, 10.);
    assert_eq!(node.visible, true);
}
```

3. You can skip declaration of default values
```rust
fn step_03() {
    let node = construct!(Node {
        visible: false
    });
    assert_eq!(node.position.0, 0.)
}
```

4. You have to mark non-default required fields with `#[required]` or you get compilation error.
```rust
pub struct Entity(usize);

#[derive(Construct)]
struct Reference {
    #[required]
    target: Entity,
    count: usize,
}
```

5. You have to pass required field to `construct!(..)`` or you get compilation error
```rust
fn step_05() {
    let reference = construct!(Reference {
        target: Entity(23)
    });
    assert_eq!(reference.target.0, 23);
    assert_eq!(reference.count, 0);
}
```

6. You derive Construct using `constructable! { .. }`, define custom params and provide custom constructor. `min: f32 = 0.` syntax defines min param with default value of 0. If you doesn't provide default value, this param counts as required.
```rust
pub struct Range {
    min: f32,
    val: f32,
    max: f32,
}

constructable! { 
    Range(min: f32 = 0., max: f32 = 1., val: f32 = 0.) {
        if max < min {
            max = min;
        }
        val = val.min(max).max(min);
        Self { min, val, max }
    }
}
```

7. Provided constructor will be called for instancing Range
```rust
fn step_07() {
    let range = construct!(Range {
        val: 100.
    });
    assert_eq!(range.min, 0.);
    assert_eq!(range.max, 1.);
    assert_eq!(range.val, 1.);
}
```

8. You can extend one construct from another construct
```rust
#[derive(Construct)]
#[extend(Node)]
pub struct Rect {
    #[default((100., 100.))]
    size: (f32, f32)
}
```

9. You can pass params for all structs in inheritance branch with single call
```rust
fn step_09() {
    let (rect, node) = construct!(Rect {
        position: (10., 10.),
        size: (10., 10.),
    });
    assert_eq!(rect.size.0, 10.);
    assert_eq!(node.position.1, 10.);
    assert_eq!(node.visible, true);
}
```

10. You can derive Mixin as well.
```rust
#[derive(Mixin)]
pub struct Input {
    disabled: bool
}
```

11. You can inject mixins into constructs:
```rust
#[derive(Construct)]
#[extend(Rect)]
#[mix(Input)]
pub struct Button {
    pressed: bool
}
```

12. You can pass arguments to inheritance tree (with mixins) as well
```rust
fn step_12() {
    let (button, input, rect, node) = construct!(Button {
        disabled: true
    });
    assert_eq!(button.pressed, false);
    assert_eq!(input.disabled, true);
    assert_eq!(rect.size.0, 100.);
    assert_eq!(node.position.0, 0.);
}
```

13. When you extend from other construct, you extend from its mixins as well.
```rust
#[derive(Construct)]
#[extend(Button)]
pub struct Radio {
    #[required]
    value: String
}
fn step_13() {
    let (radio, button, input, rect, node) = construct!(Radio {
        value: "option_0"
    });
    assert_eq!(button.pressed, false);
    assert_eq!(input.disabled, false);
    assert_eq!(rect.size.0, 100.);
    assert_eq!(node.position.0, 0.);
    assert_eq!(radio.value, "option_0".to_string());
}
```
14. You can implement static protocols. It will be accesable for all inherited items.
```rust
// Implement protocols for Node

impl node_construct::Protocols {
    #[allow(unused_variables)]
    pub fn add_child(&self, entity: Entity) {
    }
}

fn step_14() {
    // It is accessable from Button as well as from
    // any item that extends Node directly or indirectly
    protocols!(Button).add_child(Entity(23));
}
```

15. You can check if construct extends other construct at any level with `Extends<T>` trait
```rust
fn takes_everything_that_extends_node<T: Extends<Node>>(_: T) { }
fn step_15() {
    let (button, input, rect, node) = construct!(Button { disabled: true });
    takes_everything_that_extends_node(rect);
    takes_everything_that_extends_node(button);

    // won't compile: Extends<T> respects only Constructs, not Mixins
    // takes_everything_that_extends_node(input);

    // won't compile: Node doesn't extends Node
    // takes_everything_that_extends_node(node);

    assert_eq!(input.disabled, true);
    assert_eq!(node.position.0, 0.);
}
```


### Upcoming features

- docstring bypassing

- `mixable! { ... }`, just like `constructable! { ... }`

- union props

- generics (generic structs not supported yet)

- nested results, like 
```rust
let (radio, base) = construct!(*Radio { ... });
```

- expose constructivism_macro as macro-library, so it can be injected into your own namespace

- doc-based bindgen for third-parti libraries (not right now)

- nested construct inference (looks like possible):
```rust
#[derive(Construct, Default)]
pub struct Vec2 {
    x: f32,
    y: f32
}
#[derive(Construct)]
pub struct Div {
    position: Vec2,
    size: Vec2,
}
fn step_inference() {
    let div = construct!(Div {
        position: {{ x: 23., y: 20. }},
        size: {{ x: 23., y: 20. }}
    })
}
```

### Limitations:
- only public structs (or enums with `constructable!`)
- no generics supported yet (looks very possible)
- limited number of params for the whole inheritance tree (current version compiles with 16, tested with 64)
- only static structs/enums (no lifetimes)

### Cost:
I didn't perform any stress-tests. It should run pretty fast: there is no heap allocations, only some deref calls per `construct!` per defined prop per depth level. Cold compilation time grows with number of params limit (1.5 mins for 64), but the size of the binary doesn't changes.

> TODO: Provide stress testing and results

### License

The `constructivism_macro` is dual-licensed under either:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

This means you can select the license you prefer!
This dual-licensing approach is the de-facto standard in the Rust ecosystem and there are [very good reasons](https://github.com/bevyengine/bevy/issues/2373) to include both.