#![allow(dead_code, unused_imports)]

#[macro_export]
macro_rules! __arena_index {
    (
        $v:vis $name:ident $(,)?
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        $v struct $name {
            index: $crate::arena::ArenaIndex,
        }

        impl $name {
            #[allow(dead_code)]
            $v fn generation(&self) -> u64 {
                self.index.generation()
            }

            #[allow(dead_code)]
            $v fn index(&self) -> usize {
                self.index.index()
            }
        }

        impl From<$crate::arena::ArenaIndex> for $name {
            fn from(value: $crate::arena::ArenaIndex) -> Self {
                Self {
                    index: value,
                }
            }
        }

        impl From<$name> for $crate::arena::ArenaIndex {
            fn from(value: $name) -> Self {
                value.index
            }
        }
    };
}

pub use __arena_index as arena_index;

use std::mem;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArenaIndex {
    generation: u64,
    index: usize,
}

impl ArenaIndex {
    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn index(&self) -> usize {
        self.index
    }
}

enum Entry<T> {
    Free,
    Occupied { generation: u64, value: T },
}

pub struct Arena<T> {
    arena: Vec<Entry<T>>,
    next_free: Vec<usize>,
    generation: u64,
    count: usize,
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self {
            arena: vec![],
            next_free: vec![],
            generation: 0,
            count: 0,
        }
    }
}

impl<T> Arena<T> {
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn size(&self) -> usize {
        self.arena.len()
    }

    pub fn occupied_by_usize(&self) -> impl Iterator<Item = usize> {
        self.arena
            .iter()
            .enumerate()
            .filter(|(_, e)| matches!(e, Entry::Occupied { .. }))
            .flat_map(|(i, e)| match e {
                Entry::Occupied { .. } => Some(i),
                Entry::Free => None,
            })
    }

    pub fn occupied(&self) -> impl Iterator<Item = ArenaIndex> {
        self.arena
            .iter()
            .enumerate()
            .filter(|(_, e)| matches!(e, Entry::Occupied { .. }))
            .flat_map(|(i, e)| match e {
                Entry::Occupied { generation, .. } => Some(ArenaIndex {
                    generation: *generation,
                    index: i,
                }),
                Entry::Free => None,
            })
    }

    pub fn insert(&mut self, value: T) -> ArenaIndex {
        self.count += 1;

        if let Some(free_index) = self.next_free.pop()
            && let Some(entry) = self.arena.get_mut(free_index)
        {
            *entry = Entry::Occupied {
                generation: self.generation,
                value,
            };

            ArenaIndex {
                generation: self.generation,
                index: free_index,
            }
        } else {
            self.arena.push(Entry::Occupied {
                generation: self.generation,
                value,
            });

            ArenaIndex {
                generation: self.generation,
                index: self.arena.len() - 1,
            }
        }
    }

    pub fn get_index_by_usize(&self, index: usize) -> Option<ArenaIndex> {
        if let Entry::Occupied { generation, .. } = self.arena.get(index)? {
            Some(ArenaIndex {
                generation: *generation,
                index,
            })
        } else {
            None
        }
    }

    pub fn get_by_usize(&self, index: usize) -> Option<&T> {
        if let Entry::Occupied { value, .. } = self.arena.get(index)? {
            Some(value)
        } else {
            None
        }
    }

    pub fn get(&self, index: ArenaIndex) -> Option<&T> {
        if let Entry::Occupied { generation, .. } = self.arena.get(index.index)?
            && *generation == index.generation
        {
            self.get_by_usize(index.index)
        } else {
            None
        }
    }

    pub fn get_mut_by_usize(&mut self, index: usize) -> Option<&mut T> {
        if let Entry::Occupied { value, .. } = self.arena.get_mut(index)? {
            Some(value)
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, index: ArenaIndex) -> Option<&mut T> {
        if let Entry::Occupied { generation, .. } = self.arena.get(index.index)?
            && *generation == index.generation
        {
            self.get_mut_by_usize(index.index)
        } else {
            None
        }
    }

    pub fn remove_by_usize(&mut self, index: usize) -> Option<T> {
        if let to_remove @ Entry::Occupied { .. } = self.arena.get_mut(index)? {
            let value = mem::replace(to_remove, Entry::Free);

            let Entry::Occupied { value, .. } = value else {
                panic!("Occupied entry was unexpectedly free")
            };

            self.next_free.push(index);

            self.generation += 1;
            self.count -= 1;

            Some(value)
        } else {
            None
        }
    }

    pub fn remove(&mut self, index: ArenaIndex) -> Option<T> {
        if let Entry::Occupied { generation, .. } = self.arena.get(index.index)?
            && *generation == index.generation
        {
            self.remove_by_usize(index.index)
        } else {
            None
        }
    }
}
