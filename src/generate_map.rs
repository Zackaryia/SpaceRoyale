use std::f64::consts::PI;
use std::hash;

use bevy::render::color::Color;
use rand::distributions::{Distribution, Standard};
use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng, random};

use crate::helper; // 0.8.0

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum PlanetTypesGen {
	Terestrial,
	GasGiant,
	BlackHole,
	Alien,
}

impl PlanetTypesGen {
    fn all() -> Vec<Self> {
        vec![PlanetTypesGen::Terestrial, PlanetTypesGen::GasGiant, PlanetTypesGen::BlackHole, PlanetTypesGen::Alien]
    }

    fn get_frequency(&self) -> f64 {
        match &self {
            _ => 1.0
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
			PlanetTypesGen::GasGiant => 0.0,
			PlanetTypesGen::BlackHole => 300.0,
		}
	}

	fn get_density(&self) -> f64 {
		return match &self {
			PlanetTypesGen::Alien => 500.0,
			PlanetTypesGen::Terestrial => 500.0,
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

const MINIMUM_PLANET_DISTANCE: f64 = 650.0;

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

pub fn generate_world(offset: (f64, f64), size: f64, planet_count: i32) -> Vec<PlanetGen> {
	assert!(size > 0.0);
    let mut rng = thread_rng();

	let mut planets: Vec<PlanetGen> = vec![];
    let mut planet_frequencies: Vec<(PlanetTypesGen, f64)> = vec![];

    for planet_type in PlanetTypesGen::all() {
        planet_frequencies.push((planet_type, planet_type.get_frequency()))
    }

	for _ in 0..planet_count {
        let canidate_planet_type = helper::weighted_random(&mut rng, planet_frequencies.clone());
		loop {
			let canidate_location = (
				rng.gen_range(0.0..size),
				rng.gen_range(0.0..size),
			);

			let canidate_planet_volume = canidate_planet_type.get_radius().powf(2.0) * PI;
			let canidate_radius = (helper::normal_dist(
				&mut rng,
				canidate_planet_volume,
				canidate_planet_volume * 0.2,
			) / PI)
				.sqrt();

			let canidate_density = helper::normal_dist(
				&mut rng,
				canidate_planet_type.get_density(),
				canidate_planet_type.get_density() * 0.2,
			);

			let mut canidate_planet = PlanetGen {
				x: canidate_location.0,
				y: canidate_location.1,
				radius: canidate_radius,
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
