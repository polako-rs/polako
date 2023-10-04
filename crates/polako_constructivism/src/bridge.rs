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

trait TextProps {
    fn text(&self) -> String;
    fn set_text(&mut self, text: impl Into<String>);
}
impl TextProps for Text {
    fn text(&self) -> String {
        if let Some(section) = self.sections.first() {
            section.value.clone()
        } else {
            format!("")
        }
    }
    fn set_text(&mut self, text: impl Into<String>) {
        if self.sections.is_empty() {
            self.sections
                .push(TextSection::new(text, TextStyle::default()))
        } else {
            self.sections[0].value = text.into()
        }
    }
}
derive_segment! {
    seg => Text;
    construct => (text: String = format!("")) -> {
        Text::from_section(text, TextStyle::default())
    };
    props => {
        text: String = [text, set_text];
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

trait ReadOnly {
    fn readonly<T>(&mut self, _: T) {
        warn!("Attempt to set readonly prop");
    }
}

impl ReadOnly for Time {}

derive_construct! {
    seq => Time -> Nothing;
    construct => () -> {
        Time::default()
    };
    props => {
        elapsed_seconds: f32 = [elapsed_seconds, readonly];
        delta_seconds: f32 = [delta_seconds, readonly];
    };
}
