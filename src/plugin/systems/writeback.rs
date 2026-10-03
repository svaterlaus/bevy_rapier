use crate::dynamics::MassProperties;
use crate::dynamics::ReadMassProperties;
use crate::plugin::context::{RapierContextEntityLink, RapierRigidBodySet};
use crate::plugin::RapierConfiguration;
use crate::prelude::MassModifiedEvent;
use bevy::ecs::query::QueryEntityError;
use bevy::prelude::*;

/// System responsible for writing updated mass properties back into the [`ReadMassProperties`] component.
///
/// A [`MassModifiedEvent`] naming an entity that has since despawned is skipped: messages age
/// across fixed ticks, so one written from `Update` can be read after the body it names is gone.
/// An entity that is still alive but has no [`RapierContextEntityLink`] remains a panic, since a
/// dropped request there would leave its mass properties stale for good.
pub fn writeback_mass_properties(
    link: Query<&RapierContextEntityLink>,
    rigidbody_set: Query<&RapierRigidBodySet>,
    config: Query<&RapierConfiguration>,

    mut mass_props: Query<&mut ReadMassProperties>,
    mut mass_modified: MessageReader<MassModifiedEvent>,
) {
    for entity in mass_modified.read() {
        let link = match link.get(entity.0) {
            Ok(link) => link,
            Err(QueryEntityError::NotSpawned(_)) => continue,
            Err(err) => panic!("Could not find `RapierContextEntityLink`: {err:?}"),
        };
        let config = config
            .get(link.0)
            .expect("Could not find `RapierConfiguration`");
        if config.physics_pipeline_active {
            let Ok(rigidbody_set) = rigidbody_set.get(link.0) else {
                continue;
            };

            if let Some(handle) = rigidbody_set.entity2body.get(entity).copied() {
                if let Some(rb) = rigidbody_set.bodies.get(handle) {
                    if let Ok(mut mass_props) = mass_props.get_mut(**entity) {
                        let new_mass_props =
                            MassProperties::from_rapier(rb.mass_properties().local_mprops);

                        // NOTE: we write the new value only if there was an
                        //       actual change, in order to not trigger bevy’s
                        //       change tracking when the values didn’t change.
                        if mass_props.get() != &new_mass_props {
                            mass_props.set(new_mass_props);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn mass_modified_for_despawned_entity_is_skipped() {
        let mut app = App::new();
        app.add_message::<MassModifiedEvent>();
        let body = app.world_mut().spawn_empty().id();
        app.world_mut().write_message(MassModifiedEvent::from(body));
        app.world_mut().despawn(body);

        app.world_mut()
            .run_system_once(writeback_mass_properties)
            .expect("writeback runs");
    }

    #[test]
    #[should_panic(expected = "Could not find `RapierContextEntityLink`")]
    fn mass_modified_for_live_entity_without_link_panics() {
        let mut app = App::new();
        app.add_message::<MassModifiedEvent>();
        let body = app.world_mut().spawn_empty().id();
        app.world_mut().write_message(MassModifiedEvent::from(body));

        let _ = app.world_mut().run_system_once(writeback_mass_properties);
    }
}
