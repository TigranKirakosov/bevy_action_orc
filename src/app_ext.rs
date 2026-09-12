use std::any::TypeId;

use bevy::prelude::*;

use crate::{lifecycle, reactor::OrcTypeRegistry};

pub trait ActionOrcAppExt {
    fn register_node<T: FromReflect + TypePath + Default + 'static>(&mut self) -> &mut Self;
}

impl ActionOrcAppExt for App {
    fn register_node<T: FromReflect + TypePath + Default + 'static>(&mut self) -> &mut Self {
        self.register_type::<lifecycle::Started<T>>();
        self.register_type::<lifecycle::Finished<T>>();

        let mut registry = self.world_mut().resource_mut::<OrcTypeRegistry>();
        registry
            .started
            .insert(TypeId::of::<T>(), TypeId::of::<lifecycle::Started<T>>());
        registry
            .finished
            .insert(TypeId::of::<T>(), TypeId::of::<lifecycle::Finished<T>>());
        self
    }
}
