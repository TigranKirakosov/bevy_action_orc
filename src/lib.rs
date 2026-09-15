use action_orc::{Meta, NodeId, Reactor, ReactorError, Resolution};
use bevy::prelude::*;
use std::fmt;

mod commands;
mod lifecycle;
mod reflect;
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
    pub(crate) pending_node_statuses: i32,
}

#[derive(Component)]
pub struct OrcNode {
    pub(crate) reactor_id: Entity,
    pub(crate) node_id: NodeId,
    pub(crate) meta: Meta,
}

#[derive(Debug)]
pub enum OrcError {
    NodeMissing(Entity),
    ReactorMissing(Entity),
    TypeIdMissing {
        status: &'static str,
        type_name: &'static str,
    },
    NodeEntityMismatch,
    Reactor(ReactorError),
}

impl fmt::Display for OrcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrcError::NodeMissing(e) => write!(f, "OrcNode component is missing on entity {e:?}."),
            OrcError::ReactorMissing(e) => {
                write!(f, "Orc (Reactor) component is missing on entity {e:?}.")
            }
            OrcError::TypeIdMissing {
                status,
                type_name: node_type,
            } => write!(
                f,
                "Type registration mapping is missing in registries for type '{status}<{node_type}>'."
            ),
            OrcError::NodeEntityMismatch => {
                write!(f, "Mismatched NodeId within Reactor's entity map.")
            }
            OrcError::Reactor(err) => write!(f, "Reactor execution failed: {err}"),
        }
    }
}

impl std::error::Error for OrcError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            OrcError::Reactor(err) => Some(err),
            _ => None,
        }
    }
}

impl From<ReactorError> for OrcError {
    fn from(err: ReactorError) -> Self {
        OrcError::Reactor(err)
    }
}

#[derive(Default)]
pub struct GraphConfig {
    pub loop_schedule: bool,
}

pub struct ResolveOptions {
    pub loop_schedule: bool,
}

#[derive(Event)]
pub struct ResolveNode {
    pub(crate) reactor_id: Entity,
    pub(crate) node_id: NodeId,
    pub(crate) resolution: Resolution,
    pub(crate) options: ResolveOptions,
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
            pending_node_statuses: entity_map.len() as i32,
            loop_schedule,
            entity_map,
        }
    }

    pub(crate) fn apply_pending_statuses_delta(&mut self, delta: i32) {
        self.pending_node_statuses += delta;
    }

    pub(crate) fn is_schedule_complete(&self) -> bool {
        self.pending_node_statuses == 0
    }

    pub(crate) fn entity(&self, node_id: NodeId) -> Result<&Entity, OrcError> {
        self.entity_map
            .get(node_id)
            .ok_or(OrcError::NodeEntityMismatch)
    }
}

impl OrcNode {
    pub fn finished(&self, options: ResolveOptions) -> ResolveNode {
        ResolveNode {
            reactor_id: self.reactor_id,
            node_id: self.node_id,
            resolution: Resolution::Finished,
            options,
        }
    }
}
