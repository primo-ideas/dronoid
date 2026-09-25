use bevy::prelude::*;
use bevy_app::{App, AppExit, Startup};
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use tracing::info;

use crate::Controls;
use crate::Error;
use crate::Rules;
use crate::player::EnteringPlayer;

pub fn run_game(
    rules: Rules,
    controls: Controls,
    transport_stopper_tx: Sender<()>,
    new_player_rx: Receiver<EnteringPlayer>,
) -> super::Result<()> {
    let exit = App::new()
        .add_plugins(MinimalPlugins)
        .add_systems(Startup, startup)
        .add_systems(FixedFirst, physics)
        .add_systems(
            FixedUpdate,
            (cycle, new_players, terrain, actions, factories, sync, flush).chain(),
        )
        .insert_resource(Time::<Fixed>::from_seconds(rules.tick_duration as f64))
        .insert_resource(RapierBroadPhase::default())
        .insert_resource(RapierCCDSolver::default())
        .insert_resource(RapierImpulseJointSet::default())
        .insert_resource(RapierIntegrationParameters::default())
        .insert_resource(RapierIslandManager::default())
        .insert_resource(RapierMultibodyJointSet::default())
        .insert_resource(RapierNarrowPhase::default())
        .insert_resource(RapierPipeline::default())
        .insert_resource(RapierBodies::default())
        .insert_resource(RapierColliders::default())
        .insert_resource(Controls(controls))
        .insert_resource(TransportStopper(transport_stopper_tx))
        .insert_resource(NewPlayerReceiver(new_player_rx))
        .insert_resource(Players::default())
        .insert_resource(TerrainGenerator::new(&rules))
        .insert_resource(Rules(rules))
        .insert_resource(Timers::default())
        .run();

    info!(sender = "Exit", "Syncing and cleaning");
    match exit {
        AppExit::Error(err_num) => {
            let num = err_num.get();
            match num {
                1 => Err(Error::StopError(num)),
                _ => {
                    todo!();
                }
            }
        }

        AppExit::Success => Ok(()),
    }
}
