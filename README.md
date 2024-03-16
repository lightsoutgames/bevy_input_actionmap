# bevy_input_actionmap

Maps key and gamepad events to actions in [Bevy](https://bevyengine.org). See the example for usage.

I'll be the first to admit that this crate needs some polish. Things it seems to do right:

* Binds string actions to single or multiple keycodes, gamepad buttons, or stick motions. Not sure that cross-input gestures work (I.e. gamepad buttons and keys).
* Binds the same action to multiple distinct input types. The same action can be bound to a key, gamepad button, etc.
* Resolves key/button conflicts. Binding actions to _Enter_, _Ctrl-Enter_ and _Ctrl-Alt-Enter_ only runs a single action if _Ctrl-Alt-Enter_ is pressed.

Things that don't work and that I'd appreciate help with:

* Mouse gestures. PRs welcome.
* Serialization of keybindings. PRs welcome.
* Probably a million other things. PRs welcome.

## Bevy Version Support

| bevy | bevy_input_actionmap |
| ---- | -------------------- |
| 0.13 | 0.13                 |
| 0.7  | 0.1                  |
