use action_orc::*;
use bevy::{platform::collections::HashSet, prelude::*};

use crate::reactor::{Orc, OrcChannel, OrcMessage, OrcNode};

pub trait OrcCommandsExt {
    fn launch_orc(&mut self, graph: Graph);
}

impl OrcCommandsExt for Commands<'_, '_> {
    fn launch_orc(&mut self, graph: Graph) {
        self.queue(move |world: &mut World| {
            let reactor_id = world.commands().spawn_empty().id();
            let mut reactor = Reactor::from(graph);
            let mut entity_map = Vec::new();
            let mut type_ids = HashSet::new();

            for (node_id, meta) in reactor.node_meta() {
                let type_id = *meta.type_id();
                let entity = world
                    .spawn(OrcNode {
                        type_id,
                        reactor_id,
                        node_id,
                    })
                    .id();

                entity_map.push(entity);
                type_ids.insert(type_id);
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

            world
                .commands()
                .entity(reactor_id)
                .insert(Orc::new(reactor, entity_map));
        });
    }
}
