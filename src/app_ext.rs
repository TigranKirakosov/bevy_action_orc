use std::any::TypeId;

use bevy::prelude::*;

use crate::{lifecycle, reactor::ReactorRegistry};

pub trait ActionOrcAppExt {
    fn register_node<T: FromReflect + TypePath + Default + 'static>(&mut self) -> &mut Self;
}

impl ActionOrcAppExt for App {
    fn register_node<T: FromReflect + TypePath + Default + 'static>(&mut self) -> &mut Self {
        self.register_type::<lifecycle::Started<T>>();
        self.register_type::<lifecycle::Resolved<T>>();

        let mut registry = self.world_mut().resource_mut::<ReactorRegistry>();
        registry
            .started
            .insert(TypeId::of::<T>(), TypeId::of::<lifecycle::Started<T>>());
        registry
            .resolved
            .insert(TypeId::of::<T>(), TypeId::of::<lifecycle::Resolved<T>>());
        self
    }
}
