use rapier3d::prelude::*;

#[test]
fn main() {
	// The set that will contain our rigid-bodies.
	let mut rigid_body_set = RigidBodySet::new();

	// Builder for a fixed rigid-body.
	let _ = RigidBodyBuilder::fixed();
	// Builder for a dynamic rigid-body.
	let _ = RigidBodyBuilder::dynamic();
	// Builder for a kinematic rigid-body controlled at the velocity level.
	let _ = RigidBodyBuilder::kinematic_velocity_based();
	// Builder for a kinematic rigid-body controlled at the position level.
	let _ = RigidBodyBuilder::kinematic_position_based();
	// Builder for a body with a status specified by an enum.
	let rigid_body = RigidBodyBuilder::new(RigidBodyType::Dynamic)
		// The rigid body translation.
		// Default: zero vector.
		.translation(vector![0.0, 5.0, 1.0])
		// The rigid body rotation.
		// Default: no rotation.
		.rotation(vector![0.0, 0.0, 5.0])
		// The rigid body position. Will override `.translation(...)` and `.rotation(...)`.
		// Default: the identity isometry.
		.position(Isometry::new(
			vector![1.0, 3.0, 2.0],
			vector![0.0, 0.0, 0.4],
		))
		// The linear velocity of this body.
		// Default: zero velocity.
		.linvel(vector![1.0, 3.0, 4.0])
		// The angular velocity of this body.
		// Default: zero velocity.
		.angvel(vector![3.0, 0.0, 1.0])
		// The scaling factor applied to the gravity affecting the rigid-body.
		// Default: 1.0
		.gravity_scale(0.5)
		// Whether or not this body can sleep.
		// Default: true
		.can_sleep(true)
		// Whether or not CCD is enabled for this rigid-body.
		// Default: false
		.ccd_enabled(false)
		// All done, actually build the rigid-body.
		.build();

	// Insert the rigid-body into the set.
	let handle = rigid_body_set.insert(rigid_body);
}
