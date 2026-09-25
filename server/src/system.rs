use bevy::prelude::*;
use bevy_app::AppExit;
use crossbeam_channel::TryRecvError;
use rapier2d::{
    dynamics::RigidBodyBuilder,
    geometry::ColliderBuilder,
    math::{Vec2, Vector},
};
use std::num::NonZero;
use std::time::Duration;
use tokio::time::Instant;
use tracing::debug;
use tracing::{info, warn};

use crate::{
    component::{
        Beacon, Factory, HasExtended, Id, Kind, Owned, PlayerResources, RapierObject, ZoneExtension,
    },
    helper,
    player::OnlinePlayer,
    resource::{
        ControlsR, NewPlayerReceiver, Players, RapierBodies, RapierBroadPhase, RapierCCDSolver,
        RapierColliders, RapierImpulseJointSet, RapierIntegrationParameters, RapierIslandManager,
        RapierMultibodyJointSet, RapierNarrowPhase, RapierPipeline, RulesR, TerrainGenerator,
        Timers, TransportStopper,
    },
};

pub(crate) fn startup(rules: Res<RulesR>) {
    let rules_str = format!("{:#?}", rules.0);
    debug!(sender = "Game", "{rules_str}");
    info!(sender = "Game", "Waiting for players");
}

#[allow(unused)]
pub(crate) fn debug(entities: Query<&Kind>, mut timers: ResMut<Timers>) {
    let now = Instant::now();
    let kinds = entities.iter().map(|x| &x.0).collect();
    if now - timers.last_info > Duration::from_secs_f32(10.) {
        debug!(
            sender = "Debug",
            "Stat: Minerals: {}, Factories: {}, Dronoids: {},  SpawnBeacon: {}",
            helper::count_kind(&kinds, &dronoid_protocol::Kind::Mineral),
            helper::count_kind(&kinds, &dronoid_protocol::Kind::Factory),
            helper::count_kind(&kinds, &dronoid_protocol::Kind::Dronoid),
            helper::count_kind(&kinds, &dronoid_protocol::Kind::Spawn),
        );
        timers.last_info = now;
    }
}

pub(crate) fn cycle(
    controls: ResMut<ControlsR>,
    transport_stopper: Res<TransportStopper>,
    rules: Res<RulesR>,
    mut timers: ResMut<Timers>,
    mut exit: MessageWriter<AppExit>,
) {
    let now = Instant::now();
    let delta = (now - timers.last_cycle).as_secs_f32();
    timers.last_cycle = now;
    if delta > rules.0.tick_duration * 2. {
        let late = (delta / rules.0.tick_duration) as u32;
        warn!(
            sender = "Game",
            "Delta is {delta} seconds. Server is late of {late} ticks"
        );
    }

    if controls.0.stopped() {
        let app_exit = if transport_stopper.0.send(()).is_err() {
            AppExit::Error(NonZero::new(1).unwrap())
        } else {
            AppExit::Success
        };
        exit.write(app_exit);
    }
}

pub(crate) fn new_players(
    new_player_rx: ResMut<NewPlayerReceiver>,
    mut players: ResMut<Players>,
    mut rapier_bodies: ResMut<RapierBodies>,
    mut rapier_colliders: ResMut<RapierColliders>,
    rules: Res<RulesR>,
    mut commands: Commands,
) {
    while let Ok(new_player) = new_player_rx.0.try_recv() {
        let player_name = new_player.name.clone();
        info!(sender = "Game", "Player '{player_name}' joined");
        players.0.insert(
            new_player.id,
            OnlinePlayer::new(
                new_player.message_sender,
                new_player.action_receiver,
                new_player.name,
                rules.0.starting_minerals,
            ),
        );
        let rigid_body = RigidBodyBuilder::fixed()
            .translation(rapier2d::math::Vector::new(
                new_player.spawn_point.0,
                new_player.spawn_point.1,
            ))
            .build();
        let rapier_hdl = rapier_bodies.0.insert(rigid_body);
        let collider = ColliderBuilder::cuboid(10., 10.).build();
        rapier_colliders
            .0
            .insert_with_parent(collider, rapier_hdl, &mut rapier_bodies.0);
        commands.spawn((
            Beacon,
            Kind(dronoid_protocol::Kind::Spawn),
            Owned(new_player.id),
            ZoneExtension { radius: 100. },
            RapierObject { rapier_hdl },
            Id(helper::gen_id()),
        ));
    }
}

