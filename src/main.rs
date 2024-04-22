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
const MAX_ITERATIONS: usize = 50;
const DEFAULT_ZOOM: f32 = 50.;
const PALETTE: [Color; 7] = [
    Color::RED,
    Color::ORANGE,
    Color::YELLOW,
    Color::GREEN,
    Color::BLUE,
    Color::PURPLE,
    Color::BLACK,
];

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(WIDTH as f32, HEIGHT as f32),
                    title: String::from("Mandelbrot Set"),
                    mode: WindowMode::Windowed,
                    position: WindowPosition::Centered(MonitorSelection::Primary),
                    ..default()
                }),
                ..default()
            }),
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
        ))
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
    let mut calc_cam = Camera2dBundle::default();
    calc_cam.projection.scale /= DEFAULT_ZOOM;
    calc_cam.camera.is_active = false;
    cmds.spawn((CalculationsCamera, calc_cam));
    cmds.spawn((VisualCamera, Camera2dBundle::default()));
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
        &PALETTE[0].as_rgba_u8(),
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
fn mandelbrot_function(z: Vec2, c: Vec2) -> Vec2 {
    /*
    z^2 = (a + bi)^2
        = (a + bi)(a + bi)
        = a^2 + 2abi - b^2
        = (a^2 - b^2) + 2abi <=> <x^2 - y^2, 2xy>
    */
    Vec2::new(z.x * z.x - z.y * z.y, 2. * z.x * z.y) + c
}

fn calculate_color(n: usize) -> Color {
    let idx_dec = (PALETTE.len() - 1) as f32 * n as f32 / MAX_ITERATIONS as f32;
    let (idx, frac) = (idx_dec.trunc() as usize, idx_dec.fract());
    if idx == PALETTE.len() - 1 {
        *PALETTE.last().unwrap()
    } else {
        Color::rgb(
            f32::lerp(PALETTE[idx].r(), PALETTE[idx + 1].r(), frac),
            f32::lerp(PALETTE[idx].g(), PALETTE[idx + 1].g(), frac),
            f32::lerp(PALETTE[idx].b(), PALETTE[idx + 1].b(), frac),
        )
    }
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
                let mut n = 0usize;

                for _ in 1..=MAX_ITERATIONS {
                    z = mandelbrot_function(z, c);

                    if z.length() > 2. {
                        break;
                    }
                    n += 1;
                }
                calculate_color(n).as_rgba_u8()
            })
            .flatten(),
    );
}
