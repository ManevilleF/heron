#![cfg(any(dim2, dim3))]

use std::f32::consts::PI;
use std::time::Duration;

use bevy::core::CorePlugin;
use bevy::prelude::*;
use bevy::reflect::TypeRegistryArc;
use rstest::rstest;

use heron_core::*;
use heron_rapier::convert::{IntoBevy, IntoRapier};
use heron_rapier::RapierPlugin;
use utils::*;

mod utils;

fn test_app() -> App {
    let mut builder = App::new();
    builder
        .init_resource::<TypeRegistryArc>()
        .insert_resource(PhysicsSteps::every_frame(Duration::from_secs(1)))
        .add_plugin(CorePlugin)
        .add_plugin(RapierPlugin);
    builder
}

#[rstest]
#[case(RigidBody::Dynamic)]
#[case(RigidBody::KinematicVelocityBased)]
fn body_is_created_with_velocity(#[case] body_type: RigidBody) {
    let mut app = test_app();

    let linear = Vec3::new(1.0, 2.0, 3.0);
    let angular = AxisAngle::new(Vec3::Z, 2.0);

    let entity = app
        .world
        .spawn()
        .insert_bundle((
            Transform::default(),
            GlobalTransform::default(),
            body_type,
            CollisionShape::Sphere { radius: 1.0 },
            Velocity { linear, angular },
        ))
        .id();

    app.update();

    let bodies = app.world.resource::<RigidBodySet>();

    let body = bodies
        .get(
            app.world
                .get::<heron_rapier::RigidBodyHandle>(entity)
                .unwrap()
                .into_rapier(),
        )
        .unwrap();

    let actual_linear = (*body.linvel()).into_bevy();

    assert_eq!(linear.x, actual_linear.x);
    assert_eq!(linear.y, actual_linear.y);

    #[cfg(dim3)]
    assert_eq!(linear.z, actual_linear.z);

    #[cfg(dim3)]
    assert_eq!(angular, (*body.angvel()).into_bevy().into());

    #[cfg(dim2)]
    assert_eq!(angular.angle(), body.angvel());
}

#[rstest]
#[case(RigidBody::Dynamic)]
#[case(RigidBody::KinematicVelocityBased)]
fn velocity_may_be_added_after_creating_the_body(#[case] body_type: RigidBody) {
    let mut app = test_app();

    let entity = app
        .world
        .spawn()
        .insert_bundle((
            Transform::default(),
            GlobalTransform::default(),
            body_type,
            CollisionShape::Sphere { radius: 1.0 },
        ))
        .id();

    app.update();

    let linear = Vec3::new(1.0, 2.0, 3.0);
    let angular = AxisAngle::new(Vec3::Z, 2.0);

    app.world
        .entity_mut(entity)
        .insert(Velocity { linear, angular });

    app.update();

    let bodies = app.world.resource::<RigidBodySet>();

    let body = bodies
        .get(
            app.world
                .get::<heron_rapier::RigidBodyHandle>(entity)
                .unwrap()
                .into_rapier(),
        )
        .unwrap();

    let actual_linear = (*body.linvel()).into_bevy();
    assert_eq!(linear.x, actual_linear.x);
    assert_eq!(linear.y, actual_linear.y);

    #[cfg(dim3)]
    assert_eq!(linear.z, actual_linear.z);

    #[cfg(dim2)]
    assert_eq!(angular.angle(), body.angvel());

    #[cfg(dim3)]
    assert_eq!(angular, (*body.angvel()).into_bevy().into());
}

#[test]
fn velocity_is_updated_to_reflect_rapier_world() {
    let mut app = test_app();

    let linear = Vec3::new(1.0, 2.0, 3.0);
    let angular: AxisAngle = AxisAngle::new(Vec3::Z, PI * 0.5);

    let entity = app
        .world
        .spawn()
        .insert_bundle((
            Transform::default(),
            GlobalTransform::default(),
            RigidBody::Dynamic,
            CollisionShape::Sphere { radius: 1.0 },
            Velocity::default(),
            Acceleration::from_linear(linear).with_angular(angular),
        ))
        .id();

    app.update();
    app.update();

    let velocity = app.world.get::<Velocity>(entity).unwrap();

    assert_eq!(velocity.linear.x, linear.x);
    assert_eq!(velocity.linear.y, linear.y);

    #[cfg(dim3)]
    assert_eq!(velocity.linear.z, linear.z);

    #[cfg(dim3)]
    assert_eq!(angular, velocity.angular);

    #[cfg(dim2)]
    assert!((angular.angle() - velocity.angular.angle()).abs() < 0.001);
}

#[rstest]
#[case(RigidBody::Dynamic)]
#[case(RigidBody::KinematicVelocityBased)]
fn velocity_can_move_kinematic_bodies(#[case] body_type: RigidBody) {
    let mut app = test_app();
    let translation = Vec3::new(1.0, 2.0, 3.0);
    let rotation = Quat::from_axis_angle(Vec3::Z, PI / 2.0);

    let entity = app
        .world
        .spawn()
        .insert_bundle((
            body_type,
            CollisionShape::Sphere { radius: 2.0 },
            Transform::default(),
            GlobalTransform::default(),
            Velocity::from(translation).with_angular(rotation.into()),
        ))
        .id();

    app.update();

    let Transform {
        translation: actual_translation,
        rotation: actual_rotation,
        ..
    } = *app.world.get::<Transform>(entity).unwrap();

    #[cfg(dim3)]
    assert_eq!(actual_translation, translation);

    #[cfg(dim2)]
    assert_eq!(actual_translation.truncate(), translation.truncate());

    let (axis, angle) = rotation.to_axis_angle();
    let (actual_axis, actual_angle) = actual_rotation.to_axis_angle();

    assert!(actual_axis.angle_between(axis) < 0.001);
    assert!((actual_angle - angle).abs() < 0.001);
}

#[test]
#[cfg(dim2)]
fn z_components_is_preserved() {
    let mut app = test_app();
    let translation = Vec3::new(1.0, 2.0, 3.0);

    let entity = app
        .world
        .spawn()
        .insert_bundle((
            RigidBody::Dynamic,
            CollisionShape::Sphere { radius: 2.0 },
            Transform::from_translation(Vec3::splat(5.0)),
            GlobalTransform::default(),
            Velocity::from(translation),
        ))
        .id();

    app.update();

    let Transform {
        translation: actual_translation,
        ..
    } = *app.world.get::<Transform>(entity).unwrap();

    assert_eq!(5.0, actual_translation.z);
}
