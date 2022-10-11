use rapier2d::prelude::*;

use crate::{config::config, entity::EntityMap, level::Level, time::GameTime};

#[derive(Default)]
pub struct RapierComponent {
    rigid_body_handle: RigidBodyHandle,
}

pub struct RapierSystem {
    gravity: Vector<Real>,
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    physics_hooks: (),
    event_handler: (),
}

impl RapierSystem {
    pub fn new(level: &'static Level) -> Self {
        let mut rigid_body_set = RigidBodySet::new();
        let mut collider_set = ColliderSet::new();
        for collider in level.iter_colliders(&level.pixel_bounds()) {
            let half_extents = vector![collider.rect.w / 2., collider.rect.h / 2.];
            let origin: Vector<Real> = collider.rect.point().into();
            let rigid_body = RigidBodyBuilder::fixed()
                .translation(origin + half_extents)
                .build();
            let handle = rigid_body_set.insert(rigid_body);
            let collider = ColliderBuilder::cuboid(half_extents.x, half_extents.y);
            collider_set.insert_with_parent(collider, handle, &mut rigid_body_set);
        }
        RapierSystem {
            // We manually apply gravity ourselves, so it's set to zero here.
            gravity: vector![0.0, 0.0],
            rigid_body_set,
            collider_set,
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            physics_hooks: (),
            event_handler: (),
        }
    }

    pub fn apply_impulse(&mut self, rapier: &RapierComponent, amount: macroquad::prelude::Vec2) {
        let body = self
            .rigid_body_set
            .get_mut(rapier.rigid_body_handle)
            .unwrap();
        body.apply_impulse(amount.into(), true);
    }

    pub fn run(&mut self, entities: &mut EntityMap, time: &GameTime) {
        for (_id, entity) in entities.iter_mut() {
            if entity.physics.use_rapier {
                if entity.rapier.is_none() {
                    let bbox = entity.sprite.bbox();
                    let half_extents = vector![bbox.w / 2., bbox.h / 2.];
                    let origin: Vector<Real> = bbox.point().into();
                    let mut rigid_body = RigidBodyBuilder::dynamic()
                        .translation(origin + half_extents)
                        .lock_rotations()
                        .build();
                    if !entity.physics.defies_gravity {
                        rigid_body.add_force(vector![0.0, config().gravity], true);
                    }
                    let rigid_body_handle = self.rigid_body_set.insert(rigid_body);
                    let collider =
                        ColliderBuilder::cuboid(half_extents.x, half_extents.y).density(0.001);
                    let _collider_handle = self.collider_set.insert_with_parent(
                        collider,
                        rigid_body_handle,
                        &mut self.rigid_body_set,
                    );
                    entity.rapier = Some(RapierComponent { rigid_body_handle });
                }
            }
        }
        self.integration_parameters.dt = time.time_since_last_frame as f32;
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            &self.physics_hooks,
            &self.event_handler,
        );
        for (_id, entity) in entities.iter_mut() {
            if let Some(rapier) = &entity.rapier {
                let body = &self.rigid_body_set[rapier.rigid_body_handle];
                let bbox = entity.sprite.bbox();
                let half_extents = vector![bbox.w / 2., bbox.h / 2.];
                let center = body.translation();
                let top_left = center - half_extents;
                entity.sprite.pos = top_left.into();
            }
        }
    }
}
