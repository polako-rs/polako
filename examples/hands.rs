use bevy::ecs::system::StaticSystemParam;
use bevy::prelude::*;
use polako::eml::*;
use polako::flow::*;

#[derive(Signal)]
pub struct Pressed {
    entity: Entity,
}

#[derive(Signal)]
pub struct TakeDamageSignal {
    entity: Entity,
    amount: f32,
    kind: u8,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Polako UI Sample".into(),
                resolution: (450., 400.).into(),
                position: WindowPosition::At(IVec2::new(300, 300)),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FlowPlugin)
        .add_plugins(PolakoInputPlugin)
        .add_systems(Startup, hello_world)
        .add_systems(Update, ui_text_system)
        .add_systems(Update, div_system)
        .run();
}

#[derive(Element)]
#[construct(Div -> Empty)]
#[signals(
    /// Emits when pointer enters the div's rect
    hover: HoverSignal,

    /// Emits when div become focused
    focus: FocusSignal,

    /// Emits when there is pointer motion on the top of the div
    motion: MotionSignal,

    /// When damage taken
    take_damage: TakeDamageSignal,
)]
pub struct Div {
    #[prop(construct)]
    /// The background color of element
    bg: Color,
}

#[derive(Behavior)]
pub struct UiText {
    /// The text value of UiText element.
    pub text: String,
    #[param(default = Color::hex("2f2f2f").unwrap())]
    pub text_color: Color,
}

#[derive(Element)]
#[construct(Label -> UiText -> Div)]
pub struct Label;
impl ElementBuilder for Label {
    fn build_element(_: Vec<Entity>) -> Blueprint<Self> {
        blueprint! {
            Label::Base + TextBundle
        }
    }
}

#[derive(Default)]
pub struct Dummy;

fn hello_world(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.add(eml! {
        resource(time, Time);
        Body [
            Column {
                .on.enter: () => {
                    info("Column enters.");
                },
                .on.update: (e) => {
                    if time.elapsed > 1. && time.elapsed <= 2. {
                        info("1..2");
                        delta.take_damage.emit(.amount: time.elapsed);
                    } else if -time.elapsed > -(1.0) {
                        info("< 1");
                    }
                    delta.text = e.delta.fmt("Frame time: {:0.4}");
                    elapsed.text = time.elapsed.fmt("Elapsed time: {:0.2}");
                    elapsed.bg.g = (time.elapsed - 2.) * 0.5;
                    // info("Updated with delta = {:0.4}s", e.delta);
                },
                .on.motion: () => {
                    info("motioning");
                }
            } [
                delta: Label {
                    .text: "0.0000",
                    .on.take_damage: (e) => {
                        info("taking damage {}", e.amount);
                        // info("taking damage");
                    }
                },
                elapsed: Label { .text: "0.00" },
            ]
        ]
    });
}

