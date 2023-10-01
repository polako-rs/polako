use bevy::prelude::*;
use polako_flow::*;
use polako_constructivism::*;

#[derive(Construct)]
#[construct(Label -> Text -> Nothing)]
pub struct Label;

#[derive(Component)]
struct Source;


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FlowPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, update_source_text)
        .run();
}

fn setup(
    mut commands: Commands
) {
    commands.spawn(Camera2dBundle::default());
    let mut source_text = None;
    let mut target_text = None;
    commands.spawn(NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Column,
            ..default()
        },
        ..default()
    }).with_children(|b| {
        source_text = Some(b.spawn(TextBundle::default()).insert(Source).id());
        target_text = Some(b.spawn(TextBundle::default()).id());

    });
    let source_text = source_text.unwrap();
    let target_text = target_text.unwrap();
    commands.add(BindComponentToComponent {
        from: source_text.get(prop!(Label.text)),
        to: target_text.set(prop!(Label.text)),
    });
}

fn update_source_text(
    time: Res<Time>,
    mut source: Query<&mut Text, With<Source>>
) {
    for mut text in source.iter_mut() {
        if text.sections.is_empty() {
            text.sections.push(TextSection::new(format!(""), TextStyle::default()));
        }
        text.sections[0].value = format!("{}", time.elapsed_seconds());
    }
}