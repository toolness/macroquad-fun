use crate::entity::{EntityMap, EntityProcessor};

#[derive(Default)]
pub struct PushComponent {
    /// Whether the entity can push other entities.
    pub can_push: bool,

    /// The extent to which the entity can be pushed around. If this is
    /// 0, it can't be pushed; otherwise, it reflects how much it will
    /// be pushed per frame based on the pusher's current x-velocity.
    pub pushable_coefficient: f32,
}

pub struct PushSystem {
    pub processor: EntityProcessor,
}

impl PushSystem {
    pub fn run(&mut self, entities: &mut EntityMap) {
        self.processor.filter_and_process_entities(
            entities,
            |entity| {
                entity
                    .push
                    .as_ref()
                    .map(|push| push.can_push)
                    .unwrap_or(false)
            },
            |pusher, entities| {
                for pushed in entities.values_mut() {
                    if let Some(push) = &pushed.push {
                        if push.pushable_coefficient > 0. {
                            let pusher_bbox = pusher.sprite.bbox();
                            let pushed_bbox = pushed.sprite.bbox();
                            if let Some(intersection) = pusher_bbox.intersect(pushed_bbox) {
                                // Only push if the pusher is overlapping the full height of the
                                // pushed entity.
                                if intersection.h == pushed_bbox.h {
                                    let sign = if pusher_bbox.x > pushed_bbox.x {
                                        -1.
                                    } else {
                                        1.
                                    };
                                    let x_delta = intersection.w * push.pushable_coefficient * sign;
                                    pushed.sprite.pos.x += x_delta;
                                }
                            }
                        }
                    }
                }
            },
        )
    }
}
