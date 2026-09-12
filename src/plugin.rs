use action_orc::{NodeStatus, Resolution};
use bevy::prelude::*;

use crate::reactor::{Orc, OrcChannel, OrcMessage, OrcNode, OrcTypeRegistry, ResolveNode};

pub struct ActionOrcPlugin;
impl Plugin for ActionOrcPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OrcChannel>();
        app.init_resource::<OrcTypeRegistry>();
        app.add_systems(Update, (init_orcs, drain_orc_messages).chain());
        app.add_observer(resolve_node);
    }
}

fn init_orcs(orcs: Query<&mut Orc, Added<Orc>>) {
    for mut orc in orcs {
        orc.reactor.init().unwrap();
    }
}

fn resolve_node(on: On<ResolveNode>, mut reactors: Query<&mut Orc>) {
    let ResolveNode {
        reactor_id,
        node_id,
    } = on.event();

    let mut reactor = reactors.get_mut(*reactor_id).expect("ReactorId mismatch");
    reactor
        .resolve(*node_id, Resolution::Finished)
        .expect("Node id mismatch");
}

fn drain_orc_messages(world: &mut World) {
    let channel = world.resource::<OrcChannel>();
    let messages: Vec<_> = channel.rx.try_iter().collect();

    world.resource_scope::<OrcTypeRegistry, ()>(|world: &mut World, reactor_registry| {
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

            let node_status = (match status {
                NodeStatus::Started => reactor_registry.started.get(&node.type_id),
                NodeStatus::Resolved(res) => match res {
                    Resolution::Finished => reactor_registry.finished.get(&node.type_id),
                },
            })
            .expect("Component TypeId mistmatch.");

            let registration = type_registry
                .get(*node_status)
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
