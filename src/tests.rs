use action_orc::*;

use crate::{
    app_ext::ActionOrcAppExt,
    commands_ext::ActionOrcCommandsExt,
    lifecycle::{Resolved, Started},
    plugin::ActionOrcPlugin,
    reactor::{ResolveNode, ScheduleNode},
};
use bevy::prelude::*;
use local_macros::*;

#[derive(Resource, Default, Deref, DerefMut)]
struct Log(Vec<(&'static str, &'static str)>);

#[test]
fn integration_example() {
    let mut app = App::new();
    app.add_plugins(ActionOrcPlugin);
    app.init_resource::<Log>();

    // Expanded `register_nodes!`
    #[derive(Reflect, Default)]
    struct A;
    #[derive(Reflect, Default)]
    struct B;
    #[derive(Reflect, Default)]
    struct C;

    app.register_node::<A>();
    app.register_node::<B>();
    app.register_node::<C>();

    // Expanded `mock_systems!`
    app.add_systems(
        Update,
        (
            (on_started::<A>, on_resolved::<A>),
            (on_started::<B>, on_resolved::<B>),
            (on_started::<C>, on_resolved::<C>),
        )
            .chain(),
    );

    let graph = orc!(
        A -> B -> C;
    );

    app.world_mut().commands().launch_reactor(graph);

    tick(&mut app, 2); // ECS wind-up
    tick(&mut app, 4);

    let log = app.world().resource::<Log>().as_slice();
    assert_eq! { log,
        &[
        /* Wave 1 */("Started", "A"),

        /* Wave 2 */("Resolved", "A"),
                    ("Started", "B"),

        /* Wave 3 */("Resolved", "B"),
                    ("Started", "C"),

        /* Wave 4 */("Resolved", "C"),
        ]
    };
}

#[test]
fn parallel_concurrency() {
    let mut app = App::new();
    app.add_plugins(ActionOrcPlugin);
    app.init_resource::<Log>();

    register_nodes!(app, A, B, C);
    mock_systems!(app, A, B, C);

    let graph = orc!(
        (A | B) -> C;
    );
    app.world_mut().commands().launch_reactor(graph);

    tick(&mut app, 2);

    let log = app.world().resource::<Log>().as_slice();
    assert_eq!(log.len(), 2);
    assert!(log.contains(&("Started", "A")));
    assert!(log.contains(&("Started", "B")));

    tick(&mut app, 2);

    let log = app.world().resource::<Log>().as_slice();
    assert_eq!(log.len(), 5);
    assert!(log.contains(&("Resolved", "A")));
    assert!(log.contains(&("Resolved", "B")));
    assert_eq!(*log.last().unwrap(), ("Started", "C"));

    tick(&mut app, 2);

    let log = app.world().resource::<Log>().as_slice();
    assert_eq!(log.len(), 6);
    assert_eq!(*log.last().unwrap(), ("Resolved", "C"));
}

#[test]
fn embedded_linear_composition() {
    let mut app = App::new();
    app.add_plugins(ActionOrcPlugin);
    app.init_resource::<Log>();

    register_nodes!(app, Enter, X, Y, Exit);
    mock_systems!(app, Enter, X, Y, Exit);

    let sub = orc!(
        X -> Y;
    );

    let graph = orc!(
        Enter -> #[sub] -> Exit;
    );

    app.world_mut().commands().launch_reactor(graph);

    tick(&mut app, 2); // ECS wind-up
    tick(&mut app, 5);

    let log = app.world().resource::<Log>().as_slice();
    assert_eq! { log,
        &[
        /* Wave 1 */("Started", "Enter"),

        /* Wave 2 */("Resolved", "Enter"),
                    ("Started", "X"),

        /* Wave 3 */("Resolved", "X"),
                    ("Started", "Y"),

        /* Wave 4 */("Resolved", "Y"),
                    ("Started", "Exit"),

        /* Wave 5 */("Resolved", "Exit"),
        ]
    };
}

