#![doc(html_logo_url = "https://github.com/jkb0o/polako/raw/master/docs/polakoko.png")]

pub use bevy;
pub mod constructivism {
    pub use polako_constructivism::*;
    pub use polako_macro::*;
    pub mod prelude {
        pub use polako_constructivism::{design, Construct, ConstructItem, Segment};
        pub use polako_macro::*;
    }
}

pub mod eml {

    pub use polako_eml::*;
    pub use polako_macro::*;
}

pub mod flow {
    pub use polako_flow::*;
}
