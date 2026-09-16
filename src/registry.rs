use action_orc::Meta;
use bevy::{prelude::*, utils::TypeIdMap};
use std::{any::TypeId, marker::PhantomData};

use crate::OrcError;

use super::lifecycle::*;

type TriggerFn = fn(Commands, Entity);

#[derive(Resource, Default)]
pub(crate) struct OrcRegistry {
    pub(crate) trigger_start: TypeIdMap<TriggerFn>,
    pub(crate) trigger_finish: TypeIdMap<TriggerFn>,
    pub(crate) trigger_reset: TypeIdMap<TriggerFn>,
}

pub trait NodeConstraint: FromReflect + TypePath + Default + 'static {}
impl<T: FromReflect + TypePath + Default + 'static> NodeConstraint for T {}

pub trait OrcAppExt {
    fn register_node<T: NodeConstraint>(&mut self) -> &mut Self;
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<OrcRegistry>();
}

impl OrcAppExt for App {
    fn register_node<T: NodeConstraint>(&mut self) -> &mut Self {
        let type_id = TypeId::of::<T>();

        let mut registry = self.world_mut().resource_mut::<OrcRegistry>();
        registry
            .trigger_start
            .insert(type_id, |mut commands, entity| {
                commands.trigger(NodeStarted::<T> {
                    entity,
                    _marker: PhantomData,
                });
            });

        registry
            .trigger_finish
            .insert(type_id, |mut commands, entity| {
                commands.trigger(NodeFinished::<T> {
                    entity,
                    _marker: PhantomData,
                });
            });

        registry
            .trigger_reset
            .insert(type_id, |mut commands, entity| {
                commands.trigger(NodeReset::<T> {
                    entity,
                    _marker: PhantomData,
                });
            });

        self.add_observer(|on: On<NodeStarted<T>>, mut commands: Commands| {
            commands.entity(on.entity).insert(Active::<T>::default());
        });

        self.add_observer(|on: On<NodeFinished<T>>, mut commands: Commands| {
            let entity = on.entity;
            commands.entity(entity).remove::<Active<T>>();
            commands.entity(entity).insert(Finished::<T>::default());
        });

        self.add_observer(|on: On<NodeReset<T>>, mut commands: Commands| {
            let entity = on.entity;
            commands.entity(entity).remove::<(Active<T>, Finished<T>)>();
        });
        self
    }
}
impl OrcRegistry {
    pub(crate) fn start(&self, commands: Commands, entity: Entity, meta: &Meta) -> Result {
        let (type_id, type_name) = (meta.type_id(), meta.type_name());
        let trigger = self
            .trigger_start
            .get(type_id)
            .ok_or(OrcError::TypeIdMissing { type_name })?;
        trigger(commands, entity);
        Ok(())
    }

    pub(crate) fn finish(&self, commands: Commands, entity: Entity, meta: &Meta) -> Result {
        let (type_id, type_name) = (meta.type_id(), meta.type_name());
        let trigger = self
            .trigger_finish
            .get(type_id)
            .ok_or(OrcError::TypeIdMissing { type_name })?;
        trigger(commands, entity);
        Ok(())
    }

    pub(crate) fn reset(&self, commands: Commands, entity: Entity, meta: &Meta) -> Result {
        let (type_id, type_name) = (meta.type_id(), meta.type_name());
        let trigger = self
            .trigger_reset
            .get(type_id)
            .ok_or(OrcError::TypeIdMissing { type_name })?;
        trigger(commands, entity);
        Ok(())
    }
}
