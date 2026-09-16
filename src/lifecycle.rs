use bevy::prelude::*;
use std::marker::PhantomData;

use crate::prelude::NodeConstraint;

#[derive(Event)]
pub(crate) struct NodeStarted<T: NodeConstraint> {
    pub(crate) entity: Entity,
    pub(crate) _marker: PhantomData<T>,
}

#[derive(Event)]
pub(crate) struct NodeFinished<T: NodeConstraint> {
    pub(crate) entity: Entity,
    pub(crate) _marker: PhantomData<T>,
}

#[derive(EntityEvent)]
pub(crate) struct NodeReset<T: NodeConstraint> {
    #[event_target]
    pub(crate) entity: Entity,
    pub(crate) _marker: PhantomData<T>,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct Active<T: FromReflect + TypePath + Default>(#[reflect(ignore)] PhantomData<T>);

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct Finished<T: FromReflect + TypePath + Default>(#[reflect(ignore)] PhantomData<T>);
