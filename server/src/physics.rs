use bevy::prelude::*;
use rapier2d::dynamics::SoftBodySet;
use rapier2d::math::Vector;
use rapier2d::{
    dynamics::{
        CCDSolver, ImpulseJointSet, IntegrationParameters, IslandManager, MultibodyJointSet,
        RigidBodySet,
    },
    geometry::{ColliderSet, DefaultBroadPhase, NarrowPhase},
    pipeline::PhysicsPipeline,
};

#[derive(Resource, Default)]
pub(crate) struct RapierIntegrationParameters(pub(crate) IntegrationParameters);

#[derive(Resource, Default)]
pub(crate) struct RapierIslandManager(pub(crate) IslandManager);

#[derive(Resource, Default)]
pub(crate) struct RapierBroadPhase(pub(crate) DefaultBroadPhase);

#[derive(Resource, Default)]
pub(crate) struct RapierNarrowPhase(pub(crate) NarrowPhase);

#[derive(Resource, Default)]
pub(crate) struct RapierImpulseJointSet(pub(crate) ImpulseJointSet);

#[derive(Resource, Default)]
pub(crate) struct RapierMultibodyJointSet(pub(crate) MultibodyJointSet);

#[derive(Resource, Default)]
pub(crate) struct RapierCCDSolver(pub(crate) CCDSolver);

#[derive(Resource, Default)]
pub(crate) struct RapierBodies(pub(crate) RigidBodySet);

#[derive(Resource, Default)]
pub(crate) struct RapierColliders(pub(crate) ColliderSet);

#[derive(Resource, Default)]
pub(crate) struct RapierPipeline(pub(crate) PhysicsPipeline);

#[derive(Resource, Default)]
pub(crate) struct RapierSoftBodies(pub(crate) SoftBodySet);

pub(crate) fn cycle(
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
    mut rapier_softbodies: ResMut<RapierSoftBodies>,
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
        &mut rapier_softbodies.0,
        &mut rapier_ccd_solver.0,
        &(),
        &(),
    );
}
