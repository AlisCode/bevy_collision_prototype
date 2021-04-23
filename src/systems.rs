use crate::{
    components::Velocity,
    config::BevyCollisionConfig,
    ncollide::{
        na::Isometry2,
        pipeline::{
            narrow_phase::ContactEvent as NCollideContactEvent, CollisionGroups,
            CollisionObjectSlabHandle, GeometricQueryType,
        },
        query::{ContactManifold, Proximity},
    },
};
use bevy::{prelude::*, utils::HashMap};
use ncollide2d::{
    pipeline::CollisionObject,
    shape::{Shape, ShapeHandle},
};

use crate::BevyCollisionWorld;

#[derive(Debug)]
pub enum ContactEvent {
    Started {
        other: Entity,
        manifold: ContactManifold<f32>,
    },
    Stopped {
        other: Entity,
    },
}

#[derive(Debug)]
pub struct ProximityEvent {
    other: Entity,
    old_proximity: Proximity,
    proximity: Proximity,
}

#[derive(Debug)]
pub struct CollisionQueue<E>(pub Vec<E>);

impl<E: 'static> CollisionQueue<E> {
    pub fn drain(&mut self) -> impl Iterator<Item = E> + '_ {
        self.0.drain(0..)
    }
}

impl<T> Default for CollisionQueue<T> {
    fn default() -> Self {
        CollisionQueue(Vec::new())
    }
}

pub type ContactQueue = CollisionQueue<ContactEvent>;
pub type ProximityQueue = CollisionQueue<ProximityEvent>;

pub struct ColliderBuilder {
    translation: Option<(f32, f32)>,
    rotation: Option<f32>,
    shape: ShapeHandle<f32>,
    collision_groups: Option<CollisionGroups>,
    sensor: Option<bool>,
    moving: Option<bool>,
}

impl ColliderBuilder {
    pub fn new<S: Shape<f32>>(shape: S) -> Self {
        Self {
            shape: ShapeHandle::new(shape),
            collision_groups: None,
            sensor: None,
            rotation: None,
            translation: None,
            moving: None,
        }
    }

    pub fn with_translation(mut self, x: f32, y: f32) -> Self {
        self.translation = Some((x, y));
        self
    }

    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = Some(rotation);
        self
    }

    pub fn with_collision_groups(mut self, groups: CollisionGroups) -> Self {
        self.collision_groups = Some(groups);
        self
    }

    pub fn is_sensor(mut self, sensor: bool) -> Self {
        self.sensor = Some(sensor);
        self
    }

    pub fn moving(mut self, moving: bool) -> Self {
        self.moving = Some(moving);
        self
    }

    pub(crate) fn add_to_collision_world(
        &self,
        entity: Entity,
        world: &mut BevyCollisionWorld,
    ) -> CollisionObjectSlabHandle {
        let (x, y) = self.translation.unwrap_or_else(|| (0., 0.));
        let rotation = self.rotation.unwrap_or_else(|| 0.);
        let collision_groups = self
            .collision_groups
            .unwrap_or_else(|| CollisionGroups::new());

        let query_type = if self.sensor.unwrap_or_else(|| false) {
            GeometricQueryType::Proximity(0.)
        } else {
            GeometricQueryType::Contacts(0., 0.)
        };

        let (handle, _) = world.add(
            Isometry2::new([x, y].into(), rotation),
            self.shape.clone(),
            collision_groups,
            query_type,
            entity,
        );

        handle
    }
}

#[derive(Debug)]
pub struct ColliderHandle(pub CollisionObjectSlabHandle);

pub fn add_collider_to_world(
    mut commands: Commands,
    mut collision_world: ResMut<BevyCollisionWorld>,
    query: Query<(Entity, &ColliderBuilder)>,
) {
    for (entity, builder) in query.iter() {
        let mut add_contact_queue = true;
        if let Some(true) = builder.sensor {
            add_contact_queue = false;
        }

        let mut moving = false;
        if let Some(true) = builder.moving {
            moving = true;
        }

        let handle = builder.add_to_collision_world(entity, &mut collision_world);
        commands
            .entity(entity)
            .remove::<ColliderBuilder>()
            .insert_bundle((ColliderHandle(handle), ProximityQueue::default()));

        if add_contact_queue {
            commands.entity(entity).insert(ContactQueue::default());
        }
        if moving {
            commands.entity(entity).insert(Velocity(Vec2::ZERO));
        }
    }
}

pub fn update_bevy_transform(
    config: Res<BevyCollisionConfig>,
    mut collision_world: ResMut<BevyCollisionWorld>,
    mut objects: Query<(&ColliderHandle, &mut Transform)>,
) {
    for (handle, mut transform) in objects.iter_mut() {
        let body = collision_world
            .get_mut(handle.0)
            .expect("Failed to find a collider");
        let Isometry2 {
            translation,
            rotation,
        } = body.position();
        transform.translation.x = translation.x * config.scale;
        transform.translation.y = translation.y * config.scale;
        transform.rotation = Quat::from_rotation_z(rotation.angle());
    }
    collision_world.update();
}

