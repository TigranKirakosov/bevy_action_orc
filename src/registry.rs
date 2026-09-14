use bevy::{prelude::*, utils::TypeIdMap};
use std::any::TypeId;

use super::lifecycle::*;

#[derive(Resource, Default)]
pub(crate) struct OrcTypeRegistry {
    pub(crate) active: TypeIdMap<TypeId>,
    pub(crate) finished: TypeIdMap<TypeId>,
}

pub trait NodeConstraint: FromReflect + TypePath + Default + 'static {}
impl<T: FromReflect + TypePath + Default + 'static> NodeConstraint for T {}

pub trait OrcAppExt {
    fn register_node<T: NodeConstraint>(&mut self) -> &mut Self;
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<OrcTypeRegistry>();
}

impl OrcAppExt for App {
    fn register_node<T: NodeConstraint>(&mut self) -> &mut Self {
        self.register_type::<Active<T>>();
        self.register_type::<Finished<T>>();

        let type_id = TypeId::of::<T>();
        let started_type = TypeId::of::<Active<T>>();
        let finished_type = TypeId::of::<Finished<T>>();

        let mut orc_registry = self.world_mut().resource_mut::<OrcTypeRegistry>();
        orc_registry.active.insert(type_id, started_type);
        orc_registry.finished.insert(type_id, finished_type);
        self
    }
}
