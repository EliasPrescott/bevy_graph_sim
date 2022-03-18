use bevy::prelude::*;

pub struct SpawnSpheres {
    pub sphere_x_count: u32,
    pub sphere_y_count: u32,
    pub sphere_z_count: u32,
}

#[derive(Component)]
pub struct Sphere(pub u32, pub u32, pub u32);

#[derive(Component)]
pub struct OriginalPosition(pub Transform);

struct SphereCount(pub u32, pub u32, pub u32);

impl Plugin for SpawnSpheres {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(SphereCount(self.sphere_x_count, self.sphere_y_count, self.sphere_z_count))
            .add_startup_system(add_spheres);
    }
}

fn add_spheres(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    sphere_count: Res<SphereCount>,
) {
    let sphere_mesh = meshes.add(Mesh::from(shape::Icosphere {
        subdivisions: 6,
        radius: 5.,
        ..Default::default()
    }));
    let sphere_material = materials.add(StandardMaterial {
        base_color: Color::YELLOW,
        ..Default::default()
    });
    for x in 0..sphere_count.0 {
        for y in 0..sphere_count.1 {
            for z in 0..sphere_count.2 {
                let pos = Transform::from_xyz(
                    ((x * 12) as i64 - ((12 * sphere_count.0 as i64) / 2)) as f32,
                    ((y * 12) as i64 - ((12 * sphere_count.1 as i64) / 2)) as f32,
                    ((z * 12) as i64 - ((12 * sphere_count.2 as i64) / 2)) as f32);
                commands
                    .spawn_bundle(PbrBundle {
                        mesh: sphere_mesh.clone(),
                        material: sphere_material.clone(),
                        transform: pos,
                        ..Default::default()
                    })
                    .insert(Sphere(x, 0, z))
                    .insert(OriginalPosition(pos));
            }
        }
    }
}
