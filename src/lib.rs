mod components;
mod config;
mod plugin;
mod systems;

use bevy::prelude::Entity;

pub use components::Velocity;
pub use config::BevyCollisionConfig;
pub use plugin::CollisionPlugin;
pub use systems::{ColliderBuilder, ColliderHandle, ContactEvent, ContactQueue};

pub use ncollide2d as ncollide;

pub type BevyCollisionWorld = ncollide::world::CollisionWorld<f32, Entity>;
