# bevy_collision

WIP bevy plugin for collision handling

* Only supports 2D for now, but supporting 3D should not exactly be a big deal

# Usage note 

* Insert a new entity and add a `ColliderBuilder` component to it
* The **collision system** (this plugin) drives changes to your bevy `Transform`s, not the other way around. Keep that in mind. 
* Use the `Velocity` component to move a colliding entity around

# Example

* Launch using `cargo r --example hello`
* Use arrow keys to move
* Whenever the player is colliding with something, it turns red. That's about it.

