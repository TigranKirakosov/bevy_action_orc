use action_orc::{NodeStatus, Resolution};
use bevy::prelude::*;
use crossbeam::channel::{Receiver, Sender, unbounded};

use crate::{
    Orc, OrcNode, ResolveNode,
    lifecycle::{Active, Finished},
    registry::OrcTypeRegistry,
};

#[derive(Resource)]
pub struct OrcChannel {
    pub(crate) tx: Sender<OrcMessage>,
    pub(crate) rx: Receiver<OrcMessage>,
}

pub struct OrcMessage {
    pub(crate) node_entity: Entity,
    pub(crate) status: NodeStatus,
}

#[derive(Component)]
struct DespawnOrc;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<OrcChannel>();
    app.add_systems(Update, (init_orcs, drain_orc_messages).chain());
    app.add_systems(PostUpdate, terminate_orcs);
    app.add_observer(resolve_node);
}

fn terminate_orcs(mut commands: Commands, orcs: Query<(Entity, &Orc), With<DespawnOrc>>) {
    for (entity, orc) in orcs {
        if orc.active_node_count == 0 {
            // OrcNode's get despawned recursively as they were attached as children
            commands.entity(entity).despawn();
        }
    }
}

fn init_orcs(orcs: Query<&mut Orc, Added<Orc>>) {
    for mut orc in orcs {
        orc.reactor.start().unwrap();
    }
}

fn resolve_node(on: On<ResolveNode>, mut commands: Commands, mut reactors: Query<&mut Orc>) {
    let &ResolveNode {
        reactor_id,
        node_id,
        resolution,
    } = on.event();

    let mut reactor = reactors.get_mut(reactor_id).expect("ReactorId mismatch");
    let schedule_over = reactor
        .resolve(node_id, resolution)
        .expect("Node id mismatch");

    let &node_entity = reactor.entity(node_id).expect("NodeId to Enitity mismatch");

    if schedule_over && reactor.loop_schedule {
        reactor.restart().expect("Failed to restart Reactor");
        reactor.active_node_count = reactor.entity_map.len();
        for &entity in &reactor.entity_map {
            // commands
            //     .entity(entity)
            //     .remove::<(Active<T>, Finished<T>, Resolved)>();
        }
    } else {
        // Schedule despawn instead of immediate despawn
        commands.entity(reactor_id).insert(DespawnOrc);
    }
}

fn drain_orc_messages(world: &mut World) {
    let channel = world.resource::<OrcChannel>();
    let messages: Vec<_> = channel.rx.try_iter().collect();

    if messages.is_empty() {
        return;
    }

    world.resource_scope::<OrcTypeRegistry, ()>(|world: &mut World, orc_type_registry| {
        let app_type_registry = world.resource::<AppTypeRegistry>().clone();
        let type_registry = app_type_registry.read();

        for OrcMessage {
            node_entity,
            status,
        } in messages
        {
            let node = world
                .get::<OrcNode>(node_entity)
                .expect("Missing orc node component.");
            let reactor_id = node.reactor_id;

            let (new_status, stale_status) = match status {
                NodeStatus::Started => {
                    let target = orc_type_registry
                        .active
                        .get(&node.type_id)
                        .expect("Missing active type registration");
                    (target, None)
                }
                NodeStatus::Resolved(Resolution::Finished) => {
                    let target = orc_type_registry
                        .finished
                        .get(&node.type_id)
                        .expect("Missing finished type registration");
                    let stale = orc_type_registry.active.get(&node.type_id);
                    (target, stale)
                }
            };

            if let Some(&stale_status) = stale_status {
                if let Some(registration) = type_registry.get(stale_status)
                    && let Some(reflect_component) = registration.data::<ReflectComponent>()
                {
                    let mut entity_mut = world.entity_mut(node_entity);
                    reflect_component.remove(&mut entity_mut);
                }
            }

            let mut orc = world
                .get_mut::<Orc>(reactor_id)
                .expect("Missing parent Reactor");

            if stale_status.is_some() {
                orc.active_node_count = orc.active_node_count.saturating_sub(1);
            } else {
                orc.active_node_count += 1;
            }

            let registration = type_registry
                .get(*new_status)
                .expect("Missing registration for type");
            let reflect_component = registration
                .data::<ReflectComponent>()
                .expect("Missing Reflect impl for type");
            let reflect_default = registration
                .data::<ReflectDefault>()
                .expect("Missing Default impl for type");

            let marker_instance = reflect_default.default();

            let mut entity_mut = world.entity_mut(node_entity);
            reflect_component.insert(
                &mut entity_mut,
                marker_instance.as_partial_reflect(),
                &type_registry,
            );
        }
    });
}

impl Default for OrcChannel {
    fn default() -> Self {
        let (tx, rx) = unbounded();
        Self { tx, rx }
    }
}
