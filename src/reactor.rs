use std::any::TypeId;

use action_orc::{NodeId, NodeStatus, Reactor, ReactorError, Resolution};
use bevy::{prelude::*, utils::TypeIdMap};
use crossbeam::channel::{Receiver, Sender, unbounded};

#[derive(Event)]
pub struct ResolveNode {
    pub(crate) reactor_id: Entity,
    pub(crate) node_id: NodeId,
}

#[derive(Resource, Default)]
pub struct OrcTypeRegistry {
    pub(crate) started: TypeIdMap<TypeId>,
    pub(crate) finished: TypeIdMap<TypeId>,
}

#[derive(Resource)]
pub struct OrcChannel {
    pub(crate) tx: Sender<OrcMessage>,
    pub(crate) rx: Receiver<OrcMessage>,
}

pub struct OrcMessage {
    pub(crate) node_entity: Entity,
    pub(crate) status: NodeStatus,
}

impl Default for OrcChannel {
    fn default() -> Self {
        let (tx, rx) = unbounded();
        Self { tx, rx }
    }
}

#[derive(Debug)]
pub enum OrcError {
    NodeEntityMismatch,
    Reactor(ReactorError),
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

impl Orc {
    pub fn new(reactor: Reactor, entity_map: Vec<Entity>) -> Self {
        Self {
            reactor,
            entity_map,
        }
    }

    pub fn resolve(&mut self, node_id: NodeId, resolution: Resolution) -> Result<(), OrcError> {
        self.reactor
            .resolve(node_id, resolution)
            .map_err(OrcError::Reactor)
    }

    pub fn entity(&self, node_id: NodeId) -> Result<&Entity, OrcError> {
        self.entity_map
            .get(node_id)
            .ok_or(OrcError::NodeEntityMismatch)
    }
}
