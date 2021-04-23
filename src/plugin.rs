use bevy::prelude::*;

use crate::{
    config::BevyCollisionConfig,
    systems::{add_collider_to_world, apply_velocity, update_bevy_transform, update_events_queues},
    BevyCollisionWorld,
};

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn name(&self) -> &'static str {
        "CollisionPlugin"
    }

    fn build(&self, app: &mut bevy::prelude::AppBuilder) {
        app.insert_resource(BevyCollisionWorld::new(0.02))
            .insert_resource(BevyCollisionConfig::default())
            .add_system(add_collider_to_world.system())
            .add_system(apply_velocity.system())
            .add_system(update_bevy_transform.system())
            .add_system(update_events_queues.system());
    }
}
