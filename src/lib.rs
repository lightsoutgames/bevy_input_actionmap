use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
};

use bevy::{
    input::{
        gamepad::{GamepadAxisType, GamepadEvent},
        InputSystem,
    },
    prelude::*,
};

#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "serialize")]
mod serialize;

mod input;
pub use input::*;

/// A set of [BindInput]s which must all be met to trigger an action
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct Binding {
    pub(crate) inputs: HashSet<BindInput>,
}

impl Binding {
    pub fn new(inputs: impl Into<BindInput>) -> Self {
        let mut me = Self::default();
        me.inputs.insert( inputs.into() );
        me
    }

    pub fn axis(axis: GamepadAxisDirection, deadzone: f32) -> Self {
        Self::default().with_axis(axis, deadzone)
    }

    pub fn with(mut self, input: impl Into<BindInput>) -> Self {
        self.inputs.insert( input.into() );
        self
    }

    pub fn with_axis(mut self, axis: GamepadAxisDirection, deadzone: f32) -> Self {
        self.inputs.insert( GamepadAxisInput::new(axis, deadzone).into() );
        self
    }
    
    /// Returns the strength of this combo if all its conditions are met,
    /// [None] otherwise.
    fn check<T>(&self,
        input_map: &InputMap<T>,
        keys: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>
    ) -> Option<f32>
        where T: Hash + Eq + Clone + Send
    {
        let (all_true, total_str) = self.inputs
            .iter()
            .map(|b|{
                match b {
                    BindInput::Keyboard( k ) => (keys.pressed(*k), 1.0),
                    BindInput::MouseButton(mb) => (mouse.pressed(*mb), 1.0),
                    BindInput::GamepadButton(b) => (
                        input_map.gamepad_pressed_buttons.pressed(*b), 
                        1.0
                    ),
                    BindInput::GamepadAxis(gpa) => {
                        if let Some( strength ) = input_map.gamepad_axis.get( &gpa.axis() ) {
                            (true, *strength)
                        }
                        else {
                            (false, 0.0)
                        }
                    },
                }
            })
            .fold((true, 0.0), |(acc_true, acc_str), (t, s)| {
                (acc_true & t,  acc_str + s)
            });

        if all_true {
            Some( total_str )
        }
        else {
            None
        }
    }
}

impl<B: Into<BindInput> + Sized> From<B> for Binding {
    fn from(value: B) -> Self {
        let mut inputs = HashSet::new();
        inputs.insert(value.into());
        Self { inputs }
    }
}

impl<B: Into<BindInput> + Sized> From<Vec<B>> for Binding {
    fn from(values: Vec<B>) -> Self {
        let mut inputs = HashSet::new();
        for v in values {
            inputs.insert(v.into());
        }
        Self { inputs }
    }
}


#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum GamepadAxisDirection {
    LeftStickXPositive,
    LeftStickXNegative,
    LeftStickYPositive,
    LeftStickYNegative,
    RightStickXPositive,
    RightStickXNegative,
    RightStickYPositive,
    RightStickYNegative,
    LeftZ,
    RightZ,
    Other( u8 )
}

impl GamepadAxisDirection {
    /// Returns the opposite axis, or [None] if the axis only 
    /// returns positive values
    pub fn opposite(&self) -> Option<Self> {
        use GamepadAxisDirection as GAD;
        match self {
            GAD::LeftStickXPositive => Some(GAD::LeftStickXNegative),
            GAD::LeftStickXNegative => Some(GAD::LeftStickXPositive),
            GAD::LeftStickYPositive => Some(GAD::LeftStickYNegative),
            GAD::LeftStickYNegative => Some(GAD::LeftStickYPositive),
            GAD::RightStickXPositive => Some(GAD::RightStickXNegative),
            GAD::RightStickXNegative => Some(GAD::RightStickXPositive),
            GAD::RightStickYPositive => Some(GAD::RightStickYPositive),
            GAD::RightStickYNegative => Some(GAD::RightStickYPositive),
            GAD::LeftZ => None,
            GAD::RightZ => None,
            GAD::Other(_) => None,
        }
    }
}

