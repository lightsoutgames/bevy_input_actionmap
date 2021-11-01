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

#[cfg(test)]
#[cfg(feature = "serialize")]
mod serialize;
#[cfg(feature = "serialize")]
pub use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
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

    fn weight(&self) -> usize {
        max(self.keys.len(), self.gamepad_buttons.len())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct Action {
    //add a copy of T here
    pub bindings: Vec<Binding>, //made pub so i can access it in example will need methods to add and remove or at the very least get &bindings if you want to serialize to be implemented by the user
}

impl Action {
    fn key_pressed(&self, input: &Res<Input<KeyCode>>) -> Option<Binding> {
        let mut bindings = self
            .bindings
            .iter()
            .filter(|it| it.key_pressed(&input))
            .cloned()
            .collect::<Vec<Binding>>();
        bindings.sort_by(|v1, v2| (v1.weight().partial_cmp(&v2.weight()).unwrap()));
        bindings.last().cloned()
    }

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

pub struct InputMap<T> {
    actions: HashMap<T, Action>,//change this to being a new struct Bindings that does the same thing but frees up the name for Action to be use in none uniqe collections
                                //could also just leave the a redundent copy of T in actions
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
    pub fn add_action(&mut self, key: T) -> &mut Self {
        self.actions.insert(key, Default::default());
        self
    }

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

    pub fn active<K: Into<T>>(&self, key: K) -> bool {
        self.active.contains_key(&key.into())
    }

    pub fn just_active<K: Into<T>>(&self, key: K) -> bool {
        self.just_active.contains_key(&key.into())
    }

    pub fn just_inactive<K: Into<T>>(&self, key: K) -> bool {
        self.just_inactive.contains(&key.into())
    }

    pub fn strength<K: Into<T>>(&self, key: K) -> f32 {
        if let Some(strength) = self.active.get(&key.into()) {
            *strength
        } else {
            0.
        }
    }

    pub fn clear(&mut self) {
        self.wants_clear = true;
        self.pressed_buttons.clear();
        self.gamepad_axis.clear();
        self.raw_active.clear();
        self.active.clear();
        self.just_active.clear();
        self.just_inactive.clear();
    }

    fn clear_just_active_inactive(mut input_map: ResMut<InputMap<T>>)
    where
        T: 'static + Debug,
    {
        input_map.just_active.clear();
        input_map.just_inactive.clear();
    }

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
                            input.gamepad_axis.remove(&opposite);
                        } else {
                            input.gamepad_axis.remove(&direction);
                            input.gamepad_axis.remove(&opposite);
                        }
                    }
                }
            }
        }
    }

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
    //could we change Actions to contain a copy of T so we pass in a Vec<Action>
    pub fn add_actions(&mut self, actions : Vec<(T, Vec<Binding>)>){
        for (action, bindings) in actions {
            self.add_action(action.clone());
            for binding in bindings{
                self.bind(action.clone(), binding);
            }
        }
    }

    pub fn set_actions(&mut self, actions : Vec<(T, Vec<Binding>)>){
        self.actions.clear();
        self.add_actions(actions)
    }

    pub fn get_actions(&self) -> &HashMap<T, Action>{
        &self.actions
    }

    pub fn get_mut_actions(&mut self) -> &mut HashMap<T, Action>{
        &mut self.actions
    }

    pub fn get_action<K : Into<T>>(&self, key : K) -> Option<&Action>{
        self.actions.get(&key.into())
    }

    pub fn get_mut_action<K : Into<T>>(&mut self, key : K) -> Option<&mut Action>{
        self.actions.get_mut(&key.into())
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
            .add_system_to_stage(
                CoreStage::First,
                InputMap::<T>::clear_just_active_inactive.system(),
            )
            .add_system_to_stage(
                CoreStage::PreUpdate,
                InputMap::<T>::key_input
                    .system()
                    .label(UPDATE_STATES_LABEL)
                    .after(InputSystem),
            )
            .add_system_to_stage(
                CoreStage::PreUpdate,
                InputMap::<T>::gamepad_state
                    .system()
                    .label(UPDATE_STATES_LABEL)
                    .after(InputSystem),
            )
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
            .add_system_to_stage(
                CoreStage::PreUpdate,
                InputMap::<T>::resolve_conflicts
                    .system()
                    .label(RESOLVE_CONFLICTS_LABEL),
            )
            .add_system_to_stage(
                CoreStage::PostUpdate,
                InputMap::<T>::clear_wants_clear.system(),
            );
    }
}
