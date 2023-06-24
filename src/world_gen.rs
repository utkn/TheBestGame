use notan::egui::epaint::ahash::HashMap;

use crate::prelude::{State, Transform, World};

pub enum EntityTemplate {
    Chest,
    Building,
    Player,
    Bandit,
}

pub struct WorldTemplate {
    templates: Vec<(EntityTemplate, Transform)>,
}

impl WorldTemplate {}

pub struct WorldGenerator {
    chunk_size: (usize, usize),
    loaded_chunks: HashMap<(usize, usize), Box<World>>,
}

impl WorldGenerator {
    fn generate_or_fetch(&mut self, chunk_pos: &(usize, usize)) -> World {
        todo!();
    }
}