pub fn apply_velocity(
    mut collision_world: ResMut<BevyCollisionWorld>,
    query: Query<(&ColliderHandle, &Velocity)>,
) {
    for (handle, vel) in query.iter() {
        if let Some(collider) = collision_world.get_mut(handle.0) {
            let pos = collider.position();
            let mut translation = pos.translation.vector;
            translation.x += vel.0.x;
            translation.y += vel.0.y;
            collider.set_position(Isometry2::new(translation, 0.));
        }
    }
}

pub fn update_events_queues(
    collision_world: Res<BevyCollisionWorld>,
    mut query_contacts: Query<&mut ContactQueue>,
    mut query_proximity: Query<&mut ProximityQueue>,
) {
    let contacts_map: HashMap<
        (CollisionObjectSlabHandle, CollisionObjectSlabHandle),
        (Entity, Entity, ContactManifold<f32>),
    > = collision_world
        .contact_pairs(true)
        .map(|(a, b, _, manifold)| {
            let collider_a = collision_world
                .collision_object(a)
                .expect("Collider does not exist");
            let collider_b = collision_world
                .collision_object(b)
                .expect("Collider does not exist");
            (
                (a, b),
                (*collider_a.data(), *collider_b.data(), manifold.clone()),
            )
        })
        .collect();

    let mut contact_events: HashMap<Entity, Vec<ContactEvent>> = HashMap::default();
    collision_world
        .narrow_phase
        .contact_events()
        .iter()
        .for_each(|event| {
            match event {
                NCollideContactEvent::Started(a, b) => {
                    let (entity_a, entity_b, manifold) = contacts_map
                        .get(&(*a, *b))
                        .or_else(|| contacts_map.get(&(*b, *a)))
                        .expect("Failed to find contact info");

                    let entry = contact_events.entry(*entity_a).or_default();
                    entry.push(ContactEvent::Started {
                        other: *entity_b,
                        manifold: manifold.clone(),
                    });

                    let entry = contact_events.entry(*entity_b).or_default();
                    entry.push(ContactEvent::Started {
                        other: *entity_a,
                        manifold: manifold.clone(),
                    });
                }
                NCollideContactEvent::Stopped(a, b) => {
                    let entity_a = collision_world
                        .collision_object(*a)
                        .expect("Failed to get CollisionObject")
                        .data();
                    let entity_b = collision_world
                        .collision_object(*b)
                        .expect("Failed to get CollisionObject")
                        .data();

                    let entry = contact_events.entry(*entity_a).or_default();
                    entry.push(ContactEvent::Stopped { other: *entity_b });

                    let entry = contact_events.entry(*entity_b).or_default();
                    entry.push(ContactEvent::Stopped { other: *entity_a });
                }
            };
        });

    let proximities: HashMap<
        (CollisionObjectSlabHandle, CollisionObjectSlabHandle),
        (Proximity, Proximity),
    > = collision_world
        .narrow_phase
        .proximity_events()
        .iter()
        .flat_map(|event| {
            let a = event.collider1;
            let b = event.collider2;
            vec![
                ((a, b), (event.prev_status, event.new_status)),
                ((b, a), (event.prev_status, event.new_status)),
            ]
        })
        .collect();

    let mut proximity_map: HashMap<Entity, Vec<ProximityEvent>> = HashMap::default();
    collision_world
        .proximity_pairs(true)
        .for_each(|(a, b, _, _)| {
            let collider_a = collision_world
                .collision_object(a)
                .expect("Collider does not exist");
            let collider_b = collision_world
                .collision_object(b)
                .expect("Collider does not exist");
            let (old, new) = proximities[&(a, b)];
            add_proximity_to_queue(&mut proximity_map, collider_a, collider_b, old, new);
            add_proximity_to_queue(&mut proximity_map, collider_b, collider_a, old, new);
        });

    for (entity, events) in contact_events.into_iter() {
        let mut queue = query_contacts
            .get_mut(entity)
            .expect("Failed to get contact queue");
        queue.0 = events;
    }

    for (entity, events) in proximity_map.into_iter() {
        let mut queue = query_proximity
            .get_mut(entity)
            .expect("Failed to get contact queue");
        queue.0 = events;
    }
}

fn add_proximity_to_queue(
    map: &mut HashMap<Entity, Vec<ProximityEvent>>,
    a: &CollisionObject<f32, Entity>,
    b: &CollisionObject<f32, Entity>,
    old_proximity: Proximity,
    proximity: Proximity,
) {
    let entry = map.entry(*a.data()).or_default();
    entry.push(ProximityEvent {
        other: *b.data(),
        old_proximity,
        proximity,
    });
}