#[test]
fn embedded_parallel_composition() {
    let mut app = App::new();
    app.add_plugins(ActionOrcPlugin);
    app.init_resource::<Log>();

    register_nodes!(app, Enter, ConcurrentTask, X, Y, Exit);
    mock_systems!(app, Enter, ConcurrentTask, X, Y, Exit);

    let sub = orc!(
        X -> Y;
    );

    let graph = orc!(
        Enter -> ( ConcurrentTask | #[sub] ) -> Exit;
    );

    app.world_mut().commands().launch_reactor(graph);

    tick(&mut app, 2); // ECS wind-up
    tick(&mut app, 5);

    let log = app.world().resource::<Log>().as_slice();
    assert_eq! { log,
        &[
        /* Wave 1 */("Started", "Enter"),

        /* Wave 2 */("Resolved", "Enter"),
                    ("Started", "ConcurrentTask"),
                    ("Started", "X"),

        /* Wave 3 */("Resolved", "ConcurrentTask"),
                    ("Resolved", "X"),
                    ("Started", "Y"),

        /* Wave 4 */("Resolved", "Y"),
                    ("Started", "Exit"),

        /* Wave 5 */("Resolved", "Exit"),
        ]
    };
}

#[test]
fn embedded_back_to_back_composition() {
    let mut app = App::new();
    app.add_plugins(ActionOrcPlugin);
    app.init_resource::<Log>();

    register_nodes!(app, Enter, A, B, X, Y, Exit);
    mock_systems!(app, Enter, A, B, X, Y, Exit);

    let sub_a = orc!(A -> B;);
    let sub_b = orc!(X -> Y;);
    let graph = orc!(Enter -> #[sub_a] -> #[sub_b] -> Exit;);

    app.world_mut().commands().launch_reactor(graph);

    tick(&mut app, 2); // ECS wind-up
    tick(&mut app, 7);
    let log = app.world().resource::<Log>().as_slice();
    assert_eq! { log,
        &[
        /* Wave 1 */("Started", "Enter"),

        /* Wave 2 */("Resolved", "Enter"),
                    ("Started", "A"),

        /* Wave 3 */("Resolved", "A"),
                    ("Started", "B"),

        /* Wave 4 */("Resolved", "B"),
                    ("Started", "X"),

        /* Wave 5 */("Resolved", "X"),
                    ("Started", "Y"),

        /* Wave 6 */("Resolved", "Y"),
                    ("Started", "Exit"),

        /* Wave 7 */("Resolved", "Exit"),
        ]
    };
}

#[test]
fn concept_warchief_campaign() {
    let mut app = App::new();
    app.add_plugins(ActionOrcPlugin);
    app.init_resource::<Log>();

    register_nodes! {
        app,
        BuildCamp,
        GatherResources,
        Defend,
        RequestReinforcements,
        PrepareCampaign,
        BuildWarmachines,
        TrainGrunts,
        AssembleArmy,
        LaunchCampaign,
        CelebrateVictory,
        X, Y,
    };

    mock_systems! {
        app,
        BuildCamp,
        GatherResources,
        Defend,
        RequestReinforcements,
        PrepareCampaign,
        BuildWarmachines,
        TrainGrunts,
        AssembleArmy,
        LaunchCampaign,
        CelebrateVictory,
        X, Y,
    };

    fn warchief_campaign(reinforce: &Graph) -> Graph {
        orc! {
            // Define first timeline
            BuildCamp -> (
                gather: GatherResources,
                // make #[reinforce] dependant on upstream nodes
                (Defend | RequestReinforcements) -> #[reinforce],
            );

            // Define second parallel timline, linked with the first one by [gather] and #[reinforce] nodes
            prepare: PrepareCampaign -> (
                [gather] -> BuildWarmachines
                | TrainGrunts
                | #[reinforce] // make this timline dependant on #[reinforce] aswell
            );

            // Declare exit node
            victory: CelebrateVictory;

            // Define path to exit
            [prepare] -> AssembleArmy -> LaunchCampaign -> [victory];
        }
    }

    let reinforce = orc!(X -> Y;);

    let graph = warchief_campaign(&reinforce);
    app.world_mut().commands().launch_reactor(graph);

    // No ECS wind-up: some action resolved at the same frame
    tick(&mut app, 8);

    let log = app.world().resource::<Log>().as_slice();
    assert_eq! {
        log, &[
    /* Wave 1 */("Started", "BuildCamp"),
                ("Started", "PrepareCampaign"),

    /* Wave 2 */("Resolved", "BuildCamp"),
                ("Started", "GatherResources"),

    /* Wave 3 */("Resolved", "PrepareCampaign"),
                ("Started", "TrainGrunts"),
                ("Started", "AssembleArmy"),

    /* Wave 4 */("Resolved", "GatherResources"),
                ("Started", "Defend"),
                ("Started", "RequestReinforcements"),
                ("Started", "BuildWarmachines"),

    /* Wave 5 */("Resolved", "TrainGrunts"),
                ("Resolved", "AssembleArmy"),
                ("Started", "LaunchCampaign"),

    /* Wave 6 */("Resolved", "Defend"),
                ("Resolved", "RequestReinforcements"),
                ("Resolved", "BuildWarmachines"),
                ("Resolved", "LaunchCampaign"),
                ("Started", "CelebrateVictory"),
                ("Started", "X"),

    /* Wave 7 */("Resolved", "CelebrateVictory"),
                ("Resolved", "X"),
                ("Started", "Y"),

    /* Wave 8 */("Resolved", "Y"),
            ]
        };
}

fn on_started<T: FromReflect + TypePath + Default>(
    started: Query<(Entity, &ScheduleNode), Added<Started<T>>>,
    mut commands: Commands,
    mut queue: ResMut<Log>,
) {
    for (entity, node) in started {
        let type_str = get_name::<T>();
        queue.push(("Started", type_str));

        commands.trigger(ResolveNode {
            reactor_id: node.reactor_id,
            node_id: entity,
        });
    }
}

fn on_resolved<T: FromReflect + TypePath + Default>(
    resolved: Query<(), Added<Resolved<T>>>,
    mut queue: ResMut<Log>,
) {
    for _ in resolved {
        let type_str = get_name::<T>();
        queue.push(("Resolved", type_str));
    }
}

fn tick(app: &mut App, times: usize) {
    for _ in 0..times {
        app.update();
    }
}

fn get_name<T>() -> &'static str {
    let full_name = std::any::type_name::<T>();
    let short_name = full_name.split("::").last();
    short_name.unwrap_or(full_name)
}

mod local_macros {
    macro_rules! register_nodes {
        ($app:expr, $($node:ident),* $(,)?) => {
            $(
                #[derive(Reflect, Default)]
                struct $node;
                $app.register_node::<$node>();
            )*
        };
    }
    pub(crate) use register_nodes;

    macro_rules! mock_systems {
        ($app:expr, $($node:ident),* $(,)?) => {
            $app.add_systems(Update, (
                $(
                    (on_started::<$node>, on_resolved::<$node>),
                )*
            ).chain());
        };
    }
    pub(crate) use mock_systems;
}
