use action_orc::*;

use crate::{
    OrcNode, OrcPlugin, ResolveNode,
    commands::OrcCommandsExt,
    lifecycle::{Finished, Started},
    registry::OrcAppExt,
};
use bevy::prelude::*;
use local_macros::*;

#[derive(Resource, Default, Deref, DerefMut)]
struct Log(Vec<(&'static str, &'static str)>);

#[test]
fn integration_example() {
    let mut app = App::new();
    app.add_plugins(OrcPlugin);
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
            (on_started::<A>, on_finished::<A>),
            (on_started::<B>, on_finished::<B>),
            (on_started::<C>, on_finished::<C>),
        )
            .chain(),
    );

    let graph = orc!(
        A -> B -> C;
    );

    app.world_mut().commands().queue_graph(graph);

    tick(&mut app, 2); // ECS wind-up
    tick(&mut app, 4);

    let log = app.world().resource::<Log>().as_slice();
    assert_eq! { log,
        &[
        /* Wave 1 */("Started", "A"),

        /* Wave 2 */("Finished", "A"),
                    ("Started", "B"),

        /* Wave 3 */("Finished", "B"),
                    ("Started", "C"),

        /* Wave 4 */("Finished", "C"),
        ]
    };
}

#[test]
fn parallel_concurrency() {
    let mut app = App::new();
    app.add_plugins(OrcPlugin);
    app.init_resource::<Log>();

    register_nodes!(app, A, B, C);
    mock_systems!(app, A, B, C);

    let graph = orc!(
        (A | B) -> C;
    );
    app.world_mut().commands().queue_graph(graph);

    tick(&mut app, 2);

    let log = app.world().resource::<Log>().as_slice();
    assert_eq!(log.len(), 2);
    assert!(log.contains(&("Started", "A")));
    assert!(log.contains(&("Started", "B")));

    tick(&mut app, 2);

    let log = app.world().resource::<Log>().as_slice();
    assert_eq!(log.len(), 5);
    assert!(log.contains(&("Finished", "A")));
    assert!(log.contains(&("Finished", "B")));
    assert_eq!(*log.last().unwrap(), ("Started", "C"));

    tick(&mut app, 2);

    let log = app.world().resource::<Log>().as_slice();
    assert_eq!(log.len(), 6);
    assert_eq!(*log.last().unwrap(), ("Finished", "C"));
}

#[test]
fn embedded_linear_composition() {
    let mut app = App::new();
    app.add_plugins(OrcPlugin);
    app.init_resource::<Log>();

    register_nodes!(app, Enter, X, Y, Exit);
    mock_systems!(app, Enter, X, Y, Exit);

    let sub = orc!(
        X -> Y;
    );

    let graph = orc!(
        Enter -> @sub -> Exit;
    );

    app.world_mut().commands().queue_graph(graph);

    tick(&mut app, 2); // ECS wind-up
    tick(&mut app, 5);

    let log = app.world().resource::<Log>().as_slice();
    assert_eq! { log,
        &[
        /* Wave 1 */("Started", "Enter"),

        /* Wave 2 */("Finished", "Enter"),
                    ("Started", "X"),

        /* Wave 3 */("Finished", "X"),
                    ("Started", "Y"),

        /* Wave 4 */("Finished", "Y"),
                    ("Started", "Exit"),

        /* Wave 5 */("Finished", "Exit"),
        ]
    };
}

#[test]
fn embedded_parallel_composition() {
    let mut app = App::new();
    app.add_plugins(OrcPlugin);
    app.init_resource::<Log>();

    register_nodes!(app, Enter, ConcurrentTask, X, Y, Exit);
    mock_systems!(app, Enter, ConcurrentTask, X, Y, Exit);

    let sub = orc!(
        X -> Y;
    );

    let graph = orc!(
        Enter -> ( ConcurrentTask | @sub ) -> Exit;
    );

    app.world_mut().commands().queue_graph(graph);

    tick(&mut app, 2); // ECS wind-up
    tick(&mut app, 5);

    let log = app.world().resource::<Log>().as_slice();
    assert_eq! { log,
        &[
        /* Wave 1 */("Started", "Enter"),

        /* Wave 2 */("Finished", "Enter"),
                    ("Started", "ConcurrentTask"),
                    ("Started", "X"),

        /* Wave 3 */("Finished", "ConcurrentTask"),
                    ("Finished", "X"),
                    ("Started", "Y"),

        /* Wave 4 */("Finished", "Y"),
                    ("Started", "Exit"),

        /* Wave 5 */("Finished", "Exit"),
        ]
    };
}

#[test]
fn embedded_back_to_back_composition() {
    let mut app = App::new();
    app.add_plugins(OrcPlugin);
    app.init_resource::<Log>();

    register_nodes!(app, Enter, A, B, X, Y, Exit);
    mock_systems!(app, Enter, A, B, X, Y, Exit);

    let sub_a = orc!(A -> B;);
    let sub_b = orc!(X -> Y;);
    let graph = orc!(Enter -> @sub_a -> @sub_b -> Exit;);

    app.world_mut().commands().queue_graph(graph);

    tick(&mut app, 2); // ECS wind-up
    tick(&mut app, 7);
    let log = app.world().resource::<Log>().as_slice();
    assert_eq! { log,
        &[
        /* Wave 1 */("Started", "Enter"),

        /* Wave 2 */("Finished", "Enter"),
                    ("Started", "A"),

        /* Wave 3 */("Finished", "A"),
                    ("Started", "B"),

        /* Wave 4 */("Finished", "B"),
                    ("Started", "X"),

        /* Wave 5 */("Finished", "X"),
                    ("Started", "Y"),

        /* Wave 6 */("Finished", "Y"),
                    ("Started", "Exit"),

        /* Wave 7 */("Finished", "Exit"),
        ]
    };
}

