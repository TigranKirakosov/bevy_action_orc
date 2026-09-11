use std::any::TypeId;

use action_orc::Reactor as _Reactor;
use bevy::{prelude::*, utils::TypeIdMap};
use crossbeam::channel::{Receiver, Sender, unbounded};

#[derive(Event)]
pub struct ResolveNode {
    pub(crate) reactor_id: Entity,
    pub(crate) node_id: Entity,
}

pub struct ReactorMessage {
    pub(crate) entity: Entity,
    pub(crate) event: action_orc::Event,
}

#[derive(Resource, Default)]
pub struct ReactorRegistry {
    pub(crate) started: TypeIdMap<TypeId>,
    pub(crate) resolved: TypeIdMap<TypeId>,
}

#[derive(Resource)]
pub struct ReactorChannel {
    pub(crate) tx: Sender<ReactorMessage>,
    pub(crate) rx: Receiver<ReactorMessage>,
}

impl Default for ReactorChannel {
    fn default() -> Self {
        let (tx, rx) = unbounded();
        Self { tx, rx }
    }
}

pub type Reactor = _Reactor<Entity>;

#[derive(Component, Deref, DerefMut)]
pub struct ScheduleReactor {
    #[deref]
    pub(crate) reactor: Reactor,
    pub(crate) events_buffer: Vec<(Entity, action_orc::Event)>,
}

impl ScheduleReactor {
    pub fn new(reactor: Reactor) -> Self {
        Self {
            reactor,
            events_buffer: Vec::new(),
        }
    }
}

#[derive(Component)]
pub struct ScheduleNode {
    pub(crate) reactor_id: Entity,
    pub(crate) type_id: TypeId,
}
