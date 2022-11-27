use macroquad::prelude::clamp;

use crate::{
    config::config,
    entity::{filter_and_process_entities, EntityMap},
    time::GameTime,
};

#[derive(Clone, Copy)]
pub enum LifeTransfer {
    Giving(f32),
    Receiving(f32),
}

pub type LifeTransferComponent = LifeTransfer;

pub fn life_transfer_system(entities: &mut EntityMap, time: &GameTime) {
    filter_and_process_entities(
        entities,
        |entity| matches!(entity.life_transfer, Some(LifeTransfer::Giving(_))),
        |life_giving_entity, entities, _| {
            let config = config();
            let mut max_glow_amount = 0.;
            let life_giving_center = life_giving_entity.sprite.bbox().center();
            for (_id, life_receiving_entity) in entities.iter_mut() {
                let Some(LifeTransfer::Receiving(_)) = life_receiving_entity.life_transfer else {
                    continue;
                };
                let life_receiving_center = life_receiving_entity.sprite.bbox().center();
                let distance = clamp(
                    life_receiving_center.distance(life_giving_center)
                        - config.life_transfer_min_radius,
                    // We never want this to be zero because we're using it
                    // as a denominator later, and we never want to divide by zero.
                    0.001,
                    config.life_transfer_max_radius,
                );

                // Make a base oscillator from -1 to 1.
                let base_oscillator =
                    (time.now as f32 * config.life_transfer_speed_coefficient).sin();

                // Shift the oscillator to go from 0 to 1.
                let zero_to_one_oscillator = (1. + base_oscillator) / 2.;

                // Now shift it to go from 1-config.life_transfer_oscillate_amount to 1.
                let oscillator = zero_to_one_oscillator * config.life_transfer_oscillate_amount
                    + (1. - config.life_transfer_oscillate_amount);

                let base_glow_amount = 1. - distance / config.life_transfer_max_radius;
                let oscillating_glow_amount = oscillator * base_glow_amount;
                if oscillating_glow_amount > max_glow_amount {
                    max_glow_amount = oscillating_glow_amount;
                }
                life_receiving_entity.life_transfer =
                    Some(LifeTransfer::Receiving(oscillating_glow_amount));
            }
            life_giving_entity.life_transfer = Some(LifeTransfer::Giving(max_glow_amount));
        },
    );
}

pub fn get_life_receiving_amount_or_zero(life_transfer: Option<LifeTransfer>) -> f32 {
    if let Some(LifeTransfer::Receiving(amount)) = life_transfer {
        amount
    } else {
        0.
    }
}

pub fn get_life_giving_amount_or_zero(life_transfer: Option<LifeTransfer>) -> f32 {
    if let Some(LifeTransfer::Giving(amount)) = life_transfer {
        amount
    } else {
        0.
    }
}