pub(crate) fn terrain(
    zone_extenders: Query<(Entity, &ZoneExtension, &RapierObject), Without<HasExtended>>,
    spawns: Query<&RapierObject, With<Beacon>>,
    mut terrain_generator: ResMut<TerrainGenerator>,
    rules: Res<RulesR>,
    mut rapier_bodies: ResMut<RapierBodies>,
    mut rapier_colliders: ResMut<RapierColliders>,
    mut commands: Commands,
) {
    for (entity, zone_extender, rapier_object) in zone_extenders.iter() {
        let zone_position = rapier_object.position(&rapier_bodies.0);
        let radius = zone_extender.radius.ceil() as i32;
        for x in -radius..radius {
            'here: for y in -radius..radius {
                let mineral_x = zone_position.0 + x as f32;
                let mineral_y = zone_position.1 + y as f32;
                if (mineral_x - zone_position.0).powf(2.) + (mineral_y - zone_position.1).powf(2.)
                    < radius.pow(2) as f32
                {
                    for spawn_position_hdl in spawns.iter() {
                        let spawn_position = spawn_position_hdl.position(&rapier_bodies.0);
                        if (mineral_x - spawn_position.0).powf(2.)
                            + (mineral_y - spawn_position.1).powf(2.)
                            < 20usize.pow(2) as f32
                        {
                            continue 'here;
                        }
                    }
                    if !terrain_generator.put_mineral(mineral_x, mineral_y, &rules.0) {
                        continue 'here;
                    }
                    let rigid_body = RigidBodyBuilder::fixed()
                        .translation(Vec2::new(mineral_x, mineral_y))
                        .build();
                    let rapier_hdl = rapier_bodies.0.insert(rigid_body);
                    let collider = ColliderBuilder::cuboid(1., 1.).build();
                    rapier_colliders.0.insert_with_parent(
                        collider,
                        rapier_hdl,
                        &mut rapier_bodies.0,
                    );
                    commands.spawn((
                        PlayerResources,
                        RapierObject { rapier_hdl },
                        Kind(dronoid_protocol::Kind::Mineral),
                        Id(helper::gen_id()),
                    ));
                }
            }
        }
        commands.entity(entity).insert(HasExtended);
    }
}

pub(crate) fn actions(
    zone_extenders: Query<(&ZoneExtension, &RapierObject, &Owned)>,
    mut factories: Query<&mut Factory>,
    mut rapier_bodies: ResMut<RapierBodies>,
    mut rapier_colliders: ResMut<RapierColliders>,
    mut players: ResMut<Players>,
    rules: Res<RulesR>,
    mut commands: Commands,
) {
    let mut ids_to_remove = Vec::<u32>::new();

    for (id, player) in &mut players.0 {
        loop {
            match player.action_receiver.try_recv() {
                Err(err) => match err {
                    TryRecvError::Disconnected => {
                        ids_to_remove.push(*id);
                        break;
                    }
                    TryRecvError::Empty => break,
                },
                Ok(player_action) => match player_action {
                    dronoid_protocol::Action::PlaceFactory((pos_x, pos_y)) => {
                        let may_build = helper::is_in_player_zone(
                            pos_x,
                            pos_y,
                            id,
                            zone_extenders.iter().collect(),
                            &rapier_bodies.0,
                        ) && player.minerals_cnt >= rules.0.factory_cost;

                        if may_build {
                            player.minerals_cnt -= rules.0.factory_cost;

                            let rigid_body = RigidBodyBuilder::fixed()
                                .translation(Vec2::new(pos_x, pos_y))
                                .build();
                            let rapier_hdl = rapier_bodies.0.insert(rigid_body);
                            let collider = ColliderBuilder::cuboid(10., 10.).build();
                            rapier_colliders.0.insert_with_parent(
                                collider,
                                rapier_hdl,
                                &mut rapier_bodies.0,
                            );
                            let id = helper::gen_id();
                            player.owned_factories.insert(
                                id,
                                commands
                                    .spawn((
                                        ZoneExtension {
                                            radius: rules.0.factory_extension,
                                        },
                                        RapierObject { rapier_hdl },
                                        Factory::default(),
                                        Kind(dronoid_protocol::Kind::Factory),
                                        Id(id),
                                    ))
                                    .id(),
                            );
                        }

                        player
                            .messages
                            .push(dronoid_protocol::ServerMessage::Response(
                                dronoid_protocol::Response::PlaceFactory { result: may_build },
                            ));
                    }
                    dronoid_protocol::Action::ControlFactory(control) => {
                        let maybe_bevy_entity = player.owned_factories.get(&control.id);
                        if maybe_bevy_entity.is_none() {
                            player.to_kick = true;
                            break;
                        }
                        let bevy_entity = maybe_bevy_entity.unwrap();
                        let mut factory = factories.get_mut(*bevy_entity).unwrap();
                        match control.order {
                            dronoid_protocol::FactoryOrder::ManualSpawn => {
                                factory.must_spawn = true;
                            }
                            dronoid_protocol::FactoryOrder::SetAutoSpawn(value) => {
                                factory.auto_spawn = value;
                            }
                        }
                    }
                },
            }
        }
    }
}

