use action_orc::{NodeStatus, Resolution};
use bevy::prelude::*;
use crossbeam::channel::{Receiver, Sender, unbounded};

use crate::{
    Orc, OrcError, OrcNode, ResolveNode,
    reflect::{reflect_insert_default_component, reflect_remove_component},
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
struct Restart;

#[derive(Component)]
struct Shutdown;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<OrcChannel>();
    app.add_systems(Update, (start_orcs, drain_orc_messages).chain());
    app.add_systems(
        PostUpdate,
        (restart_orcs, shutdown_orcs, check_graph_completion).chain(),
    );
    app.add_observer(resolve_node);
}

fn check_graph_completion(
    mut commands: Commands,
    reactors: Query<(Entity, &Orc), (Without<Restart>, Without<Shutdown>)>,
) {
    for (entity, orc) in &reactors {
        if orc.is_schedule_complete() {
            if orc.loop_schedule {
                commands.entity(entity).insert(Restart);
            } else {
                commands.entity(entity).insert(Shutdown);
            }
        }
    }
}

fn start_orcs(orcs: Query<&mut Orc, Added<Orc>>) -> Result {
    for mut orc in orcs {
        orc.reactor.start()?;
    }

    Ok(())
}

fn shutdown_orcs(mut commands: Commands, orcs: Query<(Entity, &Orc), With<Shutdown>>) {
    for (entity, orc) in orcs {
        if orc.is_schedule_complete() {
            // OrcNode's get despawned recursively as they were attached as children
            commands.entity(entity).despawn();
        }
    }
}

fn restart_orcs(mut commands: Commands, orcs: Query<(Entity, &mut Orc), With<Restart>>) -> Result {
    for (entity, mut orc) in orcs {
        if !orc.is_schedule_complete() {
            continue;
        }

        commands.entity(entity).remove::<Restart>();

        orc.restart()?;
        orc.pending_node_statuses = orc.entity_map.len() as i32;

        for &entity in &orc.entity_map {
            commands.queue(move |world: &mut World| -> Result {
                world.resource_scope::<OrcTypeRegistry, Result>(
                    |world: &mut World, orc_type_registry| -> Result {
                        let app_type_registry = world.resource::<AppTypeRegistry>().clone();
                        let type_registry = app_type_registry.read();

                        let OrcNode { meta, .. } = world
                            .get::<OrcNode>(entity)
                            .ok_or(OrcError::ReactorMissing(entity))?;

                        let (type_id, type_name) = (meta.type_id(), meta.type_name());

                        let active = orc_type_registry.active.get(type_id).ok_or(
                            OrcError::TypeIdMissing {
                                status: "Active",
                                type_name,
                            },
                        )?;

                        let finished = orc_type_registry.finished.get(type_id).ok_or(
                            OrcError::TypeIdMissing {
                                status: "Finished",
                                type_name,
                            },
                        )?;

                        for &type_id in [active, finished] {
                            reflect_remove_component(world, entity, type_id, &type_registry);
                        }
                        Ok(())
                    },
                )?;
                Ok(())
            });
        }
    }
    Ok(())
}

fn resolve_node(on: On<ResolveNode>, mut reactors: Query<&mut Orc>) -> Result {
    let &ResolveNode {
        reactor_id,
        node_id,
        resolution,
        ref options,
    } = on.event();

    let mut reactor = reactors.get_mut(reactor_id)?;
    let _ = reactor.resolve(node_id, resolution)?;
    reactor.loop_schedule = options.loop_schedule;

    Ok(())
}

fn drain_orc_messages(world: &mut World) -> Result {
    let channel = world.resource::<OrcChannel>();
    let messages: Vec<_> = channel.rx.try_iter().collect();

    if messages.is_empty() {
        return Ok(());
    }

    world.resource_scope::<OrcTypeRegistry, Result>(
        |world: &mut World, orc_type_registry| -> Result {
            let app_type_registry = world.resource::<AppTypeRegistry>().clone();
            let type_registry = app_type_registry.read();

            for OrcMessage {
                node_entity,
                status,
            } in messages
            {
                let &OrcNode {
                    reactor_id,
                    ref meta,
                    ..
                } = world
                    .get::<OrcNode>(node_entity)
                    .ok_or(OrcError::NodeMissing(node_entity))?;

                let (type_id, type_name) = (meta.type_id(), meta.type_name());

                let (new_status, stale_status) = match status {
                    NodeStatus::Started => {
                        let target = orc_type_registry.active.get(type_id).ok_or(
                            OrcError::TypeIdMissing {
                                status: "Active",
                                type_name,
                            },
                        )?;
                        (target, None)
                    }
                    NodeStatus::Resolved(Resolution::Finished) => {
                        let target = orc_type_registry.finished.get(type_id).ok_or(
                            OrcError::TypeIdMissing {
                                status: "Finished",
                                type_name,
                            },
                        )?;
                        let stale = orc_type_registry.active.get(type_id);
                        (target, stale)
                    }
                };

                let status_delta = match status {
                    NodeStatus::Started => 1,
                    NodeStatus::Resolved(Resolution::Finished) => -1,
                };

                let mut orc = world
                    .get_mut::<Orc>(reactor_id)
                    .ok_or(OrcError::ReactorMissing(reactor_id))?;
                orc.apply_pending_statuses_delta(status_delta);

                if let Some(&stale_status) = stale_status {
                    reflect_remove_component(world, node_entity, stale_status, &type_registry);
                }

                reflect_insert_default_component(world, node_entity, *new_status, &type_registry);
            }
            Ok(())
        },
    )?;

    Ok(())
}

impl Default for OrcChannel {
    fn default() -> Self {
        let (tx, rx) = unbounded();
        Self { tx, rx }
    }
}
