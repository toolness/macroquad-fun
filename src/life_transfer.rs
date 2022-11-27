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
            let mut did_give = false;
            let Some(LifeTransfer::Giving(give_amount)) = life_giving_entity.life_transfer else {
                panic!("Assertion failure, filter guarantees this condition")
            };
            for (_id, life_receiving_entity) in entities.iter_mut() {
                let Some(LifeTransfer::Receiving(receive_amount)) = life_receiving_entity.life_transfer else {
                    continue;
                };
                if !life_giving_entity
                    .sprite
                    .bbox()
                    .overlaps(&life_receiving_entity.sprite.bbox())
                {
                    continue;
                }
                did_give = true;
                let unclamped_new_receive_amount =
                    receive_amount + config.life_transfer_rate * time.time_since_last_frame as f32;
                let new_receive_amount = clamp(unclamped_new_receive_amount, 0., 1.);
                life_receiving_entity.life_transfer =
                    Some(LifeTransfer::Receiving(new_receive_amount));
            }
            let unclamped_new_give_amount = if did_give {
                give_amount + config.life_transfer_rate * time.time_since_last_frame as f32
            } else {
                give_amount - config.life_transfer_rate * time.time_since_last_frame as f32
            };
            let new_give_amount = clamp(unclamped_new_give_amount, 0., 1.);
            life_giving_entity.life_transfer = Some(LifeTransfer::Giving(new_give_amount));
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
