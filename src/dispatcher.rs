use action_orc::Meta;
use bevy::{platform::collections::HashMap, prelude::*, utils::TypeIdMap};
use std::{any::TypeId, marker::PhantomData};

use crate::OrcError;

use super::events::*;

pub trait NodeConstraint: FromReflect + TypePath + Default + 'static {}
impl<T: FromReflect + TypePath + Default + 'static> NodeConstraint for T {}

type TypedFn = fn(Commands, Entity);

#[derive(PartialEq, Eq, Hash)]
pub(crate) enum Event {
    Start,
    Finish,
    Reset,
}

#[derive(Resource, Default)]
pub(crate) struct Dispatcher {
    pub(crate) events: HashMap<Event, TypeIdMap<TypedFn>>,
}

pub trait OrcAppExt {
    fn register_node<T: NodeConstraint>(&mut self) -> &mut Self;
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Dispatcher>();
}

impl OrcAppExt for App {
    fn register_node<T: NodeConstraint>(&mut self) -> &mut Self {
        let type_id = TypeId::of::<T>();
        let mut dispatcher = self.world_mut().resource_mut::<Dispatcher>();

        dispatcher.events.entry(Event::Start).or_default().insert(
            type_id,
            |mut commands, entity| {
                commands.trigger(NodeStarted::<T> {
                    entity,
                    _marker: PhantomData,
                });
            },
        );

        dispatcher.events.entry(Event::Finish).or_default().insert(
            type_id,
            |mut commands, entity| {
                commands.trigger(NodeFinished::<T> {
                    entity,
                    _marker: PhantomData,
                });
            },
        );

        dispatcher.events.entry(Event::Reset).or_default().insert(
            type_id,
            |mut commands, entity| {
                commands.trigger(NodeReset::<T> {
                    entity,
                    _marker: PhantomData,
                });
            },
        );

        self.add_observer(|on: On<NodeStarted<T>>, mut commands: Commands| {
            commands.entity(on.entity()).insert(Active::<T>::default());
        });

        self.add_observer(|on: On<NodeFinished<T>>, mut commands: Commands| {
            let entity = on.entity();
            commands.entity(entity).remove::<Active<T>>();
            commands.entity(entity).insert(Finished::<T>::default());
        });

        self.add_observer(|on: On<NodeReset<T>>, mut commands: Commands| {
            let entity = on.entity();
            commands.entity(entity).remove::<(Active<T>, Finished<T>)>();
        });

        self
    }
}

impl Dispatcher {
    pub(crate) fn start(&self, commands: Commands, entity: Entity, meta: &Meta) -> Result {
        self.typed_fn(Event::Start, meta)?(commands, entity);
        Ok(())
    }

    pub(crate) fn finish(&self, commands: Commands, entity: Entity, meta: &Meta) -> Result {
        self.typed_fn(Event::Finish, meta)?(commands, entity);
        Ok(())
    }

    pub(crate) fn reset(&self, commands: Commands, entity: Entity, meta: &Meta) -> Result {
        self.typed_fn(Event::Reset, meta)?(commands, entity);
        Ok(())
    }

    fn typed_fn(&self, command: Event, meta: &Meta) -> Result<&TypedFn, OrcError> {
        let (type_id, type_name) = (meta.type_id(), meta.type_name());
        self.events
            .get(&command)
            .and_then(|m| m.get(type_id))
            .ok_or(OrcError::TypeIdMissing { type_name })
    }
}
