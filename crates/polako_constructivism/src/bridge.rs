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

pub trait ColorProps {
    fn r(&self) -> f32;
    fn set_r(&mut self, r: f32);
    fn g(&self) -> f32;
    fn set_g(&mut self, g: f32);
    fn b(&self) -> f32;
    fn set_b(&mut self, b: f32);
    fn a(&self) -> f32;
    fn set_a(&mut self, a: f32);
    fn get_hex(&self) -> String;
    fn set_hex(&mut self, hex: impl AsRef<str>);
}
impl ColorProps for Color {
    fn r(&self) -> f32 {
        Srgba::from(*self).red
    }
    fn set_r(&mut self, r: f32) {
        let mut srgba = Srgba::from(*self);
        srgba.red = r;
        *self = Color::Srgba(srgba);
    }
    fn g(&self) -> f32 {
        Srgba::from(*self).green
    }
    fn set_g(&mut self, g: f32) {
        let mut srgba = Srgba::from(*self);
        srgba.green = g;
        *self = Color::Srgba(srgba);
    }
    fn b(&self) -> f32 {
        Srgba::from(*self).blue
    }
    fn set_b(&mut self, b: f32) {
        let mut srgba = Srgba::from(*self);
        srgba.blue = b;
        *self = Color::Srgba(srgba);
    }
    fn a(&self) -> f32 {
        Srgba::from(*self).alpha
    }
    fn set_a(&mut self, a: f32) {
        let mut srgba = Srgba::from(*self);
        srgba.alpha = a;
        *self = Color::Srgba(srgba);
    }
    fn get_hex(&self) -> String {
        Srgba::from(*self).to_hex()
    }
    fn set_hex(&mut self, hex: impl AsRef<str>) {
        let hex = hex.as_ref();
        *self = Color::Srgba(Srgba::hex(hex).unwrap_or_else(|_| {
            info!("Cant parse '{hex}` as color, using WHITE.");
            Srgba::WHITE
        }))
    }
}
derive_construct! {
    seq => Color -> Nothing;
    construct => (hex: String = format!("")) -> {
        Color::Srgba(Srgba::hex(hex).unwrap_or_default())
    };
    props => {
        /// Red channel
        r: f32 = [r, set_r];
        /// Green channel Color
        g: f32 = [g, set_g];
        /// Blue channel of Color
        b: f32 = [b, set_b];
        /// Alpha channel of Color
        a: f32 = [a, set_a];
        /// Hex representation Color
        hex: String = [get_hex, set_hex];
    };
}

pub trait ReadOnly {
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
        /// How much time has advanced since startup, as f32 seconds.
        elapsed: f32 = [elapsed_seconds, readonly];
        /// How much time has advanced since the last update, as f32 seconds.
        delta: f32 = [delta_seconds, readonly];
    };
}
