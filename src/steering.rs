use crate::entity::EntityMap;

#[derive(Default, Clone, Copy)]
pub struct SteeringComponent {
    pub x_direction: i8,
}

pub fn steering_system(entities: &mut EntityMap) {
    for (_, entity) in entities.iter_mut() {
        if let Some(steering) = entity.steering.as_mut() {
            if steering.x_direction != 0 {
                if steering.x_direction < 0 && entity.physics.velocity.x > 0.
                    || steering.x_direction > 0 && entity.physics.velocity.x < 0.
                {
                    entity.physics.velocity.x *= -1.0;
                }
                steering.x_direction = 0;
            }
        }
    }
}
