const ACTION_SELECT: &str = "SELECT";
const ACTION_SUPER_SELECT: &str = "SUPER_SELECT";
const ACTION_AWESOME_SUPER_SELECT: &str = "AWESOME_SUPER_SELECT";
const TEST_PATH : &str = "test.keymap";
use std::hash::Hash;
use std::fmt::Debug;
use bevy::prelude::*;
use serde::{Serialize, de::DeserializeOwned};
use crate::*;

#[derive(Hash, PartialEq, Eq, Clone, Serialize, serde::Deserialize, Debug)]
enum TestAction {
    Select,
    SuperSelect,
    AwesomeSuperSelect,
}
fn test_binding<T>(to_test : T) where T : Into<Binding>{
    let binding : Binding = to_test.into();
    let serialized = ron::to_string(&binding).expect("Failed to serialize binding");
    let deserialized : Binding = ron::from_str(&serialized).expect("Failed to deserialize binding");
    assert_eq!(binding, deserialized);
}


fn test_action(action : Action){
    let serialized = ron::to_string(&action).expect("Failed to serialize action");
    let deserialized : Action = ron::from_str(&serialized).expect("Failed to deserialize action");
    assert_eq!(action, deserialized)
}

fn test_inputmap<T>(action_0 : T, action_1 : T, action_2 : T)
where T : Debug + PartialEq + Eq + Hash + Clone + Send + Sync + Serialize + DeserializeOwned{
    let mut map = InputMap::<T>::default();
    map.bind(action_0.clone(), KeyCode::Return)
    .bind(action_0, GamepadButtonType::South)
    .bind(action_1.clone(), vec![KeyCode::LAlt, KeyCode::Return])
    .bind(action_1, vec![KeyCode::RAlt, KeyCode::Return])
    // This should bind left/right control and left/right alt, but the combos would get ridiculous so hopefully this is sufficient.
    .bind(
        action_2,
        vec![KeyCode::LAlt, KeyCode::LControl, KeyCode::Return],
    );
    save_to_path(&mut map, TEST_PATH).expect("Failed to save InputMap");
    let mut new_map = InputMap::<T>::default();
    load_from_path(&mut new_map, TEST_PATH).expect("Failed to load InputMap");
    assert_eq!(map.actions, new_map.actions);
    load_from_path(&mut map, TEST_PATH).expect("Failed to Load from path");
    assert_eq!(map.actions, new_map.actions);
}

#[test]
fn it_works(){
    test_binding(KeyCode::Space);
    test_binding(vec![KeyCode::Space, KeyCode::LControl]);
    test_binding(GamepadButtonType::North);
    test_binding(vec![GamepadButtonType::North, GamepadButtonType::LeftThumb]);
    test_action(Action{
        bindings : vec![KeyCode::Space.into(), KeyCode::J.into()]
    });
    test_inputmap::<String>(ACTION_SELECT.to_string(), ACTION_SUPER_SELECT.to_string(), ACTION_AWESOME_SUPER_SELECT.to_string());
    test_inputmap::<TestAction>(TestAction::Select, TestAction::SuperSelect, TestAction::AwesomeSuperSelect);
    std::fs::remove_file(TEST_PATH).expect("Failed to remove testsave file");
}

fn save_to_path<T>(input : &InputMap<T>, path : &str)-> std::io::Result<()>
where T : Debug + PartialEq + Eq + Hash + Clone + Send + Sync + Serialize{
    let mut data = Vec::new();
    for (action, bindings) in input.get_actions(){
        data.push((action, &bindings.bindings));
    }
    let contents = ron::ser::to_string_pretty(&data, ron::ser::PrettyConfig::default()).expect("There was an error making the string");
    std::fs::write(path, contents)?;
    Ok(())
}

fn load_from_path<T>(input : &mut InputMap<T>, path : &str) -> std::io::Result<()>
where T : Debug + PartialEq + Eq + Hash + Clone + Send + Sync + DeserializeOwned{
        let ron_string = std::fs::read_to_string(path)?;
        let actions = ron::from_str(&ron_string).expect("Failed to get actions from ron string");
        input.set_actions(actions);
        //may need to clear self here but i dont really know what that does
        Ok(())
}