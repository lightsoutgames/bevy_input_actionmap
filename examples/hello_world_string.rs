use bevy::prelude::*;
use bevy_input_actionmap::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ActionPlugin::<String>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, run_commands)
        .run();
}

const ACTION_SELECT: &str = "SELECT";
const ACTION_SUPER_SELECT: &str = "SUPER_SELECT";
const ACTION_AWESOME_SUPER_SELECT: &str = "AWESOME_SUPER_SELECT";

fn setup(mut input: ResMut<InputMap<String>>) {
    input
        .bind(ACTION_SELECT, KeyCode::Enter)
        .bind(ACTION_SELECT, GamepadButtonType::South)
        .bind(ACTION_SUPER_SELECT, vec![KeyCode::AltLeft, KeyCode::Enter])
        .bind(ACTION_SUPER_SELECT, vec![KeyCode::AltRight, KeyCode::Enter])
        // This should bind left/right control and left/right alt, but the combos would get ridiculous so hopefully this is sufficient.
        .bind(
            ACTION_AWESOME_SUPER_SELECT,
            vec![KeyCode::AltLeft, KeyCode::ControlLeft, KeyCode::Enter],
        );
}

fn run_commands(input: Res<InputMap<String>>) {
    if input.just_active(ACTION_SELECT) {
        println!("Selected");
    }
    if input.just_active(ACTION_SUPER_SELECT) {
        println!("Super selected");
    }
    if input.just_active(ACTION_AWESOME_SUPER_SELECT) {
        println!("Awesome super selected");
    }
}
