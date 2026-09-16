use action_orc::{
    LoopDirective, NodeId, NodeStatus, Resolution, ScheduleDirectives, ScheduleState,
};
use bevy::prelude::*;

use crate::{Orc, OrcNode, registry::OrcRegistry};

#[derive(Message, Clone)]
pub(crate) struct ResolveNode {
    pub(crate) orc_id: Entity,
    pub(crate) node_id: NodeId,
    pub(crate) resolution: Resolution,
    pub(crate) schedule_directives: ScheduleDirectives,
}

pub struct ResolutionBuilder<'w, 's> {
    pub(crate) node: &'s OrcNode,
    pub(crate) commands: Commands<'w, 's>,
    pub(crate) loop_directive: Option<LoopDirective>,
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, (start_orcs, resolve_nodes, process_orcs).chain());
    app.add_message::<ResolveNode>();
}

fn start_orcs(orcs: Query<&mut Orc, Added<Orc>>) -> Result {
    for mut orc in orcs {
        orc.orchestrator.start()?;
    }

    Ok(())
}

fn resolve_nodes(mut messages: MessageReader<ResolveNode>, mut orcs: Query<&mut Orc>) -> Result {
    for &ResolveNode {
        orc_id,
        node_id,
        resolution,
        ref schedule_directives,
    } in messages.read()
    {
        let mut orc = orcs.get_mut(orc_id)?;
        orc.orchestrator.config_schedule(schedule_directives);
        let tx = orc.orchestrator.resolver();
        tx.send((node_id, resolution))?;
    }

    Ok(())
}

fn process_orcs(
    mut commands: Commands,
    orcs: Query<(Entity, &mut Orc)>,
    registry: Res<OrcRegistry>,
) -> Result {
    for (entity, mut orc) in orcs {
        match orc.orchestrator.tick()? {
            ScheduleState::Ended => {
                commands.entity(entity).despawn();
                continue;
            }
            ScheduleState::Restarted => {
                for (&entity, (_, meta)) in orc.entity_map.iter().zip(orc.orchestrator.node_meta())
                {
                    registry.reset(commands.reborrow(), entity, meta)?;
                }
            }
            _ => {}
        }

        for (id, status) in orc.orchestrator.drain_events() {
            let node_entity = orc.entity_map[id];
            let (_, meta) = orc.orchestrator.node_meta()[id];

            match status {
                NodeStatus::Started => {
                    registry.start(commands.reborrow(), node_entity, meta)?;
                }
                NodeStatus::Resolved(Resolution::Finished) => {
                    registry.finish(commands.reborrow(), node_entity, meta)?;
                }
            }
        }
    }

    Ok(())
}

impl OrcNode {
    pub fn resolver<'w, 's>(&'s self, commands: Commands<'w, 's>) -> ResolutionBuilder<'w, 's> {
        ResolutionBuilder {
            node: self,
            commands,
            loop_directive: None,
        }
    }
}

impl<'w, 's> ResolutionBuilder<'w, 's> {
    pub fn cancel_loop(mut self) -> Self {
        self.loop_directive = Some(LoopDirective::Break);
        self
    }

    pub fn finish(mut self) {
        self.commands.write_message(ResolveNode {
            orc_id: self.node.orc_id,
            node_id: self.node.node_id,
            resolution: Resolution::Finished,
            schedule_directives: ScheduleDirectives {
                loop_directive: self.loop_directive,
            },
        });
    }
}
