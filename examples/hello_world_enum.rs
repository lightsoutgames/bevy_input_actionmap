use bevy::prelude::*;
use bevy_input_actionmap::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ActionPlugin::<Action>::default())
        .add_startup_system(setup)
        .add_system(run_commands)
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
