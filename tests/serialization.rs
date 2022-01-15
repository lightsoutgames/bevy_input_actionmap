use crate::TestAction::{AwesomeSuperSelect, Select};
use bevy::prelude::*;
use bevy_input_actionmap::InputMap;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Hash, PartialEq, Eq, Serialize, Deserialize, Debug, Copy, Clone)]
enum TestAction {
    Select,
    SuperSelect,
    AwesomeSuperSelect,
}

#[test]
fn test_serialize_to_string() {
    let mut map = InputMap::<TestAction>::default();
    map.bind(Select, vec![KeyCode::Space, KeyCode::LControl]);
    map.bind(
        Select,
        vec![GamepadButtonType::North, GamepadButtonType::LeftThumb],
    );
    map.bind(AwesomeSuperSelect, GamepadButtonType::North);
    let serialized = ron::to_string(&map).expect("Failed serialization");
    let deserialized: InputMap<TestAction> =
        ron::from_str(&serialized).expect("Failed deserialization");
    assert_eq!(map.actions, deserialized.actions);
    println!("{:?}", map.actions);
}
