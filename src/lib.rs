use std::{
    cmp::max,
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
};

use bevy::{
    input::{
        gamepad::{GamepadAxisType, GamepadEvent, GamepadEventType},
        InputSystem,
    },
    prelude::*,
};

#[derive(Clone, Debug, Default, PartialEq)]
/// A single binding consisting of sets of applicable key presses or gamepad activity meant to
/// be used in tandem.
pub struct Binding {
    keys: HashSet<KeyCode>,
    gamepad_buttons: HashSet<GamepadButtonType>,
    gamepad_axis_directions: HashSet<GamepadAxisDirection>,
    deadzone: f32,
}

impl From<KeyCode> for Binding {
    fn from(keycode: KeyCode) -> Self {
        let mut keys = HashSet::new();
        keys.insert(keycode);
        Self {
            keys,
            ..Default::default()
        }
    }
}

impl From<Vec<KeyCode>> for Binding {
    fn from(keys: Vec<KeyCode>) -> Self {
        let mut set = HashSet::new();
        for key in keys {
            set.insert(key);
        }
        Self {
            keys: set,
            ..Default::default()
        }
    }
}

impl From<GamepadButtonType> for Binding {
    fn from(button: GamepadButtonType) -> Self {
        let mut buttons = HashSet::new();
        buttons.insert(button);
        Self {
            gamepad_buttons: buttons,
            ..Default::default()
        }
    }
}

