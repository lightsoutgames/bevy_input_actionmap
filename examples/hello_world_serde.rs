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

#[cfg(feature = "serialize")]
fn setup(mut input: ResMut<InputMap<Action>>) {
    if let Err(_) = input.load_from_path("keybinds.config") {
        {   println!("no keybind config found creating default setup"); //just to show the path it took
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
            input.save_to_path("keybinds.config").unwrap()}
    } else {//if it loaded custom keybinds dont add new ones
        println!("keybinds loaded from local file") //just to show the path it took
    }
    
}
#[cfg(not(feature = "serialize"))]
fn setup(mut input: ResMut<InputMap<Action>>){
    println!("serialize feature is off so this is the same as hello_world_enum; Why just Why");
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
