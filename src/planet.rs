use bevy::prelude::*;
use bevy_xpbd_2d::prelude::*;

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
    let r = 1000.;
    let d = 10000.;
    commands.spawn(PlanetBundle {
        planet: Planet,
        mesh: ColorMesh2dBundle {
            mesh: meshes
                .add(shape::Circle::new(r).into())
                .into(),
            material: materials.add(ColorMaterial::from(Color::WHITE)),
            ..default()
        },
        rigid_body: RigidBody::Static,
        collider: Collider::ball(r.into()),
        position: Position(Vec2::new(0., 0.).into()),
        mass: Mass((r*d).into()),
        friction: Friction::new(0.9)
    });

    let r = 100.;
    let d = 100000.;
    commands.spawn(PlanetBundle {
        planet: Planet,
        mesh: ColorMesh2dBundle {
            mesh: meshes
                .add(shape::Circle::new(r).into())
                .into(),
            material: materials.add(ColorMaterial::from(Color::BLACK)),
            ..default()
        },
        rigid_body: RigidBody::Static,
        collider: Collider::ball(r.into()),
        position: Position(Vec2::new(1000., 3000.).into()),
        mass: Mass((r*d).into()),
        friction: Friction::new(0.9)
    });

    let r = 1000.;
    let d = 1000.;
    commands.spawn(PlanetBundle {
        planet: Planet,
        mesh: ColorMesh2dBundle {
            mesh: meshes
                .add(shape::Circle::new(r).into())
                .into(),
            material: materials.add(ColorMaterial::from(Color::RED)),
            ..default()
        },
        rigid_body: RigidBody::Static,
        collider: Collider::ball(r.into()),
        position: Position(Vec2::new(-1000., 4000.).into()),
        mass: Mass((r*d).into()),
        friction: Friction::new(0.9)
    });
}
