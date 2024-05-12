use bevy_dolly::{
    dolly::{self, prelude::*},
    prelude::RigDriver,
};
use bevy::prelude::*;

impl MovableLookAt {
    pub fn from_position_target(target_position: Vec3,offset:Vec3) -> Self {
        Self(
            CameraRig::builder()
                .with(Position::new(target_position+offset))
                .with(Smooth::new_position(1.0).predictive(true))
                //.with(Smooth::new_position(1.25))                
                .with(
                    LookAt::new(target_position )
                        
                )
                .build(),
        )
    }

    pub fn set_position_target(&mut self, target_position: Vec3,offset:Vec3) {
        self.driver_mut::<Position>().position = target_position+offset;
        self.driver_mut::<LookAt>().target = target_position;
    }
}

/// A custom camera rig which combines smoothed movement with a look-at driver.
#[derive(Component, Debug, Deref, DerefMut)]
pub struct MovableLookAt(CameraRig);

// Turn the nested rig into a driver, so it can be used in another rig.
impl RigDriver for MovableLookAt {
    fn update(&mut self, params: dolly::rig::RigUpdateParams) -> Transform {
        self.0.update(params.delta_time_seconds)
    }
}
