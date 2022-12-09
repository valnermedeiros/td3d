use bevy::prelude::*;
use bevy::utils::FloatOrd;

use crate::components::TowerButtonState;
pub use crate::components::{Bullet, GameAssets, Health, Lifetime, Target, Tower, TowerType};
use crate::physics::PhysicsBundle;
use crate::*;

fn tower_shooting(
    mut commands: Commands,
    targets: Query<&GlobalTransform, With<Target>>,
    bullet_assets: Res<GameAssets>,
    mut towers: Query<(Entity, &mut Tower, &TowerType, &GlobalTransform)>,
    time: Res<Time>,
) {
    for (tower_ent, mut tower, tower_type, transform) in &mut towers {
        tower.shooting_timer.tick(time.delta());
        {
            if tower.shooting_timer.just_finished() {
                let bullet_spawn: Vec3 = transform.translation() + tower.bullet_offset;

                let direction: Option<Vec3> = targets
                    .iter()
                    .min_by_key(|target_transform| {
                        FloatOrd(Vec3::distance(target_transform.translation(), bullet_spawn))
                    })
                    .map(|closest_target| closest_target.translation() - bullet_spawn);

                if let Some(direction) = direction {
                    let (model, bullet) = tower_type.get_bullet(direction, &bullet_assets);
                    commands.entity(tower_ent).with_children(|commands| {
                        commands
                            .spawn(SceneBundle {
                                scene: model,
                                transform: Transform::from_translation(tower.bullet_offset),
                                ..Default::default()
                            })
                            .insert(Lifetime {
                                timer: Timer::from_seconds(10.0, TimerMode::Once),
                            })
                            .insert(Name::new("Bullet"))
                            .insert(bullet)
                            .insert(PhysicsBundle::moving_entity(Vec3::new(0.2, 0.2, 0.2)));
                    });
                }
            }
        }
    }
}

// fn build_tower(
//     mut commands: Commands,
//     selection: Query<(Entity, &Selection, &Transform)>,
//     keyboard: Res<Input<KeyCode>>,

//     assets: Res<GameAssets>,
// ) {
//     if keyboard.just_pressed(KeyCode::Space) {
//         for (entity, selection, transform) in &selection {
//             if selection.selected() {
//                 commands.entity(entity).despawn_recursive();
//                 spawn_tower(&mut commands, &assets, transform.translation);
//             }
//         }
//     }
// }

fn tower_button_clicked(
    interaction: Query<(&Interaction, &TowerType, &TowerButtonState), Changed<Interaction>>,
    mut commands: Commands,
    selection: Query<(Entity, &Selection, &Transform)>,
    mut player: Query<&mut Player>,
    assets: Res<GameAssets>,
) {
    let mut player = player.single_mut();

    for (interaction, tower_type, button_state) in &interaction {
        if matches!(interaction, Interaction::Clicked) {
            for (entity, selection, transform) in &selection {
                if selection.selected() {
                    if player.money >= button_state.cost {
                        player.money -= button_state.cost;
                        //Remove the base model/hitbox
                        commands.entity(entity).despawn_recursive();
                        spawn_tower(&mut commands, &assets, transform.translation, *tower_type);
                    }
                }
            }
        }
    }
}

fn spawn_tower(
    commands: &mut Commands,
    assets: &GameAssets,
    position: Vec3,
    tower_type: TowerType,
) -> Entity {
    let (model, tower) = tower_type.get_tower(&assets);

    commands
        .spawn(SpatialBundle::from_transform(Transform::from_translation(
            position,
        )))
        .insert(Name::new(format!("{:?}_Tower", tower_type)))
        .insert(tower_type)
        .insert(tower)
        .with_children(|commands| {
            commands.spawn(SceneBundle {
                scene: model,
                transform: Transform::from_xyz(0.0, -0.8, 0.0),
                ..Default::default()
            });
        })
        .id()
}

fn create_ui_on_selection(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    //Perf could probably be smarter with change detection
    selections: Query<&Selection>,
    root: Query<Entity, With<TowerUIRoot>>,
) {
    let at_least_one_selected = selections.iter().any(|selection| selection.selected());
    match root.get_single() {
        Ok(root) => {
            if !at_least_one_selected {
                commands.entity(root).despawn_recursive();
            }
        }
        //No root exist
        Err(QuerySingleError::NoEntities(..)) => {
            if at_least_one_selected {
                create_ui(&mut commands, &asset_server);
            }
        }
        _ => unreachable!("Too many ui tower roots!"),
    }
}

fn create_ui(commands: &mut Commands, asset_server: &AssetServer) {
    let button_icons = [
        asset_server.load("tomato_tower.png"),
        asset_server.load("potato_tower.png"),
        asset_server.load("cabbage_tower.png"),
    ];

    let towers = [TowerType::Tomato, TowerType::Potato, TowerType::Cabbage];

    let costs = [50, 80, 110];

    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .insert(TowerUIRoot)
        .with_children(|commands| {
            for i in 0..3 {
                commands
                    .spawn(ButtonBundle {
                        style: Style {
                            size: Size::new(Val::Percent(15.0 * 9.0 / 16.0), Val::Percent(15.0)),
                            align_self: AlignSelf::FlexEnd,
                            margin: UiRect::all(Val::Percent(2.0)),
                            ..default()
                        },
                        image: button_icons[i].clone().into(),
                        ..default()
                    })
                    .insert(TowerButtonState {
                        affordable: false,
                        cost: costs[i],
                    })
                    .insert(towers[i]);
            }
        });
}

fn grey_tower_buttons(
    mut buttons: Query<(&mut BackgroundColor, &mut TowerButtonState)>,
    player: Query<&Player>,
) {
    let player = player.single();

    for (mut tint, mut state) in &mut buttons {
        if player.money >= state.cost {
            state.affordable = true;
            *tint = Color::WHITE.into();
        } else {
            state.affordable = false;
            *tint = Color::DARK_GRAY.into();
        }
    }
}

#[derive(Default)]
pub struct TowerPlugin;

impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Tower>().add_system_set(
            SystemSet::on_update(GameState::InGame)
                .with_system(tower_shooting)
                .with_system(tower_button_clicked)
                .with_system(create_ui_on_selection)
                .with_system(grey_tower_buttons.after(create_ui_on_selection)),
        );
    }
}
