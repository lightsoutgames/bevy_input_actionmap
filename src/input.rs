use bevy::prelude::*;

use crate::GamepadAxisDirection;

/// A single key press, click, button press, or gamepad axis flick
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum BindInput {
    Keyboard( KeyCode ),
    MouseButton( MouseButton ),
    GamepadButton( GamepadButtonType ),
    GamepadAxis( GamepadAxisInput ),
}

impl From<KeyCode> for BindInput {
    fn from(value: KeyCode) -> Self {
        Self::Keyboard( value )
    }
}

impl From<MouseButton> for BindInput {
    fn from(value: MouseButton) -> Self {
        Self::MouseButton( value )
    }
}

impl From<GamepadButtonType> for BindInput {
    fn from(value: GamepadButtonType) -> Self {
        Self::GamepadButton( value )
    }
}

impl From<GamepadAxisInput> for BindInput {
    fn from(value: GamepadAxisInput) -> Self {
        Self::GamepadAxis( value )
    }
}

pub const GAMEPAD_AXIS_DEFAULT_DEADZONE: f32 = 0.15;

impl From<GamepadAxisDirection> for BindInput {
    fn from(axis: GamepadAxisDirection) -> Self {
        Self::GamepadAxis( GamepadAxisInput::new(axis, GAMEPAD_AXIS_DEFAULT_DEADZONE) )
    }
}





/// 
/// 
/// Internally, the `deadzone` value is packed into a [u16]. This is nessisary to 
/// implement [Eq] and [Hash], which are required for storing in a 
/// [HashSet](std::collections::HashSet).
/// 
/// `deadzone` values will be clamped to the range [-1.0, 1.0]. The packing 
/// process loses some precision, so if you compare the before and after, it might
/// not match, even if it is within the range [-1.0, 1.0]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct GamepadAxisInput {
    axis: GamepadAxisDirection,
    deadzone: i16,
}

impl GamepadAxisInput {
    pub fn new(axis: GamepadAxisDirection, deadzone: f32) -> Self {
        Self {
            axis,
            deadzone: float_to_snorm16(deadzone),
        }
    }

    pub fn deadzone(&self) -> f32 {
        snorm16_to_float(self.deadzone)
    }

    pub fn axis(&self) -> GamepadAxisDirection {
        self.axis
    }
}


fn float_to_snorm16( v: f32 ) -> i16 {
    let a = if v >= 0.0 { v * 32767.0 + 0.5 } else { v * 32767.0 - 0.5 };
    let b = a.clamp( -32768.0, 32767.0 );
    b as i16
}
 
fn snorm16_to_float( v: i16 ) -> f32 {
    f32::max( (v as f32) / 32767.0, -1.0 )
}


#[cfg(test)]
mod test {
    use super::{float_to_snorm16, snorm16_to_float};

    #[test]
    fn float_packing() {
        let vals = [ 0.0, -0.1, 1.0, 0.74, 0.27, -0.696969 ];
        for a in vals {
            let b = float_to_snorm16( a );
            let c = snorm16_to_float( b );
            assert!( (a - c).abs() < 0.0002 ) // float packing is imprecise, but pretty close 
        }
    }
}