#![doc(
    html_logo_url = "https://github.com/jkb0o/polako/raw/master/docs/polakoko.png"
)]


pub mod constructivism {
    pub mod prelude {
        pub use polako_macro::*;
        pub use polako_constructivism::{
            Construct,protocols, construct
        };
    }
    pub use polako_constructivism::*;
}

pub mod eml {
    
    pub use polako_macro::*;
    pub use polako_eml::*;
}