impl Binding {

    pub fn contains(&self, bind: impl Into<BindInput>) -> bool {
        let bind = bind.into();
        self.inputs.contains( &bind )
    }

    pub fn contains_axis(&self, axis: GamepadAxisDirection) -> bool {
        for bind in self.inputs.iter() {
            match bind {
                BindInput::GamepadAxis( gai ) => {
                    if gai.axis() == axis { return true }
                },
                _ => continue
            }
        }

        false
    }

    /// Searches a single binding for whether all of it's assigned keys are pressed
    fn key_pressed(&self, input: &ButtonInput<KeyCode>) -> bool {
        let keys = self.inputs.iter()
            .filter_map(|b|{
                match b {
                    BindInput::Keyboard(k) => Some(input.pressed(*k)),
                    _ => None,
                }
            })
            .collect::<Vec<bool>>();

        if keys.len() == 0 {
            false
        }
        else {
            keys.into_iter().fold(true, |it, acc| acc && it)
        }
    }

    /// Describes how many keys or buttons must be pressed at once to trigger this binding
    fn weight(&self) -> usize {
        self.inputs.len()
    }

    /// Returns a list of all [KeyCode]s this binding requires
    pub fn get_keycodes(&self) -> Vec<KeyCode> {
        self.inputs.iter()
            .filter_map(|b|{
                match b {
                    BindInput::Keyboard(k) => Some(*k),
                    _ => None,
                }
            })
            .collect()
    }

    pub fn get_mouse_buttons(&self) -> Vec<MouseButton> {
        self.inputs.iter()
            .filter_map(|b|{
                match b {
                    BindInput::MouseButton(k) => Some(*k),
                    _ => None,
                }
            })
            .collect()
    }

