use bevy::prelude::*;
use bevy_collision::{
    BevyCollisionConfig, ColliderBuilder, CollisionPlugin, ContactEvent, ContactQueue, Velocity,
};
use ncollide2d::shape::Cuboid;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(CollisionPlugin)
        .add_startup_system(load_materials.system())
        .add_startup_stage("spawn_objects", SystemStage::single(setup.system()))
        .add_system(move_player.system())
        .add_system(on_player_collision.system())
        .run();
}

pub struct Materials {
    pub white: Handle<ColorMaterial>,
    pub red: Handle<ColorMaterial>,
}

pub struct Player;

fn load_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut config: ResMut<BevyCollisionConfig>,
) {
    let white = materials.add(Color::WHITE.into());
    let red = materials.add(Color::RED.into());

    config.scale = 16.;

    commands.insert_resource(Materials { white, red });
}

fn setup(mut commands: Commands, materials: Res<Materials>) {
    commands.spawn().insert_bundle(OrthographicCameraBundle {
        ..OrthographicCameraBundle::new_2d()
    });

    commands
        .spawn()
        .insert_bundle(SpriteBundle {
            material: materials.white.clone(),
            sprite: Sprite::new(Vec2::new(16., 16.)),
            ..Default::default()
        })
        .insert(ColliderBuilder::new(Cuboid::new([0.5, 0.5].into())).moving(true))
        .insert(Player);

    commands
        .spawn()
        .insert_bundle(SpriteBundle {
            material: materials.white.clone(),
            sprite: Sprite::new(Vec2::new(16., 16.)),
            ..Default::default()
        })
        .insert(ColliderBuilder::new(Cuboid::new([0.5, 0.5].into())).with_translation(2., 2.));
}

fn move_player(input: Res<Input<KeyCode>>, mut query: Query<(&mut Velocity, &Player)>) {
    let x_axis = if input.pressed(KeyCode::Left) {
        -0.1
    } else if input.pressed(KeyCode::Right) {
        0.1
    } else {
        0.
    };

    let y_axis = if input.pressed(KeyCode::Up) {
        0.1
    } else if input.pressed(KeyCode::Down) {
        -0.1
    } else {
        0.
    };

    for (mut vel, _player) in query.iter_mut() {
        vel.0.x = x_axis;
        vel.0.y = y_axis;
    }
}

fn on_player_collision(
    materials: Res<Materials>,
    mut query: Query<(&mut ContactQueue, &mut Handle<ColorMaterial>, &Player)>,
) {
    for (mut queue, mut material, _player) in query.iter_mut() {
        for ev in queue.drain() {
            match ev {
                ContactEvent::Started { .. } => {
                    *material = materials.red.clone();
                }
                bevy_collision::ContactEvent::Stopped { .. } => {
                    *material = materials.white.clone();
                }
            }
        }
    }
}
