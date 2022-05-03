use std::fmt::Formatter;
use std::hash::Hash;
use std::marker::PhantomData;

use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::InputMap;

impl<T: Serialize> Serialize for InputMap<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.actions.len()))?;
        for (k, v) in &self.actions {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

impl<'de, T> Deserialize<'de> for InputMap<T>
where
    T: Eq + Hash + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let visitor = InputMapVisitor::<T>::new();

        deserializer.deserialize_map(visitor)
    }
}

struct InputMapVisitor<K> {
    marker: PhantomData<fn() -> InputMap<K>>,
}

impl<T> InputMapVisitor<T> {
    pub(crate) fn new() -> Self {
        InputMapVisitor {
            marker: PhantomData,
        }
    }
}

impl<'de, T> Visitor<'de> for InputMapVisitor<T>
where
    T: Deserialize<'de> + Eq + Hash,
{
    type Value = InputMap<T>;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("Expected a valid Input mapping")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map = InputMap::<T>::default();

        // While there are entries remaining in the input, add them
        // into our map.
        while let Some((key, value)) = access.next_entry()? {
            map.actions.insert(key, value);
        }

        Ok(map)
    }
}
#[derive(Hash, PartialEq, Eq, Serialize, Deserialize, Debug, Copy, Clone)]
enum TestAction {
    Select,
    SuperSelect,
    AwesomeSuperSelect,
}

#[test]
fn test_serialize_to_string() {
    let mut map = InputMap::<TestAction>::default();
    map.bind(
        TestAction::Select,
        vec![
            bevy::prelude::KeyCode::Space,
            bevy::prelude::KeyCode::LControl,
        ],
    );
    map.bind(
        TestAction::Select,
        vec![
            bevy::prelude::GamepadButtonType::North,
            bevy::prelude::GamepadButtonType::LeftThumb,
        ],
    );
    map.bind(
        TestAction::AwesomeSuperSelect,
        bevy::prelude::GamepadButtonType::North,
    );
    let serialized = ron::to_string(&map).expect("Failed serialization");
    let deserialized: InputMap<TestAction> =
        ron::from_str(&serialized).expect("Failed deserialization");
    assert_eq!(map.actions, deserialized.actions);
}
