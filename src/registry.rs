use bevy::{prelude::*, utils::TypeIdMap};
use std::any::TypeId;

use super::lifecycle::*;

#[derive(Resource, Default)]
pub struct OrcTypeRegistry {
    pub(crate) started: TypeIdMap<TypeId>,
    pub(crate) finished: TypeIdMap<TypeId>,
}

pub trait OrcAppExt {
    fn register_node<T: FromReflect + TypePath + Default + 'static>(&mut self) -> &mut Self;
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<OrcTypeRegistry>();
}

impl OrcAppExt for App {
    fn register_node<T: FromReflect + TypePath + Default + 'static>(&mut self) -> &mut Self {
        self.register_type::<Started<T>>();
        self.register_type::<Finished<T>>();

        let mut registry = self.world_mut().resource_mut::<OrcTypeRegistry>();
        registry
            .started
            .insert(TypeId::of::<T>(), TypeId::of::<Started<T>>());
        registry
            .finished
            .insert(TypeId::of::<T>(), TypeId::of::<Finished<T>>());
        self
    }
}
