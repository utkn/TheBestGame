use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct EntityRef {
    id: usize,
    version: u8,
}

impl EntityRef {
    pub fn new(id: usize, version: u8) -> Self {
        Self { id, version }
    }

    /// Returns the id of this reference. The validity must be ensured before using it directly!
    pub fn id(&self) -> usize {
        self.id
    }
}

/// A group of entities.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct EntityGroup(Vec<EntityRef>);

/// A storage type that contains entity references that could be invalidated at any time.
pub trait EntityRefStorage {
    /// Removes the invalidated entities from this storage.
    fn remove_invalids(&mut self, validity_set: &EntityValiditySet) -> HashSet<EntityRef> {
        let invalids = self.get_invalids(validity_set);
        self.try_remove_all(invalids)
    }
    /// Returns the size of the storage.
    fn len(&self) -> usize;
    /// Gets the entities in this storage that have been invalidated.
    fn get_invalids(&self, valids: &EntityValiditySet) -> HashSet<EntityRef>;
    /// Returns true if the given entity is stored in this storage.
    fn contains(&self, e: &EntityRef) -> bool;
    /// Tries to remove all the given entities. Returns the set of entities that were actually removed.
    fn try_remove_all(&mut self, entities: HashSet<EntityRef>) -> HashSet<EntityRef>;
    /// Tries to remove the given entity. Returns true if the removal was successful.
    fn try_remove(&mut self, e: &EntityRef) -> bool;
}

/// A set of entity references that are stored as a hash set.
#[derive(Clone, Debug, Default)]
pub struct EntityRefSet(HashSet<EntityRef>);

impl EntityRefSet {
    pub fn insert(&mut self, e: EntityRef) {
        self.0.insert(e);
    }

    pub fn iter(&self) -> impl Iterator<Item = &EntityRef> {
        self.0.iter()
    }
}

impl EntityRefStorage for EntityRefSet {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn get_invalids(&self, valids: &EntityValiditySet) -> HashSet<EntityRef> {
        self.0
            .iter()
            .filter(|e| !valids.is_valid(e))
            .cloned()
            .collect()
    }

    fn contains(&self, e: &EntityRef) -> bool {
        self.0.contains(e)
    }

    fn try_remove(&mut self, e: &EntityRef) -> bool {
        self.0.remove(e)
    }

    fn try_remove_all(&mut self, entities: HashSet<EntityRef>) -> HashSet<EntityRef> {
        let to_remove: HashSet<_> = self
            .0
            .iter()
            .filter(|e| entities.contains(e))
            .cloned()
            .collect();
        self.0.retain(|e| !to_remove.contains(e));
        to_remove
    }
}

/// A set that contains the valid entities.
#[derive(Clone, Default, Debug)]
pub struct EntityValiditySet(HashMap<usize, u8>);

impl EntityValiditySet {
    /// Returns true if the given entity reference is valid.
    pub fn is_valid(&self, e: &EntityRef) -> bool {
        self.0
            .get(&e.id)
            .map_or(false, |curr_v| curr_v == &e.version)
    }
}

#[derive(Clone, Default, Debug)]
pub struct EntityManager {
    curr_versions: HashMap<usize, u8>,
    free_ids: VecDeque<usize>,
    next_id: usize,
}

impl EntityManager {
    /// Returns the current version associated with the given entity id.
    pub fn get_curr_version(&self, id: usize) -> Option<u8> {
        self.curr_versions.get(&id).cloned()
    }

    /// Returns true iff the given entity reference is valid.
    pub fn is_valid(&self, e: &EntityRef) -> bool {
        self.curr_versions
            .get(&e.id)
            .map_or(false, |curr_v| curr_v == &e.version)
    }

    /// Creates a new entity and returns a valid reference to it.
    pub fn create(&mut self) -> EntityRef {
        let id = self.free_ids.pop_back().unwrap_or_else(|| {
            self.next_id += 1;
            self.next_id - 1
        });
        let version = *self.curr_versions.entry(id).or_insert(0);
        EntityRef::new(id, version)
    }

    /// Removes the entity with the given id, invalidating all the current references to it.
    pub fn remove(&mut self, id: usize) {
        // Update the effective entity version.
        self.curr_versions
            .entry(id)
            .and_modify(|v| *v += 1)
            .or_insert(0);
        self.free_ids.push_front(id);
    }

    /// Returns a set that contains the valid entities.
    pub fn extract_validity_set(&self) -> EntityValiditySet {
        EntityValiditySet(self.curr_versions.clone())
    }
}
