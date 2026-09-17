use action_orc::{NodeCommand, NodeId, NodeStatus, Resolution, ScheduleDirective, State};
use bevy::prelude::*;

use crate::{Orc, OrcNode, dispatcher::Dispatcher};

#[derive(Message, Clone)]
pub(crate) struct ResolveNode {
    pub(crate) orc_id: Entity,
    pub(crate) node_id: NodeId,
    pub(crate) command: NodeCommand,
    pub(crate) loop_schedule: Option<bool>,
}

pub struct ResolutionBuilder<'w, 's> {
    pub(crate) node: &'s OrcNode,
    pub(crate) commands: Commands<'w, 's>,
    pub(crate) loop_schedule: Option<bool>,
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
        command,
        loop_schedule,
        ..
    } in messages.read()
    {
        let mut orc = orcs.get_mut(orc_id)?;
        if let Some(loop_schedule) = loop_schedule {
            orc.orchestrator.config_schedule_mut().loop_schedule = loop_schedule;
        }

        let tx = orc.orchestrator.resolver();
        tx.send(command)?;
    }

    Ok(())
}

fn process_orcs(
    mut commands: Commands,
    orcs: Query<(Entity, &mut Orc)>,
    registry: Res<Dispatcher>,
) -> Result {
    for (entity, mut orc) in orcs {
        match orc.orchestrator.tick()? {
            State::Ended => {
                commands.entity(entity).despawn();
                continue;
            }
            State::Restarted => {
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
                NodeStatus::Resolved(resoulution) => match resoulution {
                    Resolution::Finished => {
                        registry.finish(commands.reborrow(), node_entity, meta)?;
                    }
                    Resolution::Reset => {
                        registry.reset(commands.reborrow(), node_entity, meta)?;
                    }
                },
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
            loop_schedule: None,
        }
    }
}

impl<'w, 's> ResolutionBuilder<'w, 's> {
    pub fn cancel_loop(mut self) -> Self {
        self.loop_schedule = Some(false);
        self
    }

    pub fn finish(mut self) {
        self.commands.write_message(ResolveNode {
            orc_id: self.node.orc_id,
            node_id: self.node.node_id,
            command: NodeCommand {
                schedule_directive: ScheduleDirective::Advance {
                    pivot: self.node.node_id,
                },
            },
            loop_schedule: self.loop_schedule,
        });
    }
}
