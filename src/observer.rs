use bevy::{
    ecs::{lifecycle::HookContext, system::IntoObserverSystem, world::DeferredWorld},
    prelude::*,
};
use std::marker::PhantomData;

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
    pub fn on<S, M>(system: S) -> ObserverSystem<E>
    where
        S: IntoObserverSystem<E, (), M> + Send + Sync + 'static,
    {
        on(system)
    }

    fn insert(mut world: DeferredWorld, context: HookContext) {
        world.commands().queue(move |world: &mut World| {
            let Some(sys) = world.entity_mut(context.entity).take::<Self>() else {
                return;
            };
            (sys.observer)(world, context.entity);
        });
    }
}
