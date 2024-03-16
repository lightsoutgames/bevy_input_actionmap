# bevy_input_actionmap

Maps key and gamepad events to actions in [Bevy](https://bevyengine.org).

I'll be the first to admit that this crate needs some polish. Things it seems to do right:

* Binds string actions to single or multiple keycodes, gamepad buttons, or stick motions. Cross-input gestures work (I.e. gamepad buttons and keys).
* Binds the same action to multiple distinct input types. The same action can be bound to a key, gamepad button, a mouse button, or require a combo of all three!
* Resolves key/button conflicts. Binding actions to _Enter_, _Ctrl-Enter_ and _Ctrl-Alt-Enter_ only runs a single action if _Ctrl-Alt-Enter_ is pressed.

# Example
```rust no_run
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
}

fn setup(mut input: ResMut<InputMap<MyAction>>) {
    input
        .bind(MyAction::Select, KeyCode::Enter)
        .bind(MyAction::Select, GamepadButtonType::South)
        .bind(MyAction::Select, GamepadAxisDirection::LeftStickYPositive)
        .bind(MyAction::SuperSelect, vec![KeyCode::AltLeft, KeyCode::Enter])
        .bind(MyAction::SuperSelect, vec![KeyCode::AltRight, KeyCode::Enter])
        .bind(MyAction::AwesomeSuperSelect, vec![KeyCode::AltLeft, KeyCode::ControlLeft, KeyCode::Enter] );

    // Ctrl + J + MiddleClick!
    input.bind(MyAction::AwesomeSuperSelect, 
        Binding::new(KeyCode::ControlLeft)
            .with(KeyCode::KeyJ)
            .with(MouseButton::Middle)
    );

    // Controller + Keyboard combos are not just possible, 
    // they're annoying!
    input.bind(MyAction::AwesomeSuperSelect,
        Binding::new(KeyCode::ControlLeft)
            .with_axis(GamepadAxisDirection::LeftStickXPositive, 0.2)
            .with(GamepadButtonType::RightTrigger),
    );
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
}
```

Things that don't work and that I'd appreciate help with:

* Mouse gestures. PRs welcome.
* Probably a million other things. PRs welcome.

## Bevy Version Support

| bevy | bevy_input_actionmap |
| ---- | -------------------- |
| 0.13 | 0.13                 |
| 0.7  | 0.1                  |
