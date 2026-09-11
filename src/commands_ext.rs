use std::cell::RefCell;

use action_orc::*;
use bevy::prelude::*;

use crate::reactor::{ReactorChannel, ReactorMessage, ScheduleNode, ScheduleReactor};

pub trait ActionOrcCommandsExt {
    fn launch_reactor(&mut self, graph: Graph);
}

impl ActionOrcCommandsExt for Commands<'_, '_> {
    fn launch_reactor(&mut self, graph: Graph) {
        self.queue(move |world: &mut World| {
            let allocator = world.entity_allocator();
            let allocated = RefCell::new(Vec::new());

            let mut reactor = Reactor::from(graph, |meta: &Meta| {
                let entity = allocator.alloc();
                allocated.borrow_mut().push((entity, *meta.type_id()));
                entity
            });

            let reactor_id = world.commands().spawn_empty().id();

            let tx = world.resource::<ReactorChannel>().tx.clone();
            let message_collector = move |entity, event| {
                if let Err(err) = tx.send(ReactorMessage { entity, event }) {
                    panic!("Failed to send message to ReactorBuffer: {err:?}");
                }
            };

            for (entity, type_id) in allocated.into_inner() {
                reactor.listen_for(type_id, message_collector.clone());
                let _ = world.spawn_empty_at(entity);

                world.entity_mut(entity).insert(ScheduleNode {
                    reactor_id,
                    type_id,
                });
            }

            world
                .commands()
                .entity(reactor_id)
                .insert(ScheduleReactor::new(reactor));
        });
    }
}
