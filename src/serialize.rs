use super::*;
use serde::{Serialize, de::DeserializeOwned};

#[cfg(test)]
mod test{
    const ACTION_SELECT: &str = "SELECT";
    const ACTION_SUPER_SELECT: &str = "SUPER_SELECT";
    const ACTION_AWESOME_SUPER_SELECT: &str = "AWESOME_SUPER_SELECT";
    const TESTPATH : &str = "test.keymap";
    use bevy::prelude::*;
    use super::*;
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
    #[test]
    fn test_inputmap_string(){
        let mut map = super::InputMap::<String>::default();
        map.bind(ACTION_SELECT, KeyCode::Return)
        .bind(ACTION_SELECT, GamepadButtonType::South)
        .bind(ACTION_SUPER_SELECT, vec![KeyCode::LAlt, KeyCode::Return])
        .bind(ACTION_SUPER_SELECT, vec![KeyCode::RAlt, KeyCode::Return])
        // This should bind left/right control and left/right alt, but the combos would get ridiculous so hopefully this is sufficient.
        .bind(
            ACTION_AWESOME_SUPER_SELECT,
            vec![KeyCode::LAlt, KeyCode::LControl, KeyCode::Return],
        );
        map.save_to_path(TESTPATH).expect("Failed to save string InputMap");
        let new_map = InputMap::<String>::new_from_path(TESTPATH).expect("Failed to load string InputMap");
        assert_eq!(map.actions, new_map.actions);
        map.load_from_path(TESTPATH).expect("Failed to Load from path");
        assert_eq!(map.actions, new_map.actions);
    }

    fn test_inputmap_enum(){
        let mut map = super::InputMap::<TestAction>::default();
        map.bind(TestAction::Select, KeyCode::Return)
        .bind(TestAction::Select, GamepadButtonType::South)
        .bind(TestAction::SuperSelect, vec![KeyCode::LAlt, KeyCode::Return])
        .bind(TestAction::SuperSelect, vec![KeyCode::RAlt, KeyCode::Return])
        // This should bind left/right control and left/right alt, but the combos would get ridiculous so hopefully this is sufficient.
        .bind(
            TestAction::AwesomeSuperSelect,
            vec![KeyCode::LAlt, KeyCode::LControl, KeyCode::Return],
        );
        map.save_to_path(TESTPATH).expect("Failed to save enum InputMap");
        let new_map = InputMap::<TestAction>::new_from_path(TESTPATH).expect("Failed to load enum InputMap");
        assert_eq!(map.actions, new_map.actions);
        map.load_from_path(TESTPATH).expect("Failed to Load from path");
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
        test_inputmap_string();
        test_inputmap_enum();
        std::fs::remove_file(TESTPATH).expect("Failed to remove testsave file");
    }
}

impl<T : Eq + Hash> InputMap<T> {
    pub fn save_to_path(&self, path : &str) -> std::io::Result<()> where T : Serialize{
        let contents = ron::ser::to_string_pretty(&self.actions, ron::ser::PrettyConfig::default()).expect("There was an error making the string");
        std::fs::write(path, contents)
    }
    pub fn load_from_path(&mut self, path : &str) -> std::io::Result<()> where T : DeserializeOwned{
        let ron_string = std::fs::read_to_string(path)?;
        let actions = ron::from_str(&ron_string).expect("Failed to get actions from ron string");
        self.actions = actions;
        //may need to clear self here but i dont really know what that does
        Ok(())
    }
    pub fn new_from_path(path : &str) -> std::io::Result<InputMap<T>> where T : DeserializeOwned{
        let ron_string = std::fs::read_to_string(path)?;
        let actions = ron::from_str(&ron_string).expect("Failed to get actions from ron string");
        Ok(InputMap{
            actions,
            ..Default::default()
        })
    }
}