impl From<Vec<GamepadButtonType>> for Binding {
    fn from(buttons: Vec<GamepadButtonType>) -> Self {
        let mut set = HashSet::new();
        for button in buttons {
            set.insert(button);
        }
        Self {
            gamepad_buttons: set,
            ..Default::default()
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum GamepadAxisDirection {
    LeftStickXPositive,
    LeftStickXNegative,
    LeftStickYPositive,
    LeftStickYNegative,
    RightStickXPositive,
    RightStickXNegative,
    RightStickYPositive,
    RightStickYNegative,
    DPadXPositive,
    DPadXNegative,
    DPadYPositive,
    DPadYNegative,
}

impl From<GamepadAxisDirection> for Binding {
    fn from(gamepad_axis_direction: GamepadAxisDirection) -> Self {
        let mut gamepad_axis_directions = HashSet::new();
        gamepad_axis_directions.insert(gamepad_axis_direction);
        Self {
            gamepad_axis_directions,
            ..Default::default()
        }
    }
}

impl Binding {
    /// Searches a single binding for whether all of it's assigned keys are pressed
    fn key_pressed(&self, input: &Res<Input<KeyCode>>) -> bool {
        if self.keys.is_empty() {
            false
        } else {
            self.keys
                .iter()
                .map(|it| input.pressed(*it))
                .fold(true, |it, acc| acc && it)
        }
    }

    /// Searches a single binding for whether all of it's assigned gamepad buttons are pressed
    fn button_pressed(&self, buttons: &HashMap<GamepadButtonType, f32>) -> bool {
        if self.gamepad_buttons.is_empty() {
            false
        } else {
            self.gamepad_buttons
                .iter()
                .map(|it| {
                    buttons.contains_key(it) && buttons.get(it).unwrap().abs() > self.deadzone
                })
                .fold(true, |it, acc| acc && it)
        }
    }

    /// Searches a single binding for whether all of it's assigned gamepad axes are pressed
    fn gamepad_axis_changed(&self, input: &HashMap<GamepadAxisDirection, f32>) -> bool {
        if self.gamepad_axis_directions.is_empty() {
            false
        } else {
            self.gamepad_axis_directions
                .iter()
                .map(|it| input.contains_key(it) && input.get(it).unwrap().abs() > self.deadzone)
                .fold(true, |it, acc| acc && it)
        }
    }

    /// Describes how many keys or buttons must be pressed at once to trigger this binding
    fn weight(&self) -> usize {
        max(self.keys.len(), self.gamepad_buttons.len())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
/// An Action consists of many bindings of which any caount as triggering it
pub struct Action {
    bindings: Vec<Binding>,
}

impl Action {
    /// Searches all keypress Bindings for those being actively triggered and returns Some(Binding)
    /// of the Binding in question. Should multiple bindings be triggered at once, the one with the
    /// greatest [`Binding::weight`] is returned. Should no bindings be triggered, None is returned.
    fn key_pressed(&self, input: &Res<Input<KeyCode>>) -> Option<Binding> {
        let mut bindings = self
            .bindings
            .iter()
            .filter(|it| it.key_pressed(input))
            .cloned()
            .collect::<Vec<Binding>>();
        bindings.sort_by(|v1, v2| (v1.weight().partial_cmp(&v2.weight()).unwrap()));
        bindings.last().cloned()
    }

    /// Searches all gamepad button Bindings for those being actively triggered and returns
    /// Some(Binding) of the Binding in question. Should multiple bindings be triggered at once, the
    /// one with the greatest [`Binding::weight`] is returned. Should no bindings be triggered, None
    /// is returned.
    fn button_pressed(&self, buttons: &HashMap<GamepadButtonType, f32>) -> Option<(Binding, f32)> {
        let mut bindings = self
            .bindings
            .iter()
            .filter(|it| it.button_pressed(buttons))
            .cloned()
            .collect::<Vec<Binding>>();
        bindings.sort_by(|v1, v2| (v1.weight().partial_cmp(&v2.weight()).unwrap()));
        let binding = bindings.last().cloned();
        if let Some(binding) = binding {
            let mut strength = 0.;
            for (k, v) in buttons.iter() {
                if binding.gamepad_buttons.contains(k) {
                    strength += v;
                }
            }
            strength /= binding.gamepad_buttons.len() as f32;
            Some((binding, strength))
        } else {
            None
        }
    }

    /// Searches all gamepad axis Bindings for those being actively triggered and returns
    /// Some((Binding, f32)) of the Binding and the strength of the axis pull in question. Should
    /// multiple bindings be triggered at once, the one with the greatest [`Binding::weight`] is
    /// returned. Should no bindings be triggered, None is returned.
    fn gamepad_axis_changed(
        &self,
        directions: &HashMap<GamepadAxisDirection, f32>,
    ) -> Option<(Binding, f32)> {
        let mut bindings = self
            .bindings
            .iter()
            .filter(|it| it.gamepad_axis_changed(directions))
            .cloned()
            .collect::<Vec<Binding>>();
        bindings.sort_by(|v1, v2| (v1.weight().partial_cmp(&v2.weight()).unwrap()));
        let binding = bindings.last().cloned();
        if let Some(binding) = binding {
            let mut strength = 0.;
            for (k, v) in directions.iter() {
                if binding.gamepad_axis_directions.contains(k) {
                    strength += v;
                }
            }
            strength /= binding.gamepad_axis_directions.len() as f32;
            Some((binding, strength))
        } else {
            None
        }
    }
}

/// A Bevy resource tracking bound `Action`s (including [`KeyCode`]s, [`GamepadButtonType`]s, and
/// [`GamepadAxisDirection`]s) generic over the application's action event type.
pub struct InputMap<T> {
    actions: HashMap<T, Action>,
    pressed_buttons: HashMap<GamepadButtonType, f32>,
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
            pressed_buttons: HashMap::new(),
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
    T: Hash + Eq + Clone + Send + Sync,
{
    /// Adds an instance of the application's action type to the list of actions, but with no bound
    /// inputs.
    pub fn add_action(&mut self, key: T) -> &mut Self {
        self.actions.insert(key, Default::default());
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

    /// Performs a binding as with the [`InputMap::bind`] method, but includes a deadzone for which
    /// the action much exceed in order to register eg. with analog buttons and axes.
    pub fn bind_with_deadzone<K: Into<T>, B: Into<Binding>>(
        &mut self,
        key: K,
        binding: B,
        deadzone: f32,
    ) -> &mut Self {
        let key = key.into();
        if !self.actions.contains_key(&key) {
            self.add_action(key.clone());
        }
        if let Some(actions) = self.actions.get_mut(&key) {
            let mut binding = binding.into();
            binding.deadzone = deadzone;
            actions.bindings.push(binding);
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
        self.pressed_buttons.clear();
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

    /// System that listens to pressed [`KeyCodes`] to map to the configured actions
    fn key_input(input: Res<Input<KeyCode>>, mut input_map: ResMut<InputMap<T>>)
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

    /// System that listens to [`GamepadEvent`]s to write into the raw inputs
    fn gamepad_state(mut gamepad_events: EventReader<GamepadEvent>, mut input: ResMut<InputMap<T>>)
    where
        T: 'static + Debug,
    {
        for event in gamepad_events.iter() {
            match &event {
                GamepadEvent(gamepad, GamepadEventType::Connected) => {
                    input.gamepads.insert(*gamepad);
                }
                GamepadEvent(gamepad, GamepadEventType::Disconnected) => {
                    input.gamepads.remove(gamepad);
                }
                GamepadEvent(_, GamepadEventType::ButtonChanged(button, strength)) => {
                    if strength > &0. {
                        input.pressed_buttons.insert(*button, *strength);
                    } else {
                        input.pressed_buttons.remove(button);
                    }
                }
                GamepadEvent(_, GamepadEventType::AxisChanged(axis_type, strength)) => {
                    use GamepadAxisDirection::*;
                    let positive = *strength >= 0.;
                    let direction = match axis_type {
                        GamepadAxisType::LeftStickX => Some(if positive {
                            (LeftStickXPositive, LeftStickXNegative)
                        } else {
                            (LeftStickXNegative, LeftStickXPositive)
                        }),
                        GamepadAxisType::LeftStickY => Some(if positive {
                            (LeftStickYPositive, LeftStickYNegative)
                        } else {
                            (LeftStickYNegative, LeftStickYPositive)
                        }),
                        GamepadAxisType::RightStickX => Some(if positive {
                            (RightStickXPositive, RightStickXNegative)
                        } else {
                            (RightStickXNegative, RightStickXPositive)
                        }),
                        GamepadAxisType::RightStickY => Some(if positive {
                            (RightStickYPositive, RightStickYNegative)
                        } else {
                            (RightStickYNegative, RightStickYPositive)
                        }),
                        GamepadAxisType::DPadX => Some(if positive {
                            (DPadXPositive, DPadXNegative)
                        } else {
                            (DPadXNegative, DPadXPositive)
                        }),
                        GamepadAxisType::DPadY => Some(if positive {
                            (DPadYPositive, DPadYNegative)
                        } else {
                            (DPadYNegative, DPadYPositive)
                        }),
                        _ => None,
                    };
                    if let Some((direction, opposite)) = direction {
                        if *strength != 0. {
                            input.gamepad_axis.insert(direction, *strength);
                        } else {
                            input.gamepad_axis.remove(&direction);
                        }
                        input.gamepad_axis.remove(&opposite);
                    }
                }
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
            .map(|a| (a.0, a.1.button_pressed(&input_map.pressed_buttons)))
            .filter(|v| v.1.is_some())
            .map(|v| {
                let press = v.1.unwrap();
                (v.0.clone(), press.0, press.1)
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
    fn resolve_conflicts(mut input_map: ResMut<InputMap<T>>, input: Res<Input<KeyCode>>)
    where
        T: 'static + Debug,
    {
        let mut active_resolve_conflicts = input_map.raw_active.clone();
        for (outer_action, outer_binding, outer_strength) in &input_map.raw_active {
            for (inner_action, inner_binding, inner_strength) in &input_map.raw_active {
                if outer_action == inner_action {
                    continue;
                }
                let weight = if !outer_binding.keys.is_empty() && !inner_binding.keys.is_empty() {
                    let intersection = outer_binding.keys.intersection(&inner_binding.keys);
                    intersection.count()
                } else if !outer_binding.gamepad_buttons.is_empty()
                    && !inner_binding.gamepad_buttons.is_empty()
                {
                    let intersection = outer_binding
                        .gamepad_buttons
                        .intersection(&inner_binding.gamepad_buttons);
                    intersection.count()
                } else {
                    0
                };
                if weight == outer_binding.weight() {
                    continue;
                }
                if weight != 0 {
                    let to_remove = if weight < outer_binding.weight() {
                        (inner_action.clone(), inner_binding.clone(), *inner_strength)
                    } else {
                        (outer_action.clone(), outer_binding.clone(), *outer_strength)
                    };
                    active_resolve_conflicts.retain(|v| *v != to_remove);
                }
            }
        }
        let just_active = active_resolve_conflicts
            .iter()
            .filter(|v| !input_map.active.contains_key(&v.0))
            .map(|v| (v.0.clone(), v.1.clone(), v.2))
            .collect::<Vec<(T, Binding, f32)>>();
        for v in just_active {
            if v.1.keys.is_empty() || v.1.keys.iter().any(|v| input.just_pressed(*v)) {
                input_map.just_active.insert(v.0, v.2);
            }
        }
        let active = active_resolve_conflicts
            .iter()
            .map(|v| (v.0.clone(), v.2))
            .collect::<Vec<(T, f32)>>();
        let prev_active = input_map.active.clone();
        for k in prev_active.keys() {
            let binding = active.iter().find(|v| v.0 == *k);
            if binding.is_none() {
                input_map.just_inactive.insert(k.clone());
            }
        }
        input_map.active.clear();
        for v in active {
            input_map.active.insert(v.0, v.1);
        }
        input_map.raw_active.clear();
    }

    /// System that assists in clearing the input by modifying the actual [`Input`] resource interal
    /// to Bevy
    fn clear_wants_clear(mut input_map: ResMut<InputMap<T>>, mut input: ResMut<Input<KeyCode>>)
    where
        T: 'static + Debug,
    {
        if input_map.wants_clear {
            input.update();
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

impl<'a, T> Plugin for ActionPlugin<'a, T>
where
    InputMap<T>: Default,
    T: Hash + Eq + Clone + Send + Sync + Debug,
    'a: 'static,
{
    fn build(&self, app: &mut AppBuilder) {
        const UPDATE_STATES_LABEL: &str = "UPDATE_STAES";
        const RESOLVE_CONFLICTS_LABEL: &str = "RESOLVE_CONFLICTS";
        app.init_resource::<InputMap<T>>()
            // Clear the `just_active` and `just_inactive` maps at the start of every iteration of the
            // application's main loop to ensure that there are no false positives
            .add_system_to_stage(
                CoreStage::First,
                InputMap::<T>::clear_just_active_inactive.system(),
            )
            // Register keyboard input
            .add_system_to_stage(
                CoreStage::PreUpdate,
                InputMap::<T>::key_input
                    .system()
                    .label(UPDATE_STATES_LABEL)
                    .after(InputSystem),
            )
            // Register gamepad inputs
            .add_system_to_stage(
                CoreStage::PreUpdate,
                InputMap::<T>::gamepad_state
                    .system()
                    .label(UPDATE_STATES_LABEL)
                    .after(InputSystem),
            )
            // Then map those gamepad inputs to the correct actions
            .add_system_to_stage(
                CoreStage::PreUpdate,
                InputMap::<T>::gamepad_button_input
                    .system()
                    .after(UPDATE_STATES_LABEL)
                    .before(RESOLVE_CONFLICTS_LABEL),
            )
            .add_system_to_stage(
                CoreStage::PreUpdate,
                InputMap::<T>::gamepad_axis_input
                    .system()
                    .after(UPDATE_STATES_LABEL)
                    .before(RESOLVE_CONFLICTS_LABEL),
            )
            // Resolve all conflicts based on weight
            .add_system_to_stage(
                CoreStage::PreUpdate,
                InputMap::<T>::resolve_conflicts
                    .system()
                    .label(RESOLVE_CONFLICTS_LABEL),
            )
            // And clear the inputs if requested
            .add_system_to_stage(
                CoreStage::PostUpdate,
                InputMap::<T>::clear_wants_clear.system(),
            );
    }
}
