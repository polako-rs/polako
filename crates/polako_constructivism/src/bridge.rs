use crate::*;
use bevy::prelude::*;

derive_construct! {
    seq => NodeBundle -> Nothing;
    construct => () -> {
        NodeBundle::default() 
    };
}
derive_construct! { 
    seq => TextBundle -> Nothing;
    construct => () -> {
        TextBundle::default()
    };
}

trait NameProps {
    fn get_value(&self) -> String;
    fn set_value(&mut self, value: String);
}
impl NameProps for Name {
    fn get_value(&self) -> String {
        self.as_str().to_string()
    }
    fn set_value(&mut self, value: String) {
        self.set(value);
    }
}
derive_construct! {
    seq => Name -> Nothing;
    construct => (value: String) -> {
        Name::new(value)
    };
    props => {
        value: String = [get_value, set_value];
    };
}


derive_construct! {
    seq => Color -> Nothing;
    construct => (hex: String = format!("")) -> {
        Color::hex(hex).unwrap_or_default()
    };
    props => {
        /// Red channel
        r: f32 = [r, set_r];
        /// Green channel
        g: f32 = [g, set_g];
        /// Blue channel
        b: f32 = [b, set_b];
        /// Alpha channel
        a: f32 = [a, set_a];
    };
}