    pub fn get_gamepad_buttons(&self) -> Vec<GamepadButtonType> {
        self.inputs.iter()
            .filter_map(|b|{
                match b {
                    BindInput::GamepadButton(k) => Some(*k),
                    _ => None,
                }
            })
            .collect()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
/// An Action consists of many bindings of which any count as triggering it
pub struct Action {
    pub(crate) bindings: Vec<Binding>,
}

impl Action {
    pub fn new(bind: impl Into<Binding>) -> Self {
        Self {
            bindings: vec![ bind.into() ]
        }
    }

    pub fn with(mut self, bind: impl Into<Binding>) -> Self {
        self.bindings.push(bind.into());
        self
    }
}

/// A Bevy resource tracking bound `Action`s (including [`KeyCode`]s, [`GamepadButtonType`]s, and
/// [`GamepadAxisDirection`]s) generic over the application's action event type.
#[derive(Debug, Resource)]
pub struct InputMap<T> {
    pub(crate) actions: HashMap<T, Action>,
    gamepad_pressed_buttons: ButtonInput<GamepadButtonType>,
    gamepad_axis: HashMap<GamepadAxisDirection, f32>,
    raw_active: Vec<(T, Binding, f32)>,
    active: HashMap<T, f32>,
    just_active: HashMap<T, f32>,
    just_inactive: HashSet<T>,
    gamepads: HashSet<Gamepad>,
    wants_clear: bool,
}

impl<T> Default for InputMap<T> {
    fn default() -> Self {
        Self {
            actions: HashMap::new(),
            gamepad_pressed_buttons: ButtonInput::default(),
            gamepad_axis: HashMap::new(),
            raw_active: Vec::new(),
            active: HashMap::new(),
            just_active: HashMap::new(),
            just_inactive: HashSet::new(),
            gamepads: HashSet::new(),
            wants_clear: false,
        }
    }
}

impl<T> InputMap<T>
where
    T: Hash + Eq + Clone + Send + Sync + 'static,
{
    /// Adds an instance of the application's action type to the list of actions, but with no bound
    /// inputs.
    pub fn add_action(&mut self, key: T) -> &mut Self {
        self.actions.insert(key, default());
        self
    }

    /// Adds a given Binding to the given variant of the application's action type -- should the
    /// action not already be added, it is added automatically.
    pub fn bind<K: Into<T>, B: Into<Binding>>(&mut self, action: K, binding: B) -> &mut Self {
        let key = action.into();
        if !self.actions.contains_key(&key) {
            self.add_action(key.clone());
        }
        if let Some(actions) = self.actions.get_mut(&key) {
            actions.bindings.push(binding.into());
        }
        self
    }

    /// Returns whether a given action is currently triggered.
    pub fn active<K: Into<T>>(&self, key: K) -> bool {
        self.active.contains_key(&key.into())
    }

    /// Returns whether a given action has just been triggered.
    pub fn just_active<K: Into<T>>(&self, key: K) -> bool {
        self.just_active.contains_key(&key.into())
    }

    /// Returns whether a given action has just stopped being triggered.
    pub fn just_inactive<K: Into<T>>(&self, key: K) -> bool {
        self.just_inactive.contains(&key.into())
    }

    /// Returns the strength of an active triggered action for use with analog input.
    pub fn strength<K: Into<T>>(&self, key: K) -> f32 {
        if let Some(strength) = self.active.get(&key.into()) {
            *strength
        } else {
            0.
        }
    }

    /// Clears all triggered actions without changing configured bindings.
    pub fn clear(&mut self) {
        self.wants_clear = true;
        self.gamepad_pressed_buttons.clear();
        self.gamepad_axis.clear();
        self.raw_active.clear();
        self.active.clear();
        self.just_active.clear();
        self.just_inactive.clear();
    }

    /// System that clears specifically the maps of just active or just inactive actions
    fn clear_just_active_inactive(mut input_map: ResMut<InputMap<T>>) {
        input_map.just_active.clear();
        input_map.just_inactive.clear();
    }

    fn handle_input(
        mut input_map: ResMut<InputMap<T>>,
        keys: Res<ButtonInput<KeyCode>>,
        mouse: Res<ButtonInput<MouseButton>>,
    ) {
        let mut raw_active = vec![];
        for (val, bind) in input_map.actions.iter() {
            for combo in &bind.bindings {
                if let Some( strength ) = combo.check(&input_map, &keys, &mouse) {
                    raw_active.push( (val.clone(), combo.clone(), strength) )
                }
            }
        }

        input_map.raw_active = raw_active;
    }


    /// System that listens to [`GamepadEvent`]s to write into the raw inputs
    fn gamepad_state(mut gamepad_events: EventReader<GamepadEvent>, mut input: ResMut<InputMap<T>>) {
        use GamepadAxisDirection as GAD;
        use GamepadAxisType as GAT;
        for event in gamepad_events.read() {
            // println!(" => {event:?}");
            match &event {
                GamepadEvent::Connection( event ) => {
                    if event.connected() {
                        input.gamepads.insert( event.gamepad );
                    }
                    else {
                        input.gamepads.remove( &event.gamepad );
                    }
                },
                GamepadEvent::Button( event ) => {
                    if event.value > 0.5 {
                        input.gamepad_pressed_buttons.press(event.button_type);
                    } else {
                        input.gamepad_pressed_buttons.release(event.button_type);
                    }
                },
                GamepadEvent::Axis( event ) => {
                    let strength = event.value;
                    let mut direction = match event.axis_type {
                        GAT::LeftStickX  => GAD::LeftStickXPositive,
                        GAT::LeftStickY  => GAD::LeftStickYPositive,
                        GAT::RightStickX => GAD::RightStickXPositive,
                        GAT::RightStickY => GAD::RightStickYPositive,
                        GAT::LeftZ       => GAD::LeftZ,
                        GAT::RightZ      => GAD::RightZ,
                        GAT::Other(id)   => GAD::Other(id),
                    };

                    // flip direction, if the axis goes two ways and the is below zero
                    if strength < 0.0 {
                        direction = direction.opposite().unwrap_or( direction );
                    }

                    if strength != 0.0 {
                        input.gamepad_axis.insert(direction, strength);
                    } else {
                        input.gamepad_axis.remove(&direction);
                    }
                    if let Some( opposite ) = direction.opposite() {
                        input.gamepad_axis.remove(&opposite);
                    }
                },
            }
        }
    }

    /// System that prunes conflicting actions by prioritizing that with the higher weight.
    fn resolve_conflicts(mut input_map: ResMut<InputMap<T>>, input: Res<ButtonInput<KeyCode>>) {
        let mut active_resolve_conflicts = input_map.raw_active.clone();
        
        for (idx, outer) in input_map.raw_active.iter().enumerate() {
            for inner in (input_map.raw_active).as_slice()[idx + 1 .. ].iter() {
                let (_, outer_bind, _) = &outer;
                let (_, inner_bind, _) = &inner;

                let outer_weight = outer_bind.weight();
                let inner_weight = inner_bind.weight();
                
                if outer_weight == inner_weight {
                    continue;
                }
                let to_remove = if outer_weight > inner_weight { inner } else { outer };

                active_resolve_conflicts.retain(|v| v != to_remove);
            }
        }
        
        let just_active = active_resolve_conflicts.iter()
            .filter_map(|(val, bind, strength)| {
                match input_map.active.contains_key(&val) {
                    true => None,
                    false => Some((val, bind, strength))
                }
            })
            .collect::<Vec<_>>();
        
        for (val, bind, strength) in just_active {
            if bind.get_keycodes().is_empty() || bind.key_pressed(&input) {
                input_map.just_active.insert(val.clone(), *strength);
            }
        }
        
        let active = active_resolve_conflicts
            .iter()
            .map(|(val, _, strength)| (val.clone(), *strength))
            .collect::<Vec<(T, f32)>>();

        let prev_active = input_map.active.clone();
        for prev in prev_active.keys() {
            let active_binds = active.iter().find(|(val, _)| *val == *prev );
            if active_binds.is_none() {
                input_map.just_inactive.insert(prev.clone());
            }
        }

        input_map.active.clear();
        input_map.active.extend( active );
        input_map.raw_active.clear();
    }

    /// System that assists in clearing the input by modifying the actual [`Input`] resource interal
    /// to Bevy
    fn clear_wants_clear(mut input_map: ResMut<InputMap<T>>, mut input: ResMut<ButtonInput<KeyCode>>) {
        if input_map.wants_clear {
            input.clear();
            let mut v = vec![];
            for i in input.get_pressed().cloned() {
                v.push(i);
            }
            for i in v {
                input.reset(i);
            }
        }

        input_map.wants_clear = false;
    }
}

pub struct ActionPlugin<'a, T>(std::marker::PhantomData<&'a T>);

impl<'a, T> Default for ActionPlugin<'a, T> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, SystemSet)]
pub struct UpdateSet;

