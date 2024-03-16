use bevy::prelude::*;
use bevy_input_actionmap::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ActionPlugin::<MyAction>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, run_commands)
        .run();
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
enum MyAction {
    Select,
    SuperSelect,
    AwesomeSuperSelect,
    FooBar,
}

fn setup(mut input: ResMut<InputMap<MyAction>>) {
    input
        .bind(MyAction::Select, KeyCode::Enter)
        .bind(MyAction::Select, GamepadButtonType::South)
        .bind(MyAction::SuperSelect, vec![KeyCode::AltLeft, KeyCode::Enter])
        .bind(MyAction::SuperSelect, vec![KeyCode::AltRight, KeyCode::Enter])
        // This should bind left/right control and left/right alt, but the combos would get ridiculous so hopefully this is sufficient.
        .bind(
            MyAction::AwesomeSuperSelect,
            vec![KeyCode::AltLeft, KeyCode::ControlLeft, KeyCode::Enter],
        );
        
    // Ctrl + J + MiddleClick!
    let complex_binding = Binding::new(&[KeyCode::ControlLeft, KeyCode::KeyJ])
        .with(MouseButton::Middle);

    input.bind( MyAction::FooBar, complex_binding );
}

fn run_commands(input: Res<InputMap<MyAction>>) {
    if input.just_active(MyAction::Select) {
        println!("Selected");
    }
    if input.just_active(MyAction::SuperSelect) {
        println!("Super selected");
    }
    if input.just_active(MyAction::AwesomeSuperSelect) {
        println!("Awesome super selected");
    }

    if input.just_active(MyAction::FooBar) {
        println!("FooBar!!!!!!");
    }
}
