use action_orc::*;
use bevy::{platform::collections::HashSet, prelude::*};

use crate::{
    GraphConfig, Orc, OrcNode,
    schedule::{OrcChannel, OrcMessage},
};

pub trait OrcCommandsExt {
    fn queue_graph<'a, G: AsGraphEntryProxy<'a>>(&mut self, graph: G, config: GraphConfig);
}

impl OrcCommandsExt for Commands<'_, '_> {
    fn queue_graph<'a, G: AsGraphEntryProxy<'a>>(&mut self, graph: G, config: GraphConfig) {
        let graph = graph.into_compiled_graph();

        self.queue(move |world: &mut World| {
            let reactor_id = world.commands().spawn_empty().id();
            let mut reactor = Reactor::from(graph);
            let mut entity_map = Vec::new();
            let mut type_ids = HashSet::new();

            for (node_id, meta) in reactor.node_meta() {
                let entity = world
                    .spawn((
                        ChildOf(reactor_id),
                        OrcNode {
                            meta: meta.clone(),
                            reactor_id,
                            node_id,
                        },
                    ))
                    .id();

                entity_map.push(entity);
                type_ids.insert(*meta.type_id());
            }

            let tx = world.resource::<OrcChannel>().tx.clone();
            let entity_map_clone = entity_map.clone();

            let message_collector = move |node_id, status| {
                let node_entity = entity_map_clone[node_id];
                if let Err(err) = tx.send(OrcMessage {
                    node_entity,
                    status,
                }) {
                    panic!("Failed to send orc message: {err:?}");
                }
            };

            for type_id in type_ids {
                reactor
                    .listen_for(type_id, message_collector.clone())
                    .unwrap();
            }

            world.commands().entity(reactor_id).insert(Orc::new(
                reactor,
                entity_map,
                config.loop_schedule,
            ));
        });
    }
}
