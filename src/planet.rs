use bevy::prelude::*;
use bevy_xpbd_2d::prelude::*;

const PLANET_RADIUS: f64 = 1000.;

pub struct PlanetPlugin;

impl Plugin for PlanetPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_planet);
    }
}

#[derive(Bundle)]
struct PlanetBundle {
    planet: Planet,
    mesh: ColorMesh2dBundle,
    rigid_body: RigidBody,
    collider: Collider,
    position: Position,
    mass: Mass,
    friction: Friction,
}

#[derive(Component)]
pub struct Planet;

fn setup_planet(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(PlanetBundle {
        planet: Planet,
        mesh: ColorMesh2dBundle {
            mesh: meshes
                .add(shape::Circle::new(PLANET_RADIUS as f32).into())
                .into(),
            material: materials.add(ColorMaterial::from(Color::WHITE)),
            ..default()
        },
        rigid_body: RigidBody::Static,
        collider: Collider::ball(PLANET_RADIUS),
        position: Position(Vec2::new(0., 0.).into()),
        mass: Mass(1e5),
        friction: Friction::new(0.9)
    });

    // commands.spawn(PlanetBundle {
    //     planet: Planet,
    //     mesh: ColorMesh2dBundle {
    //         mesh: meshes
    //             .add(shape::Circle::new(PLANET_RADIUS as f32).into())
    //             .into(),
    //         material: materials.add(ColorMaterial::from(Color::WHITE)),
    //         ..default()
    //     },
    //     rigid_body: RigidBody::Static,
    //     collider: Collider::ball(PLANET_RADIUS),
    //     position: Position(Vec2::new(0., 0.).into()),
    //     mass: Mass(100000.),
    // });
}
