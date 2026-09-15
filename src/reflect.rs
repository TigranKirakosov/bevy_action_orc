use bevy::prelude::*;
use bevy::reflect::TypeRegistry;
use std::any::TypeId;

pub fn reflect_remove_component(
    world: &mut World,
    entity: Entity,
    type_id: TypeId,
    registry: &TypeRegistry,
) {
    if let Some(registration) = registry.get(type_id)
        && let Some(reflect_component) = registration.data::<ReflectComponent>()
        && let Ok(mut entity_mut) = world.get_entity_mut(entity)
    {
        reflect_component.remove(&mut entity_mut);
    }
}

pub fn reflect_insert_default_component(
    world: &mut World,
    entity: Entity,
    type_id: TypeId,
    registry: &TypeRegistry,
) {
    let registration = registry
        .get(type_id)
        .expect("Type not registered in AppTypeRegistry");

    let reflect_component = registration
        .data::<ReflectComponent>()
        .expect("Type lacks #[reflect(Component)]");

    let reflect_default = registration
        .data::<ReflectDefault>()
        .expect("Type lacks #[reflect(Default)]");

    let marker_instance = reflect_default.default();
    if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
        reflect_component.insert(
            &mut entity_mut,
            marker_instance.as_partial_reflect(),
            registry,
        );
    }
}
