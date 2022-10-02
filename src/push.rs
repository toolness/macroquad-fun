use crate::entity::{EntityMap, EntityMapHelpers};

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
    // This solely exists as an instance variable so we can amortize
    // allocations across frames.
    entities_to_process: Vec<u64>,
}

impl PushSystem {
    pub fn with_capacity(capacity: usize) -> Self {
        PushSystem {
            entities_to_process: Vec::with_capacity(capacity),
        }
    }

    pub fn run(&mut self, entities: &mut EntityMap) {
        self.entities_to_process.clear();
        self.entities_to_process
            .extend(entities.iter().filter_map(|(&id, entity)| {
                if let Some(push) = entity.push.as_ref() {
                    if push.can_push {
                        return Some(id);
                    }
                }
                return None;
            }));

        for &id in self.entities_to_process.iter() {
            entities.with_entity_removed(id, |pusher, entities| {
                for pushed in entities.values_mut() {
                    if let Some(push) = &pushed.push {
                        if push.pushable_coefficient > 0. {
                            let pusher_bbox = pusher.sprite.bbox();
                            let pushed_bbox = pushed.sprite.bbox();
                            if let Some(intersection) = pusher_bbox.intersect(pushed_bbox) {
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
            });
        }
    }
}
