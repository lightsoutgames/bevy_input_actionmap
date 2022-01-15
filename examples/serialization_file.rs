use bevy::prelude::*;
use bevy_input_actionmap::*;
use serde::{Deserialize, Serialize};

fn main() {
    App::new()
        .add_plugin(ActionPlugin::<Action>::default())
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(run_commands)
        .run();
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
enum Action {
    Select,
    Up,
    Down,
    Left,
    Right,
}

const PATH: &'static str = "examples/keybindings.ron";

fn setup(mut input: ResMut<InputMap<Action>>) {
    #[cfg(feature = "serialize")]
    if let Err(_) = load_from_path(&mut input, PATH) {
        {
            println!("no keybind config found creating default setup"); //just to show the path it took
            create_default_keybindings(&mut input);
            save_to_path(&input, PATH).unwrap()
        }
    } else {
        //if it loaded custom keybinds dont add new ones
        println!("keybindings loaded from local file") //just to show the path it took
    }
    #[cfg(not(feature = "serialize"))]
        create_default_keybindings(&mut input);
}

fn create_default_keybindings(input: &mut ResMut<InputMap<Action>>) {
    //this is so if you want to change default keybindings you dont need to do more then once
    input
        .bind(Action::Select, KeyCode::Return)
        .bind(Action::Select, GamepadButtonType::Select)
        .bind(Action::Up, vec![KeyCode::Up])
        .bind(Action::Up, vec![GamepadButtonType::North])
        .bind(Action::Down, vec![KeyCode::Down])
        .bind(Action::Down, vec![GamepadButtonType::South])
        .bind(Action::Left, vec![KeyCode::Left])
        .bind(Action::Left, vec![GamepadButtonType::West])
        .bind(Action::Right, vec![KeyCode::Right])
        .bind(Action::Right, vec![GamepadButtonType::East]);
}

fn run_commands(input: Res<InputMap<Action>>) {
    if input.just_active(Action::Select) {
        println!("Selected");
    }
    if input.just_active(Action::Up) {
        println!("Go up");
    }
    if input.just_active(Action::Down) {
        println!("Go down");
    }
    if input.just_active(Action::Left) {
        println!("Go left");
    }
    if input.just_active(Action::Right) {
        println!("Go right");
    }
}

fn save_to_path(input: &InputMap<Action>, path: &str) -> std::io::Result<()> {
    let contents = ron::ser::to_string_pretty(input, ron::ser::PrettyConfig::default())
        .expect("There was an error making the string");
    std::fs::write(path, contents)?;
    Ok(())
}

fn load_from_path(input: &mut InputMap<Action>, path: &str) -> std::io::Result<()> {
    let ron_string = std::fs::read_to_string(path)?;
    let config = ron::from_str::<InputMap<Action>>(&ron_string).expect("Failed to get actions from ron string");
    input.actions = config.actions;
    //may need to clear self here but i dont really know what that does
    Ok(())
}
