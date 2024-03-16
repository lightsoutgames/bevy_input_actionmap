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
    pub fn new<B>(inputs: &[ B ]) -> Self
    where
        B: Into<BindInput> + Clone
     {
        let mut me = Self::default();
        for i in inputs.into_iter() {
            me.inputs.insert( i.clone().into() );
        }
        me
    }

    pub fn with(mut self, input: impl Into<BindInput>) -> Self {
        self.inputs.insert( input.into() );
        self
    }
    
    /// Returns the strength of this combo if all its conditions are met,
    /// [None] otherwise.
    fn check<T>(&self,
        input_map: &InputMap<T>,
        keys: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>
    ) -> Option<f32>
        where T: Hash + Eq + Clone + Send + Sync + Debug + 'static 
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

    /// Searches a single binding for whether all of it's assigned keys are pressed
    fn mouse_button_pressed(&self, input: &ButtonInput<MouseButton>) -> bool {
        let keys = self.inputs.iter()
            .filter_map(|b|{
                match b {
                    BindInput::MouseButton(m) => Some(input.pressed(*m)),
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

    /// Searches a single binding for whether all of it's assigned gamepad buttons are pressed
    fn gamepad_button_pressed(&self, buttons: &ButtonInput<GamepadButtonType>) -> bool {
        let keys = self.inputs.iter()
            .filter_map(|b|{
                match b {
                    BindInput::GamepadButton(b) => Some(buttons.pressed(*b)),
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

    /// Searches a single binding for whether all of it's assigned gamepad axes are pressed
    fn gamepad_axis_changed(&self, input: &HashMap<GamepadAxisDirection, f32>) -> bool {
        let keys = self.inputs.iter()
            .filter_map(|b|{
                match b {
                    BindInput::GamepadAxis( gba ) => Some(
                        input.contains_key(&gba.axis()) 
                            && input.get(&gba.axis()).unwrap().abs() > gba.deadzone()
                    ),
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

    // fn check_against()

    /// Searches all keypress Bindings for those being actively triggered and returns Some(Binding)
    /// of the Binding in question. Should multiple bindings be triggered at once, the one with the
    /// greatest [`Binding::weight`] is returned. Should no bindings be triggered, None is returned.
    fn key_pressed(&self, input: &Res<ButtonInput<KeyCode>>) -> Option<Binding> {
        let mut bindings = self
            .bindings
            .iter()
            .filter(|it| it.key_pressed(input))
            .cloned()
            .collect::<Vec<Binding>>();
        bindings.sort_by(|v1, v2| (v1.weight().partial_cmp(&v2.weight()).unwrap()));
        bindings.last().cloned()
    }

    /// Searches all mouse button Bindings for those being actively triggered and returns
    /// Some(Binding) of the Binding in question. Should multiple bindings be triggered at once, the
    /// one with the greatest [`Binding::weight`] is returned. Should no bindings be triggered, None
    /// is returned.
    fn mouse_button_pressed(&self, buttons: &ButtonInput<MouseButton>) -> Option<Binding> {
        let mut bindings = self
            .bindings
            .iter()
            .filter(|it| it.mouse_button_pressed(buttons))
            .cloned()
            .collect::<Vec<Binding>>();
        bindings.sort_by(|v1, v2| (v1.weight().partial_cmp(&v2.weight()).unwrap()));
        bindings.last().cloned()
    }

    /// Searches all gamepad button Bindings for those being actively triggered and returns
    /// Some(Binding) of the Binding in question. Should multiple bindings be triggered at once, the
    /// one with the greatest [`Binding::weight`] is returned. Should no bindings be triggered, None
    /// is returned.
    fn button_pressed(&self, buttons: &ButtonInput<GamepadButtonType>) -> Option<Binding> {
        let mut bindings = self
            .bindings
            .iter()
            .filter(|it| it.gamepad_button_pressed(buttons))
            .cloned()
            .collect::<Vec<Binding>>();
        bindings.sort_by(|v1, v2| (v1.weight().partial_cmp(&v2.weight()).unwrap()));
        bindings.last().cloned()
    }

    /// Searches all gamepad axis Bindings for those being actively triggered and returns
    /// Some((Binding, f32)) of the Binding and the strength of the axis pull in question. Should
    /// multiple bindings be triggered at once, the one with the greatest [`Binding::weight`] is
    /// returned. Should no bindings be triggered, None is returned.
    fn gamepad_axis_changed(
        &self,
        directions: &HashMap<GamepadAxisDirection, f32>,
    ) -> Option<(Binding, f32)> {
        let mut axis_bindings = self
            .bindings
            .iter()
            .filter(|it| it.gamepad_axis_changed(directions))
            .cloned()
            .collect::<Vec<Binding>>();
        axis_bindings.sort_by(|v1, v2| (v1.weight().partial_cmp(&v2.weight()).unwrap()));
        let binding = axis_bindings.last().cloned();
        if let Some(binding) = binding {
            let mut strength = 0.0;
            for (k, v) in directions.iter() {
                if binding.contains_axis(*k) {
                    strength += v;
                }
            }
            strength /= axis_bindings.len() as f32;
            Some((binding, strength))
        } else {
            None
        }
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
    T: Hash + Eq + Clone + Send + Sync + Debug + 'static,
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
    fn clear_just_active_inactive(mut input_map: ResMut<InputMap<T>>)
    where
        T: 'static + Debug,
    {
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


    /// System that listens to pressed [`KeyCodes`] to map to the configured actions
    fn key_input(input: Res<ButtonInput<KeyCode>>, mut input_map: ResMut<InputMap<T>>)
    where
        T: 'static + Debug,
    {
        let mut raw_active = input_map
            .actions
            .iter()
            .map(|a| (a.0, a.1.key_pressed(&input)))
            .filter(|v| v.1.is_some())
            .map(|v| (v.0.clone(), v.1.unwrap(), 1.))
            .collect::<Vec<(T, Binding, f32)>>();
        input_map.raw_active.append(&mut raw_active);
    }

    /// System that listens to pressed [`KeyCodes`] to map to the configured actions
    fn mouse_buttons(input: Res<ButtonInput<MouseButton>>, mut input_map: ResMut<InputMap<T>>)
    where
        T: 'static + Debug,
    {
        let mut raw_active = input_map
            .actions
            .iter()
            .map(|a| (a.0, a.1.mouse_button_pressed(&input)))
            .filter(|v| v.1.is_some())
            .map(|v| (v.0.clone(), v.1.unwrap(), 1.))
            .collect::<Vec<(T, Binding, f32)>>();
        input_map.raw_active.append(&mut raw_active);
    }


    /// System that listens to [`GamepadEvent`]s to write into the raw inputs
    fn gamepad_state(mut gamepad_events: EventReader<GamepadEvent>, mut input: ResMut<InputMap<T>>)
    where
        T: 'static + Debug,
    {
        use GamepadAxisDirection as GAD;
        use GamepadAxisType as GAT;
        for event in gamepad_events.read() {
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

    /// System that updates the gamepad button inputs in the [`InputMap`]
    fn gamepad_button_input(mut input_map: ResMut<InputMap<T>>)
    where
        T: 'static + Debug,
    {
        let mut raw_active = input_map
            .actions
            .iter()
            .map(|(key, act)| (key, act.button_pressed(&input_map.gamepad_pressed_buttons)))
            .filter(|(_key, act)| act.is_some())
            .map(|(key, act)| {
                (key.clone(), act.unwrap(), 1.0)
            })
            .collect::<Vec<(T, Binding, f32)>>();
        input_map.raw_active.append(&mut raw_active);
    }

    /// System that updates the gamepad axis inputs in the [`InputMap`]
    fn gamepad_axis_input(mut input_map: ResMut<InputMap<T>>)
    where
        T: 'static + Debug,
    {
        let mut raw_active = input_map
            .actions
            .iter()
            .map(|a| (a.0, a.1.gamepad_axis_changed(&input_map.gamepad_axis)))
            .filter(|v| v.1.is_some())
            .map(|v| {
                let rv = v.1.unwrap();
                (v.0.clone(), rv.0, rv.1)
            })
            .collect::<Vec<(T, Binding, f32)>>();
        input_map.raw_active.append(&mut raw_active);
    }

    /// System that prunes conflicting actions by prioritizing that with the higher weight.
    fn resolve_conflicts(mut input_map: ResMut<InputMap<T>>, input: Res<ButtonInput<KeyCode>>)
    where
        T: 'static + Debug,
    {
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
    fn clear_wants_clear(mut input_map: ResMut<InputMap<T>>, mut input: ResMut<ButtonInput<KeyCode>>)
    where
        T: 'static + Debug,
    {
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
    T: Hash + Eq + Clone + Send + Sync + Debug + 'static
{
    fn build(&self, app: &mut App) {
        app
            .init_resource::<InputMap<T>>()
            // Clear the `just_active` and `just_inactive` maps at the start of every iteration of the
            // application's main loop to ensure that there are no false positives
            .add_systems(PostUpdate, InputMap::<T>::clear_just_active_inactive)
            // // Register keyboard, gamepad button, and mouse button input
            // .add_systems(PreUpdate,
            //     (InputMap::<T>::key_input, InputMap::<T>::gamepad_state, InputMap::<T>::mouse_buttons)
            //         .in_set(UpdateSet)
            //         .after(InputSystem)
            // )
            // // Register gamepad inputs
            // .add_systems(PreUpdate, InputMap::<T>::gamepad_state
            //         .in_set(UpdateSet).after(InputSystem),
            // )
            // // Then map those gamepad inputs to the correct actions
            // .add_systems(PreUpdate, 
            //     (InputMap::<T>::gamepad_button_input, InputMap::<T>::gamepad_axis_input)
            //         .after(UpdateSet).before(ResolveConflictsSet)
            // )
            .add_systems(PreUpdate, (InputMap::<T>::gamepad_state, InputMap::<T>::handle_input)
                .after(InputSystem)
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
