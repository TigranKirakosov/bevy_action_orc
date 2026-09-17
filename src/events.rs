use bevy::prelude::*;
use std::marker::PhantomData;

use crate::prelude::NodeConstraint;

#[derive(Event, Deref)]
pub struct NodeStarted<T: NodeConstraint> {
    #[deref]
    pub(crate) entity: Entity,
    pub(crate) _marker: PhantomData<T>,
}

#[derive(Event, Deref)]
pub struct NodeFinished<T: NodeConstraint> {
    #[deref]
    pub(crate) entity: Entity,
    pub(crate) _marker: PhantomData<T>,
}

#[derive(Event, Deref)]
pub struct NodeReset<T: NodeConstraint> {
    #[deref]
    pub(crate) entity: Entity,
    pub(crate) _marker: PhantomData<T>,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct Active<T: FromReflect + TypePath + Default>(#[reflect(ignore)] PhantomData<T>);

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct Finished<T: FromReflect + TypePath + Default>(#[reflect(ignore)] PhantomData<T>);
