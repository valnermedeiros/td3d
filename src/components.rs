use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Target {
    pub speed: f32,
    pub path_index: usize,
}

#[derive(Resource)]
pub struct TargetPath {
    pub waypoints: Vec<Vec2>,
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Lifetime {
    pub timer: Timer,
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Bullet {
    pub direction: Vec3,
    pub speed: f32,
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Tower {
    pub shooting_timer: Timer,
    pub bullet_offset: Vec3,
}
#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Health {
    pub value: i32,
}

#[derive(Resource)]
pub struct GameAssets {
    pub tower_base_scene: Handle<Scene>,
    pub tomato_tower_scene: Handle<Scene>,
    pub tomato_scene: Handle<Scene>,
    pub potato_tower_scene: Handle<Scene>,
    pub potato_scene: Handle<Scene>,
    pub cabbage_tower_scene: Handle<Scene>,
    pub cabbage_scene: Handle<Scene>,
    pub target_scene: Handle<Scene>,
}

#[derive(Component)]
pub struct TowerUIRoot;

#[derive(Inspectable, Component, Clone, Copy, Debug)]

pub enum TowerType {
    Tomato,
    Potato,
    Cabbage,
}

impl TowerType {
    pub fn get_tower(&self, assets: &GameAssets) -> (Handle<Scene>, Tower) {
        match self {
            TowerType::Tomato => (
                assets.tomato_tower_scene.clone(),
                Tower {
                    shooting_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
                    bullet_offset: Vec3::new(0.0, 0.2, 0.0),
                },
            ),
            TowerType::Potato => (
                assets.potato_tower_scene.clone(),
                Tower {
                    shooting_timer: Timer::from_seconds(0.7, TimerMode::Repeating),
                    bullet_offset: Vec3::new(0.0, 0.2, 0.0),
                },
            ),
            TowerType::Cabbage => (
                assets.cabbage_tower_scene.clone(),
                Tower {
                    shooting_timer: Timer::from_seconds(0.8, TimerMode::Repeating),
                    bullet_offset: Vec3::new(0.0, 0.2, 0.0),
                },
            ),
        }
    }

    pub fn get_bullet(&self, direction: Vec3, assets: &GameAssets) -> (Handle<Scene>, Bullet) {
        match self {
            TowerType::Tomato => (
                assets.tomato_scene.clone(),
                Bullet {
                    direction,
                    speed: 3.5,
                },
            ),
            TowerType::Potato => (
                assets.potato_scene.clone(),
                Bullet {
                    direction,
                    speed: 6.5,
                },
            ),
            TowerType::Cabbage => (
                assets.cabbage_scene.clone(),
                Bullet {
                    direction,
                    speed: 8.5,
                },
            ),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    MainMenu,
    InGame,
    GameOver,
}

pub struct TargetDeathEvent;

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct TowerButtonState {
    pub cost: u32,
    pub affordable: bool,
}
