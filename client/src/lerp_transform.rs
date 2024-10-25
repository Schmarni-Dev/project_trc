use bevy::prelude::*;

pub struct LerpTransformPlugin;

impl Plugin for LerpTransformPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, lerp_pos_system);
        app.add_systems(Update, lerp_rot_system);
    }
}

pub fn lerp_rot_system(time: Res<Time>, mut query: Query<(&mut Transform, &mut LerpTransform)>) {
    for (mut transform, mut lerp_rot) in &mut query {
        lerp_rot.current_rot_time =
            (lerp_rot.current_rot_time + (time.delta_seconds() / lerp_rot.rot_time)).clamp(0., 1.);
        transform.rotation = lerp_rot
            .start_rot
            .lerp(lerp_rot.end_rot, lerp_rot.current_rot_time);
    }
}

pub fn lerp_pos_system(time: Res<Time>, mut query: Query<(&mut Transform, &mut LerpTransform)>) {
    for (mut transform, mut lerp_pos) in &mut query {
        lerp_pos.current_pos_time =
            (lerp_pos.current_pos_time + (time.delta_seconds() / lerp_pos.pos_time)).clamp(0., 1.);
        transform.translation = lerp_pos
            .start_pos
            .lerp(lerp_pos.end_pos, lerp_pos.current_pos_time);
    }
}
#[derive(Component)]
pub struct LerpTransform {
    pub pos_time: f32,
    pub start_pos: Vec3,
    pub end_pos: Vec3,
    pub current_pos_time: f32,
    pub start_rot: Quat,
    pub end_rot: Quat,
    pub current_rot_time: f32,
    pub rot_time: f32,
}

impl LerpTransform {
    pub fn new(pos: Vec3, rot: Quat) -> LerpTransform {
        LerpTransform {
            pos_time: 1.,
            start_pos: pos,
            end_pos: pos,
            current_pos_time: 1.,
            start_rot: rot,
            end_rot: rot,
            current_rot_time: 1.,
            rot_time: 1.,
        }
    }
    pub fn lerp_rot_to(&mut self, end_rot: Quat, time: f32) -> &mut Self {
        self.start_rot = self.end_rot;
        self.end_rot = end_rot;
        self.current_rot_time = 0.;
        self.rot_time = time;
        self
    }
    pub fn lerp_pos_to(&mut self, end_pos: Vec3, time: f32) -> &mut Self {
        self.start_pos = self.end_pos;
        self.end_pos = end_pos;
        self.current_pos_time = 0.;
        self.pos_time = time;
        self
    }
}
