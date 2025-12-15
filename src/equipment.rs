use crate::tile::{TextAnchor, text_tiles};
use bevy::prelude::*;
use bevy_query_observer::{AddStartObserver, Start};

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            (health_unit_display, shield_unit_display),
            display_equipment,
        )
            .chain(),
    )
    .add_start_observer(HealthUnit::observe_insertion);
}

#[derive(Component, Clone, Deref, DerefMut, Default)]
pub struct Health(pub i32);

#[derive(Component, Clone, Copy)]
#[require(Name::new("Health Unit"), EquipmentDisplay, Tooltips::HEALTH_UNIT)]
#[component(immutable)]
pub struct HealthUnit(pub i32);

impl HealthUnit {
    fn observe_insertion(data: Start<(&Self, &EquipmentOf)>, mut commands: Commands) {
        let (&health, target) = data.into_inner();

        commands
            .entity(target.0)
            .entry::<Health>()
            .or_default()
            .and_modify(move |mut h| h.0 += health.0);
    }
}

fn health_unit_display(mut units: Query<(&HealthUnit, &mut EquipmentDisplay)>) {
    for (unit, mut display) in units.iter_mut() {
        display.0 = format!("x{}", unit.0);
    }
}

#[derive(Component)]
#[require(Name::new("Shield Unit"), EquipmentDisplay, Tooltips::SHIELD_UNIT)]
pub struct ShieldUnit(pub i32);

fn shield_unit_display(mut units: Query<(&ShieldUnit, &mut EquipmentDisplay)>) {
    for (unit, mut display) in units.iter_mut() {
        display.0 = format!("x{}", unit.0);
    }
}

#[derive(Component)]
#[relationship_target(relationship = EquipmentOf)]
pub struct Equipment(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = Equipment)]
pub struct EquipmentOf(pub Entity);

#[derive(Default, Component)]
pub struct EquipmentDisplay(String);

#[derive(Component)]
pub struct Tooltips(&'static str);

impl Tooltips {
    pub const HEALTH_UNIT: Self = Self("Health");
    pub const SHIELD_UNIT: Self = Self("Shield");
}

#[derive(Component)]
struct EquipmentEntity;

const EQUIPMENT_PANEL: &str = r#"
+-------------------------------+
|                               |
|       E Q U I P M E N T       |
|                               |
+-------------------------------+
|                               |
|                               |
|                               |
|                               |
|                               |
|                               |
|                               |
|                               |
|                               |
|                               |
|                               |
|                               |
|                               |
+-------------------------------+
"#;

fn display_equipment(
    mut commands: Commands,
    player: Single<&Equipment>,
    equipment: Query<(&Name, &EquipmentDisplay, &Tooltips)>,
    entities: Query<Entity, With<EquipmentEntity>>,
    input: Res<ButtonInput<KeyCode>>,
    mut is_displayed: Local<bool>,
) {
    if input.just_pressed(KeyCode::KeyI) {
        if *is_displayed {
            for entity in entities.iter() {
                commands.entity(entity).despawn();
            }
        } else {
            for (tile, position) in text_tiles(EQUIPMENT_PANEL, 0, 0, TextAnchor::Center) {
                commands.spawn((
                    tile,
                    Transform::from_translation(position.extend(10.0)),
                    EquipmentEntity,
                ));
            }

            for (y, (name, display, tooltips)) in equipment.iter_many(player.iter()).enumerate() {
                for (tile, position) in text_tiles(
                    &format!("{} - {} - {}", name, display.0, tooltips.0),
                    -30 / 2,
                    4 - y as i32,
                    TextAnchor::TopLeft,
                ) {
                    commands.spawn((
                        tile,
                        Transform::from_translation(position.extend(15.0)),
                        EquipmentEntity,
                    ));
                }
            }
        }
        *is_displayed = !*is_displayed;
    }
}
