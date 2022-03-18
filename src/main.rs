use bevy::{
    prelude::*,
    window::{WindowMode, WindowResizeConstraints},
};

mod pan_camera;
mod move_spheres;
mod spawn_spheres;
mod parsing;
mod parsing_function;

use pan_camera::AddOrbitCamera;
use move_spheres::MoveSpheres;
use spawn_spheres::SpawnSpheres;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgba(0.0, 0.0, 0.0, 1.0)))
        .insert_resource(WindowDescriptor {
            transparent: false,
            decorations: true,
            mode: WindowMode::Windowed,
            title: "Graph Sim".to_string(),
            width: 1200.,
            height: 800.,
            resize_constraints: WindowResizeConstraints {
                min_height: 400.0,
                min_width: 400.0,
                ..Default::default()
            },
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(AddOrbitCamera)
        .add_plugin(SpawnSpheres { 
            sphere_x_count: 25,
            sphere_y_count: 1,
            sphere_z_count: 25,
         })
        .add_plugin(MoveSpheres)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    // commands.spawn_bundle(PerspectiveCameraBundle {
    //     transform: Transform::from_xyz(0., 0., 500.).looking_at(Vec3::ZERO, Vec3::Y),
    //     ..Default::default()
    // });

    commands.spawn_bundle(DirectionalLightBundle {
        ..Default::default()
    });
}
