# Broken: Godot Rapier2d Simple

Motivation for this project was to integrate rapier2d physics into godot while keeping the scene tree editor's inheritance semantics.

Unfortunately i realized too late that Godot 3.5 does not allow NativeScript's to be extended. **Therefore this entire project is of little use.**

Anyway it might serve as example code for bridging godot to rapier physics.

See the addons/rapier-2d-simple/examples/ folder for a working proof of concept.

>I'll probably try bridging rapier2d physics again once Godot 4 and rust gdnative's GDExtensions are stable...

*PS: I never got the collision signal reporting to work. Rapier supports a u128 user-data-type which i thought to abuse for raw pointers to Node2D nodes.*

*PPS: The codebase is really dirty and unsafe, full of RefCell's and raw pointer coercion. Rust's memory model is really disrespected here.*