use bevy::{math::DVec2, prelude::*};
use bevy_xpbd_2d::prelude::*;

use crate::helper;
pub struct MapPlugin;

impl Plugin for MapPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Startup, spawn_map)
			.add_systems(FixedUpdate, apply_gravity);
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
pub struct AffectedByGravity;

fn apply_gravity(
	time_step: Res<Time<Fixed>>,
	mut gravity_affected_query: Query<(&AffectedByGravity, &Position, &Mass, &mut ExternalForce)>,
	planet_query: Query<(&Planet, &Position, &Mass)>,
) {
	for (_, player_position, player_mass, mut external_force) in gravity_affected_query.iter_mut() {
		for (_, planet_position, planet_mass) in planet_query.iter() {
			let grav_direction = planet_position.0 - player_position.0;

			let force = time_step.timestep().as_secs_f64()
				* 4000.0 * ((player_mass.0 * planet_mass.0)
				/ (grav_direction).length_squared());

			let direction_norm = grav_direction.normalize();
			let force_vec = direction_norm * force;

			external_force.apply_force(force_vec);
		}
	}
}

#[derive(Component)]
pub struct Planet;

fn spawn_map(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<ColorMaterial>>,
) {
	let world_size = 3e4;
	let planet_density: f64 = 3.65 * 1e-4;
	let gened_world = generate_world(
		(world_size * -0.5, world_size * -0.5),
		world_size,
		(world_size.powf(2.0) * (planet_density).powf(2.0)) as i32,
	);

	dbg!(helper::count_objects(
		gened_world.iter().map(|x| x.planet_type).collect()
	));

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

/////////////////////////
// MAP GENERATION CODE //
/////////////////////////

use bevy::render::color::Color;
use rand::{thread_rng, Rng, SeedableRng};
use std::f64::consts::PI;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum PlanetTypesGen {
	Terestrial,
	GasGiant,
	BlackHole,
	Alien,
}

impl PlanetTypesGen {
	fn all() -> Vec<Self> {
		vec![
			PlanetTypesGen::Terestrial,
			PlanetTypesGen::GasGiant,
			PlanetTypesGen::BlackHole,
			PlanetTypesGen::Alien,
		]
	}

	fn get_frequency(&self) -> f64 {
		match &self {
			PlanetTypesGen::BlackHole => 0.7,
			_ => 1.0,
		}
	}

	fn get_radius(&self) -> f64 {
		match &self {
			PlanetTypesGen::Alien => 450.0,
			PlanetTypesGen::Terestrial => 500.0,
			PlanetTypesGen::GasGiant => 700.0,
			PlanetTypesGen::BlackHole => 90.0,
		}
	}

	fn get_padded_radius(&self) -> f64 {
		match &self {
			PlanetTypesGen::Alien => 0.0,
			PlanetTypesGen::Terestrial => 0.0,
			PlanetTypesGen::GasGiant => 50.0,
			PlanetTypesGen::BlackHole => 250.0,
		}
	}

	fn get_density(&self) -> f64 {
		return match &self {
			PlanetTypesGen::Alien => 475.0,
			PlanetTypesGen::Terestrial => 450.0,
			PlanetTypesGen::GasGiant => 300.0,
			PlanetTypesGen::BlackHole => 10000.0,
		};
	}

	pub fn get_color(&self) -> Color {
		match &self {
			PlanetTypesGen::Terestrial => Color::GREEN,
			PlanetTypesGen::GasGiant => Color::ORANGE,
			PlanetTypesGen::BlackHole => Color::BLACK,
			PlanetTypesGen::Alien => Color::GRAY,
		}
	}
}

#[derive(Debug)]
pub struct PlanetGen {
	pub x: f64,
	pub y: f64,
	pub radius: f64,
	pub mass: f64,
	pub planet_type: PlanetTypesGen,
}

const MINIMUM_PLANET_DISTANCE: f64 = 640.0;

fn check_planet_valid(size: f64, planet_list: &Vec<PlanetGen>, canidate: &PlanetGen) -> bool {
	if canidate.x - canidate.radius > size || canidate.x - canidate.radius < 0.0 {
		return false;
	}
	if canidate.y - canidate.radius > size || canidate.y - canidate.radius < 0.0 {
		return false;
	}

	for planet in planet_list {
		if planet.radius
			+ planet.planet_type.get_padded_radius()
			+ canidate.radius
			+ canidate.planet_type.get_padded_radius()
			+ MINIMUM_PLANET_DISTANCE
			> ((planet.x - canidate.x).powf(2.0) + (planet.y - canidate.y).powf(2.0)).sqrt()
		{
			return false;
		}
	}

	true
}


use rand_chacha;

pub fn generate_world(offset: (f64, f64), size: f64, planet_count: i32) -> Vec<PlanetGen> {
	assert!(size > 0.0);
	let mut rng = rand_chacha::ChaChaRng::seed_from_u64(u64::from_be_bytes([b'Z', b'a', b'c', b'k', b'C', b'o', b'o', b'l']));

	let mut planets: Vec<PlanetGen> = vec![];
	let mut planet_frequencies: Vec<(PlanetTypesGen, f64)> = vec![];

	for planet_type in PlanetTypesGen::all() {
		planet_frequencies.push((planet_type, planet_type.get_frequency()))
	}

	for _ in 0..planet_count {
		let canidate_planet_type = helper::weighted_random(&mut rng, planet_frequencies.clone());
		loop {
			let canidate_location = (rng.gen_range(0.0..size), rng.gen_range(0.0..size));

			let canidate_planet_volume = canidate_planet_type.get_radius().powf(2.0) * PI;
			let canidate_radius = (helper::normal_dist(
				&mut rng,
				canidate_planet_volume,
				canidate_planet_volume * 0.2,
				1.75,
			) / PI)
				.sqrt();

			let canidate_density = helper::normal_dist(
				&mut rng,
				canidate_planet_type.get_density(),
				canidate_planet_type.get_density() * 0.2,
				1.75,
			);

			let canidate_planet = PlanetGen {
				x: canidate_location.0,
				y: canidate_location.1,
				radius: canidate_radius,
				// If you're thinking "Hey thats not how you calculate mass!", yep.
				mass: canidate_density * canidate_radius.powf(1.4),
				planet_type: canidate_planet_type,
			};

			if check_planet_valid(size, &planets, &canidate_planet) {
				planets.push(canidate_planet);
				break;
			} else {
				continue;
			}
		}
	}

	for mut planet in &mut planets {
		planet.x += offset.0;
		planet.y += offset.1;
	}

	planets
}
