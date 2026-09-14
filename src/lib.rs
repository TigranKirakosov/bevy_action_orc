use action_orc::{NodeId, Reactor, ReactorError, Resolution};
use bevy::prelude::*;
use std::any::TypeId;

mod commands;
mod lifecycle;
mod registry;
mod schedule;

#[cfg(test)]
mod tests;

pub mod prelude {
    use super::*;

    pub use super::{OrcNode, ResolveNode};
    pub use action_orc::*;
    pub use commands::*;
    pub use lifecycle::*;
    pub use registry::{NodeConstraint, OrcAppExt};
}

#[derive(Component, Deref, DerefMut)]
pub struct Orc {
    #[deref]
    pub(crate) reactor: Reactor,
    pub(crate) entity_map: Vec<Entity>,
    pub(crate) loop_schedule: bool,
    pub(crate) active_node_count: usize,
}

#[derive(Component)]
pub struct OrcNode {
    pub(crate) reactor_id: Entity,
    pub(crate) node_id: NodeId,
    pub(crate) type_id: TypeId,
}

#[derive(Debug)]
pub enum OrcError {
    NodeEntityMismatch,
    Reactor(ReactorError),
}

#[derive(Event)]
pub struct ResolveNode {
    pub(crate) reactor_id: Entity,
    pub(crate) node_id: NodeId,
    pub(crate) resolution: Resolution,
}

pub struct OrcPlugin;
impl Plugin for OrcPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((registry::plugin, schedule::plugin));
    }
}

impl Orc {
    pub(crate) fn new(reactor: Reactor, entity_map: Vec<Entity>, loop_schedule: bool) -> Self {
        Self {
            reactor,
            active_node_count: entity_map.len(),
            loop_schedule,
            entity_map,
        }
    }

    pub(crate) fn entity(&self, node_id: NodeId) -> Result<&Entity, OrcError> {
        self.entity_map
            .get(node_id)
            .ok_or(OrcError::NodeEntityMismatch)
    }
}

impl OrcNode {
    pub fn finished(&self) -> ResolveNode {
        ResolveNode {
            reactor_id: self.reactor_id,
            node_id: self.node_id,
            resolution: Resolution::Finished,
        }
    }
}
