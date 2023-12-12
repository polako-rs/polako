use bevy::render::{Render, RenderSet};
use bevy::utils::HashSet;
use bevy::utils::synccell::SyncCell;
use vello::kurbo::{Affine, Rect, Stroke};
use vello::peniko::{Color, Fill};
use vello::{Renderer, RendererOptions, Scene, SceneBuilder, SceneFragment};



use bevy::{
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_asset::RenderAssets,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        renderer::{RenderDevice, RenderQueue},
        RenderApp,
    },
};

#[derive(Component)]
pub struct Tst(usize);

fn t(mut q: Query<Option<&mut Tst>>) {
    for i in q.iter_mut() {

    }
}

#[derive(Resource)]
struct VelloRenderer(SyncCell<Renderer>);

impl FromWorld for VelloRenderer {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>();
        let queue = world.resource::<RenderQueue>();

        VelloRenderer(SyncCell::new(
            Renderer::new(
                device.wgpu_device(),
                RendererOptions {
                    surface_format: None,
                    timestamp_period: queue.0.get_timestamp_period(),
                    antialiasing_support: vello::AaSupport::all(),
                    use_cpu: false,
                },
            )
            .unwrap(),
        ))
    }
}

struct VelloPlugin;

impl Plugin for VelloPlugin {
    fn build(&self, app: &mut App) {
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        // This should probably use the render graph, but working out the dependencies there is awkward
        render_app.add_systems(Render, render_scenes.in_set(RenderSet::Render));
    }

    fn finish(&self, app: &mut App) {
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app.init_resource::<VelloRenderer>();
    }
}

fn render_scenes(
    mut renderer: ResMut<VelloRenderer>,
    mut scenes: Query<&VelloScene>,
    gpu_images: Res<RenderAssets<Image>>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    for scene in &mut scenes {
        let gpu_image = gpu_images.get(&scene.1).unwrap();
        let params = vello::RenderParams {
            base_color: vello::peniko::Color::TRANSPARENT,
            width: gpu_image.size.x as u32,
            height: gpu_image.size.y as u32,
            antialiasing_method: vello::AaConfig::Msaa8,
        };
        renderer
            .0
            .get()
            .render_to_texture(
                device.wgpu_device(),
                &queue,
                &scene.0,
                &gpu_image.texture_view,
                &params,
            )
            .unwrap();
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(VelloPlugin)
        .add_systems(Startup, setup)
        .add_plugins(ExtractComponentPlugin::<VelloScene>::default())
        .add_systems(Update, render_fragment)
        .run()


}

#[derive(Component)]
pub struct VelloTarget(Handle<Image>);

#[derive(Component)]
// In the future, this will probably connect to the bevy heirarchy with an Affine component
pub struct VelloFragment(SceneFragment);

#[derive(Component)]
struct VelloScene(Scene, Handle<Image>);

impl ExtractComponent for VelloScene {
    type Query = (&'static VelloFragment, &'static VelloTarget);

    type Filter = ();

    type Out = Self;

    fn extract_component(
        (fragment, target): bevy::ecs::query::QueryItem<'_, Self::Query>,
    ) -> Option<Self> {
        let mut scene = Scene::default();
        let mut builder = SceneBuilder::for_scene(&mut scene);
        builder.append(&fragment.0, None);
        Some(Self(scene, target.0.clone()))
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.spawn(Camera2dBundle::default());
    let size = Extent3d {
        width: 512,
        height: 512,
        ..default()
    };

    // This is the texture that will be rendered to.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        },
        ..default()
    };

    // fill image.data with zeroes
    image.resize(size);

    let x = 1;
    let b = Box::new(&x);
    take_box(b);
    
    let image_handle = images.add(image);
    commands.spawn(SpriteBundle {
        texture: image_handle.clone(),
        transform: Transform::from_scale(Vec3::splat(0.5)),
        ..default()
    });
    commands.spawn((
        VelloFragment(SceneFragment::default()),
        VelloTarget(image_handle),
    ));
}

fn take_box(b: Box<&usize>) {

}
fn closure<'a, I: Iterator<Item = &'a str>, F: Fn(&'a str, usize, usize) -> I>(func: F) -> F {
    func
}
fn get_card_score(line: &str) -> u32 {
    // fn read_numbers(from: &str, starting: &usize, count: &usize) -> HashSet<&str> {
    //     return HashSet::from_iter(
    //         (0..count).map(|i| &from[starting + i * 3..starting + (i + 1) * 3]),
    //     );
    // }

    let read_numbers = closure(|from, starting, count| {
        (0..count).map(move |i| &from[starting + i * 3..starting + (i + 1) * 3])
    });

    let x = read_numbers("".into(), 1, 2);

    // let numbers = ;
    let numbers: HashSet<&str> = HashSet::from_iter(read_numbers(line, 10, 10));
    let answers: HashSet<&str> = HashSet::from_iter(read_numbers(line, 42, 25));

    return 0;
}

fn render_fragment(mut fragment: Query<&mut VelloFragment>) {
    let mut fragment = fragment.single_mut();
    let mut sb = SceneBuilder::for_fragment(&mut fragment.0);
    let dark = Color::rgb(0.2, 0.2, 0.2);
    let light = Color::rgb(0.8, 0.8, 0.8);
    // let linear =  Gradient::new_linear((0.0, 0.0), (0.0, 200.0)).with_stops([
    //     Color::RED,
    //     Color::GREEN,
    //     Color::BLUE,
    // ]);
    let x = |a, b| {
        (a..b).map(|x| x + 1)
    };
    let c = x(0, 2);
    sb.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &light,
        None,
        &Rect::new(128., 128., 512. - 128., 512. - 128.).to_rounded_rect(32.),
    );
    sb.stroke(
        &Stroke::new(8.),
        Affine::IDENTITY,
        &dark,
        None,
        // &Rect::new(128., 128., 512. - 128., 512. - 128.),
        &Rect::new(128., 128., 512. - 128., 512. - 128.).to_rounded_rect(32.),
    );
}
