use {
    bevy::{
        diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
        input::mouse::MouseWheel,
        prelude::*,
        render::{
            render_asset::RenderAssetUsages,
            render_resource::{Extent3d, TextureDimension, TextureFormat},
        },
        window::{WindowMode, WindowResolution},
    },
    rayon::prelude::*,
};

const WIDTH: usize = 200;
const HEIGHT: usize = 200;
const ITERATIONS: usize = 50;
const IN_SET: Color = Color::BLACK;
const OUT_SET: Color = Color::ORANGE;
const DEFAULT_ZOOM: f32 = 200.;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(WIDTH as f32, HEIGHT as f32),
                    title: String::from("Mandelbrot Set"),
                    mode: WindowMode::Windowed,
                    ..default()
                }),
                ..default()
            }),
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
        ))
        .insert_resource(ClearColor(OUT_SET))
        .add_systems(Startup, (spawn_cameras, spawn_visual))
        .add_systems(
            Update,
            (adjust_view_of_visual, color_pixels_in_viewport).chain(),
        )
        .run();
}

#[derive(Component)]
pub struct VisualCamera;

#[derive(Component)]
pub struct CalculationsCamera;

fn spawn_cameras(mut cmds: Commands) {
    cmds.spawn((VisualCamera, Camera2dBundle::default()));

    let mut calc_cam = Camera2dBundle::default();
    calc_cam.projection.scale /= DEFAULT_ZOOM;
    calc_cam.camera.is_active = false;
    cmds.spawn((CalculationsCamera, calc_cam));
}

fn adjust_view_of_visual(
    mut cam_qry: Query<(&mut Transform, &mut OrthographicProjection), With<CalculationsCamera>>,
    mut scroll_wheel_evr: EventReader<MouseWheel>,
    kb: Res<ButtonInput<KeyCode>>,
) {
    let (mut cam_xform, mut cam_projection) = cam_qry.single_mut();

    if let Some(scroll_event) = scroll_wheel_evr.read().nth(0) {
        if scroll_event.y > 0. {
            cam_projection.scale /= 2.;
        } else if scroll_event.y < 0. {
            cam_projection.scale *= 2.;
        }
    }
    let xlation_amt = cam_projection.scale * DEFAULT_ZOOM;

    if kb.just_pressed(KeyCode::ArrowUp) {
        cam_xform.translation.y += xlation_amt;
    }
    if kb.just_pressed(KeyCode::ArrowDown) {
        cam_xform.translation.y -= xlation_amt;
    }
    if kb.just_pressed(KeyCode::ArrowLeft) {
        cam_xform.translation.x -= xlation_amt;
    }
    if kb.just_pressed(KeyCode::ArrowRight) {
        cam_xform.translation.x += xlation_amt;
    }
}

#[derive(Resource, Deref, DerefMut)]
struct Visual(Handle<Image>);

fn spawn_visual(mut cmds: Commands, mut imgs: ResMut<Assets<Image>>) {
    let img = Image::new_fill(
        Extent3d {
            width: WIDTH as u32,
            height: HEIGHT as u32,
            ..default()
        },
        TextureDimension::D2,
        &OUT_SET.as_rgba_u8(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    let img_handle = imgs.add(img);
    cmds.insert_resource(Visual(img_handle.clone()));

    cmds.spawn(SpriteBundle {
        texture: img_handle.clone_weak(),
        ..default()
    });
}

/*
f : C  -> C
    z |-> z^2 + c
*/
fn f(z: Vec2, c: Vec2) -> Vec2 {
    /*
    z^2 = (a + bi)^2
        = (a + bi)(a + bi)
        = a^2 + 2abi - b^2
        = (a^2 - b^2) + 2abi <=> <x^2 - y^2, 2xy>
    */
    Vec2::new(z.x * z.x - z.y * z.y, 2. * z.x * z.y) + c
}

fn color_pixels_in_viewport(
    cam_qry: Query<(&Camera, &GlobalTransform), With<CalculationsCamera>>,
    mut imgs: ResMut<Assets<Image>>,
    visual: Res<Visual>,
) {
    let (cam, cam_glob_xform) = cam_qry.single();
    let Some(img) = imgs.get_mut(visual.0.clone_weak()) else {
        return;
    };
    img.data.clear();
    img.data.par_extend(
        (0..WIDTH * HEIGHT)
            .into_par_iter()
            .map(|i| (i % WIDTH, i / WIDTH))
            .map(|(x, y)| {
                let c = cam
                    .viewport_to_world_2d(cam_glob_xform, Vec2::new(x as f32, y as f32))
                    .unwrap();
                let mut z = Vec2::ZERO;
                let mut successful_iterations = 0;

                for _ in 1..ITERATIONS {
                    z = f(z, c);

                    if (z.x + z.y).abs() > 2. {
                        break;
                    }
                    successful_iterations += 1;
                }
                let rate = successful_iterations as f32 / ITERATIONS as f32;

                Color::rgb(
                    f32::lerp(OUT_SET.r(), IN_SET.r(), rate),
                    f32::lerp(OUT_SET.g(), IN_SET.g(), rate),
                    f32::lerp(OUT_SET.b(), IN_SET.b(), rate),
                )
                .as_rgba_u8()
            })
            .flatten(),
    );
}
