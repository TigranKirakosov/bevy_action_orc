use action_orc::*;
use bevy::prelude::*;

use crate::{GraphConfig, Orc, OrcNode};

pub trait OrcCommandsExt {
    fn queue_graph<'a, G: AsGraphEntryProxy<'a>>(&mut self, graph: G, config: GraphConfig);
}

impl OrcCommandsExt for Commands<'_, '_> {
    fn queue_graph<'a, G: AsGraphEntryProxy<'a>>(&mut self, graph: G, config: GraphConfig) {
        let graph = graph.into_compiled_graph();

        self.queue(move |world: &mut World| -> Result {
            let orchestrator_id = world.spawn_empty().id();
            let orchestrator = Orchestrator::new(
                &graph,
                ScheduleConfig {
                    should_loop: config.loop_schedule,
                },
            )?;
            let node_meta = orchestrator.node_meta();
            let mut entity_map = Vec::with_capacity(node_meta.len());

            for (node_id, _) in node_meta {
                let entity = world
                    .spawn((
                        ChildOf(orchestrator_id),
                        OrcNode {
                            orc_id: orchestrator_id,
                            node_id,
                        },
                    ))
                    .id();

                entity_map.push(entity);
            }

            world.entity_mut(orchestrator_id).insert(Orc {
                orchestrator,
                entity_map,
            });

            Ok(())
        });
    }
}
