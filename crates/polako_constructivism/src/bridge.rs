use crate::*;
use bevy::prelude::*;

constructable! { NodeBundle() NodeBundle::default() }
constructable! { TextBundle() TextBundle::default() }
constructable! { Name (value: String) {
    Name::new(value)
}}