use bevy::{math::Vec3Swizzles, prelude::*};

use crate::components::GameState;
pub use crate::components::{Health, Target, TargetDeathEvent, TargetPath, Tower};
pub use crate::player::Player;

#[derive(Default)]
pub struct TargetPlugin;

impl Plugin for TargetPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Tower>()
            .register_type::<Target>()
            .register_type::<Health>()
            .add_event::<TargetDeathEvent>()
            .insert_resource(TargetPath {
                waypoints: vec![
                    Vec2::new(6.0, 2.0),
                    Vec2::new(6.0, 6.0),
                    Vec2::new(9.0, 9.0),
                ],
            })
            .add_system_set(
                SystemSet::on_update(GameState::InGame)
                    .with_system(move_targets)
                    .with_system(target_death)
                    .with_system(hurt_player.after(move_targets)),
            );
    }
}

fn move_targets(
    mut targets: Query<(&mut Target, &mut Transform)>,
    path: Res<TargetPath>,
    time: Res<Time>,
) {
    for (mut target, mut transform) in &mut targets {
        let delta = target.speed * time.delta_seconds();
        let delta_target = path.waypoints[target.path_index] - transform.translation.xz();

        // This step will get us closer to the goal
        if delta_target.length() > delta {
            let movement = delta_target.normalize() * delta;
            transform.translation += movement.extend(0.0).xzy();
            //Copy for ownership reasons
            let y = transform.translation.y;
            transform.look_at(path.waypoints[target.path_index].extend(y).xzy(), Vec3::Y);
        } else {
            // At current step
            target.path_index += 1;
        }
    }
}

fn target_death(
    mut commands: Commands,
    targets: Query<(Entity, &Health)>,
    mut death_event_writer: EventWriter<TargetDeathEvent>,
) {
    for (entity, health) in &targets {
        if health.value <= 0 {
            death_event_writer.send(TargetDeathEvent);
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn hurt_player(
    mut commands: Commands,
    targets: Query<(Entity, &Target)>,
    path: Res<TargetPath>,
    mut player: Query<&mut Player>,
    audio: Res<Audio>,
    asset_server: Res<AssetServer>,
    mut game_state: ResMut<State<GameState>>,
) {
    for (entity, target) in &targets {
        if target.path_index >= path.waypoints.len() {
            commands.entity(entity).despawn_recursive();

            //Enemies reaching the end of their path could write an event to cause the player to take damage or play audio
            audio.play(asset_server.load("damage.wav"));

            let mut player = player.single_mut();
            if player.health > 0 {
                player.health -= 1;
            }

            if player.health == 0 {
                info!("GAME OVER");
                game_state.set(GameState::GameOver).unwrap();
            }
        }
    }
}
