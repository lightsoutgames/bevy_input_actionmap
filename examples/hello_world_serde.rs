use bevy::prelude::*;
use bevy_input_actionmap::*;

fn main() {
    App::build()
    .add_plugins(DefaultPlugins)
    .add_plugin(ActionPlugin::<Action>::default())
    .add_startup_system(setup.system())
    .add_system(run_commands.system())
    .run();
}

#[derive(Hash, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
enum Action {
    Select,
    SuperSelect,
    AwesomeSuperSelect,
}

fn setup(mut input: ResMut<InputMap<Action>>){
    #[cfg(feature = "serialize")]
    if let Err(_) = load_from_path(&mut input, "keybindings.config") {
        {   println!("no keybind config found creating default setup"); //just to show the path it took
            create_default_keybindings(&mut input);
            save_to_path(&input,"keybindings.config").unwrap()}
    } else {//if it loaded custom keybinds dont add new ones
        println!("keybindings loaded from local file") //just to show the path it took
    }
    #[cfg(not(feature = "serialize"))]
    create_default_keybindings(&mut input);
}

fn create_default_keybindings(input : &mut ResMut<InputMap<Action>>){ //this is so if you want to change default keybindings you dont need to do more then once
    input
    .bind(Action::Select, KeyCode::Return)
    .bind(Action::Select, GamepadButtonType::South)
    .bind(Action::SuperSelect, vec![KeyCode::LAlt, KeyCode::Return])
    .bind(Action::SuperSelect, vec![KeyCode::RAlt, KeyCode::Return])
    // This should bind left/right control and left/right alt, but the combos would get ridiculous so hopefully this is sufficient.
    .bind(
        Action::AwesomeSuperSelect,
        vec![KeyCode::LAlt, KeyCode::LControl, KeyCode::Return],
    );
}

fn run_commands(input: Res<InputMap<Action>>) {
    if input.just_active(Action::Select) {
        println!("Selected");
    }
    if input.just_active(Action::SuperSelect) {
        println!("Super selected");
    }
    if input.just_active(Action::AwesomeSuperSelect) {
        println!("Awesome super selected");
    }
}

fn save_to_path(input : &InputMap<Action>, path : &str)-> std::io::Result<()>{
    let mut data = Vec::new();
    for (action, bindings) in input.get_actions(){
        data.push((action, &bindings.bindings));
    }
    let contents = ron::ser::to_string_pretty(&data, ron::ser::PrettyConfig::default()).expect("There was an error making the string");
    std::fs::write(path, contents)?;
    Ok(())
}

fn load_from_path(input : &mut InputMap<Action>, path : &str) -> std::io::Result<()>{
        let ron_string = std::fs::read_to_string(path)?;
        let actions = ron::from_str(&ron_string).expect("Failed to get actions from ron string");
        input.set_actions(actions);
        //may need to clear self here but i dont really know what that does
        Ok(())
}