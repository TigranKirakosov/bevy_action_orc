use action_orc::{NodeStatus, Resolution};
use bevy::prelude::*;
use crossbeam::channel::{Receiver, Sender, unbounded};

use crate::{Orc, OrcNode, ResolveNode, registry::OrcTypeRegistry};

#[derive(Resource)]
pub struct OrcChannel {
    pub(crate) tx: Sender<OrcMessage>,
    pub(crate) rx: Receiver<OrcMessage>,
}

pub struct OrcMessage {
    pub(crate) node_entity: Entity,
    pub(crate) status: NodeStatus,
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<OrcChannel>();
    app.add_systems(Update, (init_orcs, drain_orc_messages).chain());
    app.add_observer(resolve_node);
}

fn init_orcs(orcs: Query<&mut Orc, Added<Orc>>) {
    for mut orc in orcs {
        orc.reactor.init().unwrap();
    }
}

fn resolve_node(on: On<ResolveNode>, mut reactors: Query<&mut Orc>) {
    let &ResolveNode {
        reactor_id,
        node_id,
        resolution,
    } = on.event();

    let mut reactor = reactors.get_mut(reactor_id).expect("ReactorId mismatch");
    reactor
        .resolve(node_id, resolution)
        .expect("Node id mismatch");
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

            let mut entity_mut = world.entity_mut(node_entity);

            if let Some(&stale_status) = stale_status {
                if let Some(registration) = type_registry.get(stale_status)
                    && let Some(reflect_component) = registration.data::<ReflectComponent>()
                {
                    reflect_component.remove(&mut entity_mut);
                }
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
