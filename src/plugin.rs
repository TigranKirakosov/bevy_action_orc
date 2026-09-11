use action_orc::Event;
use bevy::prelude::*;

use crate::reactor::{
    ReactorChannel, ReactorMessage, ReactorRegistry, ResolveNode, ScheduleNode, ScheduleReactor,
};

pub struct ActionOrcPlugin;
impl Plugin for ActionOrcPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ReactorChannel>();
        app.init_resource::<ReactorRegistry>();
        app.add_systems(Update, (init_reactors, drain_reactor_buffer).chain());
        app.add_observer(resolve_reactor_node);
    }
}

fn init_reactors(reactors: Query<&mut ScheduleReactor, Added<ScheduleReactor>>) {
    for mut reactor in reactors {
        reactor.init();
    }
}

fn resolve_reactor_node(on: On<ResolveNode>, mut reactors: Query<&mut ScheduleReactor>) {
    let mut reactor = reactors.get_mut(on.reactor_id).expect("ReactorId mismatch");
    reactor.resolve(&on.node_id);
}

fn drain_reactor_buffer(world: &mut World) {
    let channel = world.resource::<ReactorChannel>();
    let messages: Vec<_> = channel.rx.try_iter().collect();

    world.resource_scope::<ReactorRegistry, ()>(|world: &mut World, reactor_registry| {
        let app_type_registry = world.resource::<AppTypeRegistry>().clone();
        let type_registry = app_type_registry.read();

        for ReactorMessage { entity, event } in messages {
            let schedule_node = world
                .get::<ScheduleNode>(entity)
                .expect("Missing ScheduleNode component.");

            let target_component_id = (match event {
                Event::Started => reactor_registry.started.get(&schedule_node.type_id),
                Event::Resolved => reactor_registry.resolved.get(&schedule_node.type_id),
            })
            .expect("Component TypeId mistmatch.");

            let registration = type_registry.get(*target_component_id).unwrap();
            let reflect_component = registration.data::<ReflectComponent>().unwrap();
            let reflect_default = registration.data::<ReflectDefault>().unwrap();

            let marker_instance = reflect_default.default();
            let mut entity_mut = world.entity_mut(entity);

            reflect_component.insert(
                &mut entity_mut,
                marker_instance.as_partial_reflect(),
                &type_registry,
            );
        }
    });
}
