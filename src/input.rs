use crate::player::Player;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_input_context::<Player>()
        .add_observer(inject_bindings);
}

#[derive(InputAction)]
#[action_output(Vec2)]
pub struct Move;

fn inject_bindings(trigger: On<Insert, Player>, mut commands: Commands) {
    commands.entity(trigger.entity).insert(actions!(Player[
        (
            Action::<Move>::new(),
            DeadZone::default(),
            Pulse::new(0.2),
            Bindings::spawn((
                Cardinal::wasd_keys(),
                Cardinal::arrows(),
                Axial::left_stick(),
            )),
        ),
    ]));
}