// Expanded `.on.update: () => { ... }`
use polako_constructivism::*;
use polako_eml::EntityMark;
fn _expand(world: &mut World) {
    let mut __entity__ = world.spawn_empty();
    let entity = __entity__.id();
    let elapsed = EntityMark::<Label>::new(entity);
    let delta = EntityMark::<Label>::new(entity);
    let time = ResourceMark::<Time>::new();

    let __ext__ = <<Body as Construct>::Design as Singleton>::instance()
        .on()
        .update();
    __ext__.assign(
        &mut __entity__,
        move |e: &_, // infers as &UpdateSignal
              _params: &mut StaticSystemParam<(
            Commands,
            ParamSet<(
                Query<&mut _>, // infers as Query<&mut UiText>
                Res<_>,        // infers as Res<Time>
                Query<&mut _>, // infers as Query<&mut Div>
            )>,
        )>| {
            let (_commands, _params) = ::std::ops::DerefMut::deref_mut(_params);
            let _v_time_elapsed = {
                let _host = _params.p1();
                time.getters().elapsed(&_host).into_value().get()
            };
            let _v_delta_text = {
                let _inset = _params.p0();
                let _mut = _inset.get(delta.entity).unwrap();
                let _host = &_mut;
                delta.getters().text(&_host).into_value().get()
            };
            let _v_e_delta = {
                let _host = e;
                e.getters().delta(&_host).into_value().get()
            };
            let _v_elapsed_text = {
                let _inset = _params.p0();
                let _mut = _inset.get(elapsed.entity).unwrap();
                let _host = &_mut;
                elapsed.getters().text(&_host).into_value().get()
            };
            let _v_elapsed_bg_g = {
                let _inset = _params.p2();
                let _mut = _inset.get(elapsed.entity).unwrap();
                let _host = &_mut;
                elapsed.getters().bg(&_host).g().into_value().get()
            };
            {
                let _val = ({
                    let res = format!("Frame time: {:0.4}", _v_e_delta);
                    res
                })
                .into();
                if _v_delta_text != _val {
                    delta
                        .setters()
                        .set_text(_params.p0().get_mut(delta.entity).unwrap().as_mut(), _val);
                    delta
                        .descriptor()
                        .text()
                        .notify_changed(_commands, delta.entity);
                }
            };
            {
                let _val = ({
                    let res = format!("Elapsed time: {:0.2}", _v_time_elapsed);
                    res
                })
                .into();
                if _v_elapsed_text != _val {
                    elapsed
                        .setters()
                        .set_text(_params.p0().get_mut(elapsed.entity).unwrap().as_mut(), _val);
                    elapsed
                        .descriptor()
                        .text()
                        .notify_changed(_commands, elapsed.entity);
                }
            };
            {
                let _val = ((_v_time_elapsed - 2.) * 0.5).into();
                if _v_elapsed_bg_g != _val {
                    elapsed
                        .setters()
                        .bg(_params.p2().get_mut(elapsed.entity).unwrap().as_mut())
                        .set_g(_val);
                    elapsed
                        .descriptor()
                        .bg()
                        .notify_changed(_commands, elapsed.entity);
                }
            };
        },
    );
}

impl ElementBuilder for Div {
    fn build_element(content: Vec<Entity>) -> Blueprint<Self> {
        blueprint! {
            Div::Base
            + NodeBundle
            [[ content ]]
        }
    }
}

#[derive(Element)]
#[construct(Body -> Div)]
pub struct Body;
impl ElementBuilder for Body {
    fn build_element(content: Vec<Entity>) -> Blueprint<Self> {
        blueprint! {
            Body::Base
            + Style(
                .width: Val::Percent(100.),
                .height: Val::Percent(100.),
                .justify_content: JustifyContent::Center,
                .align_content: AlignContent::Center,
                .align_items: AlignItems::Center,
            )
            [[ content ]]
        }
    }
}

#[derive(Element)]
#[construct(Column -> Div)]
pub struct Column;
impl ElementBuilder for Column {
    fn build_element(content: Vec<Entity>) -> Blueprint<Self> {
        blueprint! {
            Column::Base
            + Style (
                .display: Display::Flex,
                .flex_direction: FlexDirection::Column,
                .align_items: AlignItems::Center,
                .align_content: AlignContent::Center,
                .row_gap: Val::Px(3.),
            )
            [[ content ]]
        }
    }
}

#[derive(Element)]
#[construct(Row -> Div)]
pub struct Row;
impl ElementBuilder for Row {
    fn build_element(content: Vec<Entity>) -> Blueprint<Self> {
        blueprint! {
            Row::Base
            + Style (
                .display: Display::Flex,
                .flex_direction: FlexDirection::Row,
                .align_items: AlignItems::Center,
                .align_content: AlignContent::Center,
                .column_gap: Val::Px(3.),
            )
            [[ content ]]
        }
    }
}

