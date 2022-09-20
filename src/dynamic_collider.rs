use crate::collision::Collider;

#[derive(Default)]
pub struct DynamicColliderComponent {
    pub relative_collider: Collider,
}