#[test]
fn warchief_campaign() {
    let mut app = App::new();
    app.add_plugins(OrcPlugin);
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
            BuildCamp -> (
                gather: GatherResources,
                (Defend | RequestReinforcements) -> @reinforce,
            );

            prepare: PrepareCampaign -> (
                [gather] -> BuildWarmachines
                | TrainGrunts
                | @reinforce
            );

            victory: CelebrateVictory;

            [prepare] -> AssembleArmy -> LaunchCampaign -> [victory];
        }
    }

    let reinforce = orc!(X -> Y;);

    let graph = warchief_campaign(&reinforce);
    app.world_mut().commands().queue_graph(graph);

    // No ECS wind-up: some action resolved at the same frame
    tick(&mut app, 8);

    let log = app.world().resource::<Log>().as_slice();
    assert_eq! {
        log, &[
    /* Wave 1 */("Started", "BuildCamp"),
                ("Started", "PrepareCampaign"),

    /* Wave 2 */("Finished", "BuildCamp"),
                ("Started", "GatherResources"),

    /* Wave 3 */("Finished", "PrepareCampaign"),
                ("Started", "TrainGrunts"),
                ("Started", "AssembleArmy"),

    /* Wave 4 */("Finished", "GatherResources"),
                ("Started", "Defend"),
                ("Started", "RequestReinforcements"),
                ("Started", "BuildWarmachines"),

    /* Wave 5 */("Finished", "TrainGrunts"),
                ("Finished", "AssembleArmy"),
                ("Started", "LaunchCampaign"),

    /* Wave 6 */("Finished", "Defend"),
                ("Finished", "RequestReinforcements"),
                ("Finished", "BuildWarmachines"),
                ("Finished", "LaunchCampaign"),
                ("Started", "CelebrateVictory"),
                ("Started", "X"),

    /* Wave 7 */("Finished", "CelebrateVictory"),
                ("Finished", "X"),
                ("Started", "Y"),

    /* Wave 8 */("Finished", "Y"),
            ]
        };
}

#[test]
fn warchief_campaign_attr_macro() {
    let mut app = App::new();
    app.add_plugins(OrcPlugin);
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

    #[graph(
        BuildCamp -> (
            gather: GatherResources,
            (Defend | RequestReinforcements) -> @reinforce,
        );

        prepare: PrepareCampaign -> (
            [gather] -> BuildWarmachines
            | TrainGrunts
            | @reinforce
        );

        victory: CelebrateVictory;

        [prepare] -> AssembleArmy -> LaunchCampaign -> [victory];
    )]
    #[params(reinforce)]
    struct WarchiefCampaign;

    app.world_mut().commands().queue_graph(WarchiefCampaign {
        reinforce: &orc!(X -> Y),
    });

    // No ECS wind-up: some action resolved at the same frame
    tick(&mut app, 8);

    let log = app.world().resource::<Log>().as_slice();
    assert_eq! {
        log, &[
    /* Wave 1 */("Started", "BuildCamp"),
                ("Started", "PrepareCampaign"),

    /* Wave 2 */("Finished", "BuildCamp"),
                ("Started", "GatherResources"),

    /* Wave 3 */("Finished", "PrepareCampaign"),
                ("Started", "TrainGrunts"),
                ("Started", "AssembleArmy"),

    /* Wave 4 */("Finished", "GatherResources"),
                ("Started", "Defend"),
                ("Started", "RequestReinforcements"),
                ("Started", "BuildWarmachines"),

    /* Wave 5 */("Finished", "TrainGrunts"),
                ("Finished", "AssembleArmy"),
                ("Started", "LaunchCampaign"),

    /* Wave 6 */("Finished", "Defend"),
                ("Finished", "RequestReinforcements"),
                ("Finished", "BuildWarmachines"),
                ("Finished", "LaunchCampaign"),
                ("Started", "CelebrateVictory"),
                ("Started", "X"),

    /* Wave 7 */("Finished", "CelebrateVictory"),
                ("Finished", "X"),
                ("Started", "Y"),

    /* Wave 8 */("Finished", "Y"),
            ]
        };
}

fn on_started<T: FromReflect + TypePath + Default>(
    started: Query<&OrcNode, Added<Started<T>>>,
    mut commands: Commands,
    mut queue: ResMut<Log>,
) {
    for &OrcNode {
        reactor_id,
        node_id,
        ..
    } in started
    {
        let type_str = get_name::<T>();
        queue.push(("Started", type_str));

        commands.trigger(ResolveNode {
            reactor_id,
            node_id,
            resolution: Resolution::Finished,
        });
    }
}

fn on_finished<T: FromReflect + TypePath + Default>(
    resolved: Query<(), Added<Finished<T>>>,
    mut queue: ResMut<Log>,
) {
    for _ in resolved {
        let type_str = get_name::<T>();
        queue.push(("Finished", type_str));
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
                    (on_started::<$node>, on_finished::<$node>),
                )*
            ).chain());
        };
    }
    pub(crate) use mock_systems;
}
