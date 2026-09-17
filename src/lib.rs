use action_orc::{NodeId, Orchestrator, ReactorError};
use bevy::prelude::*;
use std::fmt;

mod commands;
mod dispatcher;
mod events;
mod schedule;

#[cfg(test)]
mod tests;

pub mod prelude {
    use super::*;

    pub use super::OrcNode;
    pub use action_orc::*;
    pub use commands::*;
    pub use dispatcher::{NodeConstraint, OrcAppExt};
    pub use events::*;
}

#[derive(Component)]
pub(crate) struct Orc {
    pub(crate) orchestrator: Orchestrator,
    pub(crate) entity_map: Vec<Entity>,
}

#[derive(Component)]
pub struct OrcNode {
    pub(crate) orc_id: Entity,
    pub(crate) node_id: NodeId,
}

#[derive(Debug)]
pub enum OrcError {
    NodeMissing(Entity),
    ReactorMissing(Entity),
    TypeIdMissing { type_name: &'static str },
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
                type_name: node_type,
            } => write!(
                f,
                "Type registration mapping is missing in registries for type '{node_type}'."
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

pub struct OrcPlugin;
impl Plugin for OrcPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((dispatcher::plugin, schedule::plugin));
    }
}
