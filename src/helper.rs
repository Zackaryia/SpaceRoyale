use std::collections::HashMap;

use rand::Rng;
use rand_chacha::ChaChaRng;

pub fn count_objects<T: std::hash::Hash + Eq>(objects: Vec<T>) -> HashMap<T, usize> {
	let mut count_map: HashMap<T, usize> = HashMap::new();

	for obj in objects {
		// Use entry API to insert if not present, or update if present
		let counter = count_map.entry(obj).or_insert(0);
		*counter += 1;
	}

	count_map
}

pub fn modified_box_muller_transform(u: f64) -> f64 {
	assert!(u >= 0.0 && u <= 1.0);

	(-2.0 * u.ln()).sqrt() * (u * 2.0 * std::f64::consts::PI).cos()
}

pub fn normal_dist(rng: &mut ChaChaRng, avg: f64, std: f64, bound: f64) -> f64 {
	assert!(bound > 0.0);
	let mut z_score = modified_box_muller_transform(rng.gen_range(0.0..1.0));

	// Bounding the max Z_Score range because having problems with extreme outliers
	if z_score > bound {
		z_score = bound;
	}

	if z_score < -bound {
		z_score = -bound;
	}

	z_score * std + avg
}

pub fn weighted_random<T>(rng: &mut ChaChaRng, data_frequencies: Vec<(T, f64)>) -> T {
	let total_frequency: f64 = data_frequencies.iter().map(|x| x.1).sum();

	let rand_value = rng.gen_range(0.0..total_frequency);

	let mut current_total = 0.0;
	for data in data_frequencies {
		current_total += data.1;

		if current_total > rand_value {
			return data.0;
		}
	}

	unreachable!();
}