use bevy::ecs::world::EntityMut;
use polako_constructivism::Is;
use polako_constructivism::Singleton;
impl DivDesign {
    // Div can accept string literals as content
    pub fn push_text<'c, S: AsRef<str>>(
        &self,
        world: &mut World,
        content: &'c mut Vec<Entity>,
        text: S,
    ) -> Implemented {
        let entity = world.spawn(TextBundle::with_text(text)).id();
        content.push(entity);
        Implemented
    }
    // Only Div and elements based on Div can be content of the Div
    pub fn push_content<E: Element + Is<Div>>(
        &self,
        _: &mut World,
        content: &mut Vec<Entity>,
        model: EntityMark<E>,
    ) -> Implemented {
        content.push(model.entity);
        Implemented
    }
    /// Everything based on Div can access the styles using param extensions: `Row { .s.padding: 25 }`
    pub fn s(&self) -> &'static Styles {
        &Styles
    }
}

use polako_flow::input::FocusSignal;
use polako_flow::input::HoverSignal;
use polako_flow::input::MotionSignal;
use polako_flow::Signal;
use polako_input::PolakoInputPlugin;
pub struct Signals;
impl Signals {
    pub fn hover(&self) -> &'static <HoverSignal as Signal>::Descriptor {
        <<HoverSignal as Signal>::Descriptor as Singleton>::instance()
    }
}
pub struct Styles;
impl Styles {
    /// The amount of space between the edges of a node and its contents in pixels.
    pub fn padding<T: IntoRect>(&self) -> StyleProperty<T> {
        StyleProperty(|entity, padding| {
            let rect = padding.into_rect();
            if !entity.contains::<Style>() {
                entity.insert(Style::default());
            }
            entity.get_mut::<Style>().unwrap().padding = rect;
        })
    }
}
pub struct StyleProperty<T>(fn(&mut EntityMut, T));
impl<T> StyleProperty<T> {
    pub fn assign<'w>(&self, entity: &mut EntityMut<'w>, value: T) {
        (self.0)(entity, value)
    }
}

// helpers for spawning text bundle
impl Default for UiText {
    fn default() -> Self {
        UiText {
            text: "".into(),
            text_color: Color::hex("2f2f2f").unwrap(),
        }
    }
}
pub trait WithText {
    fn with_text<T: AsRef<str>>(text: T) -> Self;
}
impl WithText for TextBundle {
    fn with_text<T: AsRef<str>>(text: T) -> TextBundle {
        let mut text = TextBundle::from_section(text.as_ref(), Default::default());
        text.text.sections[0].style.font_size = 24.;
        text.text.sections[0].style.color = Color::hex("2f2f2f").unwrap();
        text
    }
}
impl WithText for Text {
    fn with_text<T: AsRef<str>>(text: T) -> Self {
        let mut text = Text::from_section(text.as_ref().to_string(), Default::default());
        text.sections[0].style.font_size = 24.;
        text.sections[0].style.color = Color::hex("2f2f2f").unwrap();
        text
    }
}

/// bypass Div.background to BackgroundColor.0 when changed
/// and Div.padding to Style.padding
fn div_system(mut colors: Query<(&Div, &mut BackgroundColor), Changed<Div>>) {
    colors.for_each_mut(|(div, mut bg)| {
        bg.0 = div.bg.into();
    });
}

/// bypass UiText text value & color to Text.sections[0] when changed
fn ui_text_system(mut texts: Query<(&UiText, &mut Text), Changed<UiText>>) {
    for (ui_text, mut text) in texts.iter_mut() {
        if text.sections.is_empty() {
            *text = Text::with_text("");
        }
        text.sections[0].value = ui_text.text.clone();
        text.sections[0].style.color = ui_text.text_color.into();
    }
}

pub trait IntoRect {
    fn into_rect(self) -> UiRect;
}

impl IntoRect for i32 {
    fn into_rect(self) -> UiRect {
        UiRect::all(Val::Px(self as f32))
    }
}
impl IntoRect for f32 {
    fn into_rect(self) -> UiRect {
        UiRect::all(Val::Px(self))
    }
}

impl IntoRect for [i32; 2] {
    fn into_rect(self) -> UiRect {
        UiRect {
            left: Val::Px(self[0] as f32),
            right: Val::Px(self[0] as f32),
            top: Val::Px(self[1] as f32),
            bottom: Val::Px(self[1] as f32),
        }
    }
}
