use bevy::{math::DVec2, prelude::*};
use bevy_xpbd_2d::prelude::*;

use crate::generate_map;
use crate::helper;
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
	let world_size = 2e5;
	let planet_density: f64 = 3.7 * 1e-4;
    let gened_world = generate_map::generate_world(
		(-1e5, -1e5),
		world_size,
		(world_size.powf(2.0) * (planet_density).powf(2.0)) as i32
	);

    dbg!(helper::count_objects(gened_world.iter().map(|x| x.planet_type).collect()));

	for planet in gened_world {
		commands.spawn(PlanetBundle {
			planet: Planet,
			mesh: ColorMesh2dBundle {
				mesh: meshes
					.add(shape::Circle::new(planet.radius as f32).into())
					.into(),
				material: materials.add(ColorMaterial::from(planet.planet_type.get_color())),
				..default()
			},
			rigid_body: RigidBody::Static,
			collider: Collider::ball((planet.radius as f32).into()),
			position: Position(DVec2::new(planet.x, planet.y).into()),
			mass: Mass((planet.mass).into()),
			friction: Friction::new(0.9),
		});
	}

    
}
