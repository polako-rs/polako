use bevy::prelude::*;
use polako_constructivism::*;
use polako_flow::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FlowPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    let text = commands.spawn(TextBundle::default()).id();
    commands.add(BindResourceToComponent {
        from: prop!(Time.elapsed).map(|s| format!("{s}")),
        to: text.set(prop!(Text.text)),
    })
}
