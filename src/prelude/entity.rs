use std::collections::{HashMap, VecDeque};

/// Reference to an entity in the system.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct EntityRef {
    id: usize,
    version: u8,
}

impl EntityRef {
    /// Creates a new entity reference.
    pub fn new(id: usize, version: u8) -> Self {
        Self { id, version }
    }

    /// Returns the id of this reference. The validity must be ensured before using it directly!
    pub fn id(&self) -> usize {
        self.id
    }
}

/// A storage type that contains entity references that could be invalidated at any time.
pub trait EntityRefBag {
    /// Removes the invalidated entities from this storage.
    fn remove_invalids(&mut self, entity_mgr: &EntityManager);
}

#[derive(Clone, Default, Debug)]
pub struct EntityManager {
    curr_versions: HashMap<usize, u8>,
    free_ids: VecDeque<usize>,
    next_id: usize,
}

impl EntityManager {
    /// Returns the current version associated with the given entity id.
    pub(super) fn get_curr_version(&self, id: usize) -> Option<u8> {
        self.curr_versions.get(&id).cloned()
    }

    pub(super) fn get_all<'a>(&'a self) -> impl Iterator<Item = EntityRef> + 'a {
        self.curr_versions
            .iter()
            .map(|(id, v)| EntityRef::new(*id, *v))
    }

    /// Returns true iff the given entity reference is valid.
    pub fn is_valid(&self, e: &EntityRef) -> bool {
        self.curr_versions
            .get(&e.id)
            .map_or(false, |curr_v| curr_v == &e.version)
    }

    /// Creates a new entity and returns a valid reference to it.
    pub(super) fn create(&mut self) -> EntityRef {
        let id = self.free_ids.pop_back().unwrap_or_else(|| {
            self.next_id += 1;
            self.next_id - 1
        });
        let version = *self.curr_versions.entry(id).or_insert(0);
        EntityRef::new(id, version)
    }

    /// Removes the entity with the given id, invalidating all the current references to it.
    pub(super) fn remove(&mut self, id: usize) {
        // Update the effective entity version.
        self.curr_versions
            .entry(id)
            .and_modify(|v| *v += 1)
            .or_insert(0);
        self.free_ids.push_front(id);
    }
}
