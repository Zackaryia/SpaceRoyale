
use bevy::prelude::*;
use bevy_xpbd_2d::prelude::*;
use bevy_hanabi::prelude::*;

pub struct ThrustPlugin;

// impl Plugin for ThrustPlugin {
//     fn build(&self, app: &mut App) {
//         app.add_systems(Startup, setup_thrust_particles);
//     }
// }


pub fn setup_thrust_particles(mut effects: ResMut<Assets<EffectAsset>>) -> Handle<EffectAsset> {
    let mut color_gradient1 = Gradient::new();
    color_gradient1.add_key(0.0, Vec4::splat(1.0));
    color_gradient1.add_key(0.1, Vec4::new(1.0, 1.0, 0.0, 1.0));
    color_gradient1.add_key(0.4, Vec4::new(1.0, 0.0, 0.0, 1.0));
    color_gradient1.add_key(1.0, Vec4::splat(0.0));

    let x = 100.;
    let mut size_gradient1 = Gradient::new();
    size_gradient1.add_key(0.0, Vec2::splat(0.1 * x));
    size_gradient1.add_key(0.5, Vec2::splat(0.5 * x));
    size_gradient1.add_key(0.8, Vec2::splat(0.08 * x));
    size_gradient1.add_key(1.0, Vec2::splat(0.0 * x));

    let writer1 = ExprWriter::new();

    let age1 = writer1.lit(0.).expr();
    let init_age1 = SetAttributeModifier::new(Attribute::AGE, age1);

    let lifetime1 = writer1.lit(2.).expr();
    let init_lifetime1 = SetAttributeModifier::new(Attribute::LIFETIME, lifetime1);

    // Add constant downward acceleration to simulate gravity
    // let accel1 = writer1.lit(Vec3::Y * -2.).expr();
    // let update_accel1 = AccelModifier::new(accel1);

    let emitter_angle_sin = writer1.lit(Attribute::AXIS_Z).cast(ScalarType::Float).sin();
    let emitter_angle_cos = writer1.lit(Attribute::AXIS_Z).cast(ScalarType::Float).cos();
    
    let emitter_position = Vec2::from([emitter_angle_sin, emitter_angle_cos]);

    let init_pos1 = SetPositionCircleModifier {
        center: writer1.lit(emitter_position).expr(),
        axis: writer1.lit(Vec3::Z).expr(),
        radius: writer1.lit(4.).expr(),
        dimension: ShapeDimension::Volume,
        // base_radius: writer1.lit(2.).expr(),
        // top_radius: writer1.lit(8.).expr(),
        // height: writer1.lit(15.).expr(),
        // dimension: ShapeDimension::Volume,
    };

    let init_vel1 = SetVelocitySphereModifier {
        center: writer1.lit(Vec3::from([0., 0., 0.])).expr(),
        speed: writer1.lit(500.).expr(),
    };

    let effect1 = effects.add(
        EffectAsset::new(32768, Spawner::rate(500.0.into()), writer1.finish())
            .with_name("emit:rate")
            .with_property("my_accel", Vec3::new(0., 0., 0.).into())
            .init(init_pos1)
            // Make spawned particles move away from the emitter origin
            .init(init_vel1)
            .init(init_age1)
            .init(init_lifetime1)
            // .update(update_accel1)
            .render(ColorOverLifetimeModifier {
                gradient: color_gradient1,
            })
            .render(SizeOverLifetimeModifier {
                gradient: size_gradient1,
                screen_space_size: false,
            }),
    );

    effect1
}