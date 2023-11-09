use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::transform::TransformSystem;
use bevy_xpbd_2d::prelude::*;
use planet::PlanetPlugin;
use player::{Player, PlayerPlugin};
use bevy_hanabi::prelude::*;
// use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy::prelude::*;
// use bevy_inspector_egui::quick::WorldInspectorPlugin;

mod planet;
mod player;
mod thrust;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        // .add_plugins(WorldInspectorPlugin::new())

        .add_plugins(HanabiPlugin)
        .add_plugins((PlanetPlugin, PlayerPlugin))
        .add_plugins((
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin::default(),
        ))
        .insert_resource(Time::<Fixed>::from_seconds(1. / 60.))
        .add_systems(Startup, setup)
        // .add_systems(FixedUpdate, )
        .add_systems(
            PostUpdate,
            update_camera
                .after(PhysicsSet::Sync)
                .before(TransformSystem::TransformPropagate),
        )
        // .add_plugins(WorldInspectorPlugin::default())
        // .add_systems(Update, (update_accel))
        .run();
}

#[derive(Bundle)]
struct PlayerCameraBundle {
    player_camera: PlayerCamera,
    camera: Camera2dBundle,
}

#[derive(Component)]
struct PlayerCamera;

fn setup(mut commands: Commands) {
    commands.spawn(PlayerCameraBundle {
        player_camera: PlayerCamera,
        camera: Camera2dBundle {
            transform: Transform::from_xyz(1200., 0., 1.),
            projection: OrthographicProjection {
                scale: 7.,
                ..default()
            },
            ..default()
        },
    });
}

fn update_camera(
    player_position_query: Query<&Position, With<Player>>,
    mut camera_query: Query<&mut Transform, With<PlayerCamera>>,
    time: Res<Time>,
) {
    let player_position = player_position_query.get_single().unwrap().extend(1.);
    let mut transform = camera_query.get_single_mut().unwrap();

    transform.translation = transform
        .translation
        .lerp(player_position.as_vec3(), time.delta_seconds() * 10.);
}
