use crate::prelude::{Transform, World};

mod empty_world;
mod entity_template;

use empty_world::*;
pub use entity_template::*;

pub struct WorldTemplate {
    entity_templates: Vec<(Transform, EntityTemplate)>,
}

impl WorldTemplate {
    pub fn new(entity_templates: impl IntoIterator<Item = (Transform, EntityTemplate)>) -> Self {
        Self {
            entity_templates: entity_templates.into_iter().collect(),
        }
    }
}

pub struct WorldGenerator;

impl WorldGenerator {
    pub fn generate(world_template: WorldTemplate) -> World {
        let mut world = create_empty_world();
        world.update_with(|state, cmds| {
            world_template
                .entity_templates
                .into_iter()
                .for_each(|(trans, templ)| {
                    templ.generate(trans, state, cmds);
                })
        });
        world
    }
}
