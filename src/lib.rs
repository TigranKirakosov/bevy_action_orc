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

    pub use action_orc::{
        AsGraphEntry, AsGraphEntryProxy, Graph, GraphBounds, GraphBuilder, GraphEntry, Tag, orc,
    };
    pub use registry::OrcAppExt;
}

#[derive(Component)]
pub struct Orc {
    pub(crate) reactor: Reactor,
    pub(crate) entity_map: Vec<Entity>,
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
    pub(crate) fn new(reactor: Reactor, entity_map: Vec<Entity>) -> Self {
        Self {
            reactor,
            entity_map,
        }
    }

    pub(crate) fn resolve(
        &mut self,
        node_id: NodeId,
        resolution: Resolution,
    ) -> Result<(), OrcError> {
        self.reactor
            .resolve(node_id, resolution)
            .map_err(OrcError::Reactor)
    }

    pub(crate) fn entity(&self, node_id: NodeId) -> Result<&Entity, OrcError> {
        self.entity_map
            .get(node_id)
            .ok_or(OrcError::NodeEntityMismatch)
    }
}
