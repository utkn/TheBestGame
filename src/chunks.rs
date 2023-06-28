use std::collections::HashMap;

use crate::prelude::{EntityRef, State};

#[derive(Clone, Copy, Debug)]
pub enum VirtualEntityRef {
    Overworld(EntityRef),
    Chunk((isize, isize), EntityRef),
}

#[derive(Debug, Default)]
pub struct WorldChunks {
    chunk_size: f32,
    overworld: State,
    chunks: HashMap<(isize, isize), State>,
}

impl WorldChunks {
    pub fn new(chunk_size: f32, overworld: State) -> Self {
        Self {
            overworld,
            chunk_size,
            chunks: Default::default(),
        }
    }

    fn get_state_mut(&mut self, pos: &(f32, f32)) -> &mut State {
        let chunk_idx = (
            (pos.0 / self.chunk_size).floor() as isize,
            (pos.1 / self.chunk_size).floor() as isize,
        );
        self.chunks
            .entry(chunk_idx)
            .or_insert_with(|| State::default())
    }

    fn get_state(&mut self, pos: &(f32, f32)) -> anyhow::Result<&State> {
        let chunk_idx = (
            (pos.0 / self.chunk_size).floor() as isize,
            (pos.1 / self.chunk_size).floor() as isize,
        );
        self.chunks
            .get(&chunk_idx)
            .ok_or(anyhow::anyhow!("no chunk at the given position"))
    }
}
