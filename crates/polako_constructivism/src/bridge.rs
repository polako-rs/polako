use crate::*;
use bevy::prelude::*;

derive_construct! { NodeBundle -> Nothing () NodeBundle::default() }
derive_construct! { TextBundle -> Nothing () TextBundle::default() }
derive_construct! { Name -> Nothing (value: String) {
    Name::new(value)
}}
