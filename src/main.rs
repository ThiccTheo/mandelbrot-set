use bevy::{prelude::*, window::WindowResolution};

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;
const ITERATIONS: usize = 50;
const IN_SET: Color = Color::BLACK;
const OUT_SET: Color = Color::ORANGE;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(WIDTH as f32, HEIGHT as f32),
                title: String::from("Mandelbrot Set"),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(OUT_SET))
        .add_systems(Startup, spawn_camera)
        .add_systems(Update, color_pixels_in_viewport)
        .run();
}

fn spawn_camera(mut cmds: Commands) {
    let mut cam = Camera2dBundle::default();
    cam.projection.scale /= 250.;
    cmds.spawn(cam);
}

fn color_pixels_in_viewport(cam_qry: Query<(&Camera, &GlobalTransform)>, mut gizmos: Gizmos) {
    let (cam, cam_glob_xform) = cam_qry.single();

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let c = cam
                .viewport_to_world_2d(cam_glob_xform, Vec2::new(x as f32, y as f32))
                .unwrap();
            let mut z = Vec2::ZERO;
            let mut successful_iterations = 0;

            for _ in 0..ITERATIONS {
                z = f(z, c);

                if (z.x + z.y).abs() > 2. {
                    break;
                }
                successful_iterations += 1;
            }
            let rate = successful_iterations as f32 / ITERATIONS as f32;

            gizmos.rect_2d(
                Vec2::new(c.x + 1., c.y - 0.5),
                0.,
                Vec2::ONE,
                Color::rgb(
                    f32::lerp(OUT_SET.r(), IN_SET.r(), rate),
                    f32::lerp(OUT_SET.g(), IN_SET.g(), rate),
                    f32::lerp(OUT_SET.b(), IN_SET.b(), rate),
                ),
            );
        }
    }
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
