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

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
enum Action {
    Select,
    SuperSelect,
    AwesomeSuperSelect,
}

fn setup(mut input: ResMut<InputMap<Action>>) {
    input
        .bind(Action::Select, KeyCode::Enter)
        .bind(Action::Select, GamepadButtonType::South)
        .bind(Action::SuperSelect, vec![KeyCode::AltLeft, KeyCode::Enter])
        .bind(Action::SuperSelect, vec![KeyCode::AltRight, KeyCode::Enter])
        // This should bind left/right control and left/right alt, but the combos would get ridiculous so hopefully this is sufficient.
        .bind(
            Action::AwesomeSuperSelect,
            vec![KeyCode::AltLeft, KeyCode::ControlLeft, KeyCode::Enter],
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
