use bevy::{input::mouse::MouseMotion, prelude::*};
use dolly::prelude::*;

pub struct FlyCameraPlugin;

impl Plugin for FlyCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_camera);
    }
}

#[derive(Component)]
pub struct FlyCamera {
    camera_rig: CameraRig,
    sensitivity: f32,
    speed: f32,
}

impl FlyCamera {
    pub fn new(sensitivity: f32, speed: f32) -> Self {
        let camera = CameraRig::<RightHanded>::builder()
            .with(Position::new(dolly::glam::Vec3::Y))
            .with(YawPitch::new())
            .with(Smooth::new_position_rotation(1.0, 1.0))
            .build();

        Self {
            camera_rig: camera,
            sensitivity,
            speed,
        }
    }
}

pub fn update_camera(
    keyboard_input: Res<Input<KeyCode>>,
    mut mouse_motion_event_reader: EventReader<MouseMotion>,
    mut camera_controller: Query<(&mut Transform, &mut FlyCamera), With<Camera>>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    let mut mouse_delta = Vec2::ZERO;
    for event in mouse_motion_event_reader.iter() {
        mouse_delta += event.delta;
    }
    if mouse_delta.is_nan() {
        mouse_delta = Vec2::ZERO;
    }
    let mut input_vec = Vec3::ZERO;
    input_vec.x = movement_axis(&keyboard_input, KeyCode::D, KeyCode::A);
    input_vec.y = movement_axis(&keyboard_input, KeyCode::Space, KeyCode::LShift);
    input_vec.z = movement_axis(&keyboard_input, KeyCode::S, KeyCode::W);

    for (mut transform, mut fly_camera) in camera_controller.iter_mut() {
        transform.translation = dolly_vec3_to_bevy(fly_camera.camera_rig.final_transform.position);
        transform.rotation = dolly_quat_to_bevy(fly_camera.camera_rig.final_transform.rotation);
        let rotation = fly_camera.camera_rig.final_transform.rotation;
        let rotation: bevy::math::Quat = dolly_quat_to_bevy(rotation);
        let mut movement = (strafe_vector(&rotation.into()) * input_vec.x)
            + (forward_walk_vector(&rotation) * input_vec.z)
            + (Vec3::Y * input_vec.y);
        movement = movement.normalize_or_zero();

        let mouse_delta = -mouse_delta * fly_camera.sensitivity * dt;
        let speed = fly_camera.speed;
        fly_camera
            .camera_rig
            .driver_mut::<YawPitch>()
            .rotate_yaw_pitch(mouse_delta.x, mouse_delta.y);
        fly_camera
            .camera_rig
            .driver_mut::<Position>()
            .translate(bevy_vec3_to_dolly(movement * speed * dt));
        fly_camera.camera_rig.update(dt);
    }
}

pub fn movement_axis(input: &Res<Input<KeyCode>>, plus: KeyCode, minus: KeyCode) -> f32 {
    let mut axis = 0.0f32;
    if input.pressed(plus) {
        axis += 1.0;
    }
    if input.pressed(minus) {
        axis -= 1.0;
    }
    axis
}

pub fn forward_vector(rotation: &Quat) -> Vec3 {
    rotation.mul_vec3(Vec3::Z).normalize()
}

pub fn forward_walk_vector(rotation: &Quat) -> Vec3 {
    let f = forward_vector(rotation);
    let f_flattened = Vec3::new(f.x, 0.0, f.z).normalize();
    f_flattened
}

pub fn strafe_vector(rotation: &Quat) -> Vec3 {
    // Rotate it 90 degrees to get the strafe direction
    Quat::from_rotation_y(90.0f32.to_radians())
        .mul_vec3(forward_walk_vector(rotation))
        .normalize()
}

#[inline]
fn dolly_vec3_to_bevy(t: dolly::glam::Vec3) -> bevy::math::Vec3 {
    bevy::math::Vec3::from_array(t.into())
}

#[inline]
fn bevy_vec3_to_dolly(t: bevy::math::Vec3) -> dolly::glam::Vec3 {
    dolly::glam::Vec3::new(t.x, t.y, t.z)
}

#[inline]
fn dolly_quat_to_bevy(t: dolly::glam::Quat) -> bevy::math::Quat {
    bevy::math::Quat::from_array(t.into())
}

#[inline]
fn bevy_quat_to_dolly(t: bevy::math::Quat) -> dolly::glam::Quat {
    dolly::glam::Quat::from_array(t.into())
}
