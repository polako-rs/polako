New eml dialect:

```rust
eml! {
    Label {
        class[visible, red],
        class[disabled] = {{ btn.disabled }}
        style.margin: [1., 23.],
        bind.value: {{ coolotbox.color }}
    }
}
```