pub(crate) fn factories(
    mut query: Query<&mut Factory>,
    mut rapier_bodies: ResMut<RapierBodies>,
    mut rapier_colliders: ResMut<RapierColliders>,
    rules: Res<RulesR>,
    mut commands: Commands,
) {
    for mut factory in &mut query {
        if factory.cooldown < 0. {
            if factory.must_spawn || factory.auto_spawn {
                factory.spawn_dronoid(&mut rapier_bodies.0, &mut rapier_colliders.0, &mut commands);
                factory.cooldown = 5.;
                if factory.must_spawn {
                    factory.must_spawn = false
                }
            }
        } else {
            factory.cooldown -= rules.0.tick_duration;
        }
    }
}

pub(crate) fn physics(
    rapier_integration_parameters: Res<RapierIntegrationParameters>,
    mut rapier_island_manager: ResMut<RapierIslandManager>,
    mut rapier_broad_phase: ResMut<RapierBroadPhase>,
    mut rapier_narrow_phase: ResMut<RapierNarrowPhase>,
    mut rapier_impulse_joint_set: ResMut<RapierImpulseJointSet>,
    mut rapier_multibody_joint_set: ResMut<RapierMultibodyJointSet>,
    mut rapier_ccd_solver: ResMut<RapierCCDSolver>,
    mut rapier_pipeline: ResMut<RapierPipeline>,
    mut rapier_bodies: ResMut<RapierBodies>,
    mut rapier_colliders: ResMut<RapierColliders>,
) {
    rapier_pipeline.0.step(
        Vector::new(0., 0.),
        &rapier_integration_parameters.0,
        &mut rapier_island_manager.0,
        &mut rapier_broad_phase.0,
        &mut rapier_narrow_phase.0,
        &mut rapier_bodies.0,
        &mut rapier_colliders.0,
        &mut rapier_impulse_joint_set.0,
        &mut rapier_multibody_joint_set.0,
        &mut rapier_ccd_solver.0,
        &(),
        &(),
    );
}

pub(crate) fn sync(
    entities: Query<(&RapierObject, &Kind, &Id)>,
    zone_extenders: Query<(&ZoneExtension, &RapierObject, &Owned)>,
    rapier_bodies: Res<RapierBodies>,
    mut players: ResMut<Players>,
) {
    for (id, player) in &mut players.0 {
        let mut state = dronoid_protocol::State::default();
        state.entities_in_zone = helper::get_entities_in_zone(
            *id,
            entities.iter().collect(),
            &rapier_bodies.0,
            zone_extenders,
        );
        state.minerals_cnt = player.minerals_cnt;
        player
            .messages
            .push(dronoid_protocol::ServerMessage::State(state));
    }
}

pub(crate) fn flush(mut players: ResMut<Players>) {
    let mut ids_to_remove = Vec::<u32>::new();
    for (id, player) in &mut players.0 {
        if player.to_kick || player.flush_messages().is_err() {
            ids_to_remove.push(*id);
        }
    }
    for id in ids_to_remove {
        players.0.remove(&id);
    }
}
