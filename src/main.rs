use bevy::{
    core_pipeline::clear_color::ClearColorConfig, ecs::query::QuerySingleError,
    pbr::NotShadowCaster, prelude::*, render::view::RenderLayers,
};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_mod_picking::*;
use bevy_rapier3d::{
    prelude::{NoUserData, RapierConfiguration, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};
use simula_action::ActionPlugin;
use simula_camera::{flycam::*, orbitcam::*};
use simula_video::rt;

mod bullet;
mod components;
mod main_menu;
mod physics;
mod player;
mod target;
mod tower;

pub use bullet::*;
use main_menu::*;
use physics::{PhysicsBundle, PhysicsPlugin};
pub use player::*;
pub use target::*;
pub use tower::*;

pub const HEIGHT: f32 = 720.0;
pub const WIDTH: f32 = 1280.0;

use crate::components::{GameAssets, GameState, Health, Target, TowerUIRoot};

fn main() {
    let mut app = App::new();

    app.insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                width: WIDTH,
                height: HEIGHT,
                resizable: false,
                ..default()
            },
            ..default()
        }))
        .add_plugin(EguiPlugin)
        .add_plugin(ActionPlugin)
        .add_plugin(OrbitCameraPlugin)
        .add_plugin(FlyCameraPlugin)
        // init physics
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        // mod picking
        .add_plugins(DefaultPickingPlugins)
        //game state
        .add_state(GameState::MainMenu)
        .add_plugin(MainMenuPlugin)
        .add_startup_system(spawn_camera)
        // add system set to run on inGame state
        .add_system_set(SystemSet::on_enter(GameState::InGame).with_system(spawn_scene))
        // .add_startup_system(spawn_scene)
        .add_plugin(BulletPlugin)
        .add_plugin(TowerPlugin)
        .add_plugin(TargetPlugin)
        .add_plugin(PhysicsPlugin)
        .add_plugin(PlayerPlugin)
        // .add_system(what_is_selected)
        // .add_startup_system(setup)
        .add_startup_system_to_stage(StartupStage::PreStartup, asset_loading)
        .add_plugin(WorldInspectorPlugin::new())
        .run();
}

fn spawn_camera(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let rt_image = images.add(rt::common_render_target_image(UVec2 { x: 256, y: 256 }));

    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 2.0, -10.0)
                .looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
            ..default()
        })
        .insert(RenderLayers::all())
        .insert(PickingCameraBundle::default())
        .with_children(|parent| {
            let mut _child = parent.spawn(Camera3dBundle {
                camera_3d: Camera3d {
                    clear_color: ClearColorConfig::Custom(Color::BLACK),
                    ..default()
                },
                camera: Camera {
                    priority: -1,
                    target: bevy::render::camera::RenderTarget::Image(rt_image.clone()),
                    ..default()
                },
                ..default()
            });
        })
        .insert(FlyCamera::default());
}

fn spawn_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut rapier_config: ResMut<RapierConfiguration>,
    game_assets: Res<GameAssets>,
) {
    rapier_config.gravity = Vec3::ZERO;

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 50.0 })),
            material: materials.add(Color::rgb(0.2, 1.0, 0.2).into()),
            // transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        })
        .insert(Name::new("Floor"));

    // spawn light

    commands
        .spawn(PointLightBundle {
            point_light: PointLight {
                intensity: 1500.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(4.0, 8.0, 4.0),
            ..default()
        })
        .insert(Name::new("Light"));

    let default_collider_color = materials.add(Color::rgba(0.3, 0.5, 0.3, 0.3).into());
    let selected_collider_color = materials.add(Color::rgba(0.3, 0.9, 0.3, 0.9).into());

    for i in 0..10 {
        for j in 0..2 {
            commands
                .spawn(SpatialBundle::from_transform(Transform::from_xyz(
                    2.0 * i as f32 + j as f32,
                    0.8,
                    5.0 * j as f32,
                )))
                .insert(Name::new("Tower_Base"))
                .insert(meshes.add(shape::Capsule::default().into()))
                .insert(Highlighting {
                    initial: default_collider_color.clone(),
                    hovered: Some(selected_collider_color.clone()),
                    pressed: Some(selected_collider_color.clone()),
                    selected: Some(selected_collider_color.clone()),
                })
                .insert(default_collider_color.clone())
                .insert(NotShadowCaster)
                .insert(PickableBundle::default())
                .with_children(|commands| {
                    commands.spawn(SceneBundle {
                        scene: game_assets.tower_base_scene.clone(),
                        transform: Transform::from_xyz(0.0, -0.8, 0.0),
                        ..Default::default()
                    });
                });
        }
    }

    for i in 1..25 {
        commands
            .spawn(SceneBundle {
                scene: game_assets.target_scene.clone(),
                transform: Transform::from_xyz(-2.0 * i as f32, 0.4, 2.5),
                ..Default::default()
            })
            .insert(Target {
                speed: 0.45,
                ..Default::default()
            })
            .insert(Health { value: 3 })
            .insert(Name::new("Target"))
            .insert(PhysicsBundle::moving_entity(Vec3::new(0.2, 0.2, 0.2)));
    }
}

// fn what_is_selected(selection: Query<(&Name, &Selection)>) {
//     for (name, selection) in &selection {
//         if selection.selected() {
//             info!("{}", name);
//         }
//     }
// }

fn asset_loading(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(GameAssets {
        tower_base_scene: assets.load("TowerBase.glb#Scene0"),
        tomato_tower_scene: assets.load("TomatoTower.glb#Scene0"),
        tomato_scene: assets.load("Tomato.glb#Scene0"),
        potato_tower_scene: assets.load("PotatoTower.glb#Scene0"),
        potato_scene: assets.load("Potato.glb#Scene0"),
        cabbage_tower_scene: assets.load("CabbageTower.glb#Scene0"),
        cabbage_scene: assets.load("Cabbage.glb#Scene0"),
        target_scene: assets.load("Target.glb#Scene0"),
    });
}

// fn setup(mut commands: Commands) {}
