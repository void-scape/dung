use std::marker::PhantomData;

use bevy::{
    color::palettes::css::GRAY,
    ecs::{lifecycle::HookContext, system::IntoObserverSystem, world::DeferredWorld},
    prelude::*,
};
use bevy_frp::{SignalExt, TQuery};

pub enum BattleState {
    Player,
    PlayerResult,
    EnemyResult,
    Complete(BattleComplete),
}

enum BattleComplete {
    Win,
    Loss,
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, |mut commands: Commands| {
        let battle_screen = battle_screen(commands.reborrow());

        commands.spawn((
            Node {
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            children![battle_screen],
        ));
    });
}

fn battle_screen(mut commands: Commands) -> impl Bundle + use<> {
    (
        Node {
            width: px(500),
            height: px(500),
            flex_direction: FlexDirection::Column,
            border: UiRect::all(px(1)),
            padding: UiRect::all(px(12)),
            ..Default::default()
        },
        BackgroundColor(Color::BLACK),
        BorderColor::all(Color::WHITE),
        children![
            (
                Node {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                children![(
                    Text::new("FIGHT"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                )]
            ),
            // battle canvas
            (
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    padding: UiRect::horizontal(px(12)),
                    ..Default::default()
                },
                children![(
                    Node {
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::SpaceBetween,
                        ..Default::default()
                    },
                    children![Text::new("p"), Text::new("e"),]
                )]
            ),
            (
                Node {
                    justify_content: JustifyContent::SpaceBetween,
                    width: Val::Percent(100.0),
                    column_gap: px(12),
                    ..Default::default()
                },
                children![
                    (
                        button("Attack", commands.reborrow()),
                        on(|trigger: On<Pointer<Click>>| {
                            info!("attack");
                        }),
                    ),
                    (
                        button("Escape", commands.reborrow()),
                        on(|trigger: On<Pointer<Click>>| {
                            info!("escape");
                        }),
                    ),
                    (
                        button("Block", commands.reborrow()),
                        on(|trigger: On<Pointer<Click>>| {
                            info!("block");
                        }),
                    )
                ]
            )
        ],
    )
}

#[derive(Component)]
#[component(on_insert = Self::insert)]
pub struct ObserverSystem<E: EntityEvent> {
    observer: Box<dyn FnOnce(&mut World, Entity) + Send + Sync>,
    event: PhantomData<fn() -> E>,
}

pub fn on<E, S, M>(system: S) -> ObserverSystem<E>
where
    S: IntoObserverSystem<E, (), M> + Send + Sync + 'static,
    E: EntityEvent,
{
    ObserverSystem {
        observer: Box::new(move |world, entity| {
            world.entity_mut(entity).observe(system);
        }),
        event: PhantomData,
    }
}

impl<E: EntityEvent> ObserverSystem<E> {
    fn insert(mut world: DeferredWorld, context: HookContext) {
        world.commands().queue(move |world: &mut World| {
            let Some(sys) = world.entity_mut(context.entity).take::<Self>() else {
                return;
            };
            (sys.observer)(world, context.entity);
        });
    }
}

/// A UI element that can receive focus.
struct FocusTarget;

#[derive(Component)]
struct IsFocused;

fn button<T: Into<String>>(text: T, mut commands: Commands) -> impl Bundle + use<T> {
    let target = bevy_frp::Target::new();
    let background_color = commands.derive(move |focused: TQuery<Has<IsFocused>>| {
        if focused.get(target).unwrap_or_default() {
            BackgroundColor(GRAY.into())
        } else {
            BackgroundColor(Color::BLACK)
        }
    });

    (
        Button,
        target,
        on(|t: On<Pointer<Over>>, mut commands: Commands| {
            commands.entity(t.entity).insert(IsFocused);
        }),
        on(|t: On<Pointer<Out>>, mut commands: Commands| {
            commands.entity(t.entity).remove::<IsFocused>();
        }),
        Node {
            // width: px(150),
            padding: UiRect::all(px(12)),
            border: UiRect::all(px(1)),
            // horizontally center child text
            justify_content: JustifyContent::Center,
            // vertically center child text
            align_items: AlignItems::Center,
            ..default()
        },
        BorderColor::all(Color::WHITE),
        // BorderRadius::MAX,
        background_color,
        // BackgroundColor(Color::BLACK),
        children![(
            Text::new(text),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            TextShadow::default(),
        )],
    )
}