#[derive(Clone, Debug, PartialEq, Eq, Hash, SystemSet)]
pub struct ResolveConflictsSet;



impl<T> Plugin for ActionPlugin<'static, T>
where
    InputMap<T>: Default,
    T: Hash + Eq + Clone + Send + Sync + 'static
{
    fn build(&self, app: &mut App) {
        app
            .init_resource::<InputMap<T>>()
            // Clear the `just_active` and `just_inactive` maps at the start of every iteration of the
            // application's main loop to ensure that there are no false positives
            .add_systems(PostUpdate, InputMap::<T>::clear_just_active_inactive)
            // Register keyboard, gamepad button, and mouse button input
            .add_systems(PreUpdate, (InputMap::<T>::gamepad_state, InputMap::<T>::handle_input)
                .after(InputSystem)
                .chain_ignore_deferred()
                .before(ResolveConflictsSet)
            )
            // Resolve all conflicts based on weight
            .add_systems(PreUpdate, InputMap::<T>::resolve_conflicts
                .in_set(ResolveConflictsSet)
            )
            // And clear the inputs if requested
            .add_systems(PostUpdate, InputMap::<T>::clear_wants_clear);
    }
}



// this bit tells the compiler to import the code sections of README.md 
// as this type's documentation.
//
// This means running `cargo test` will ensure the example code compiles.
#[doc = include_str!("../README.md")]
#[cfg(doctest)]
struct ReadmeDoctests;