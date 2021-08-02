use ahash::AHashMap;

use typed_arena::Arena;

pub struct Interner<'vm> {
    map: AHashMap<&'vm str, u32>,
    vec: Vec<&'vm str>,
    arena: &'vm Arena<u8>,
}

impl Interner<'_> {
    pub fn new(arena: &Arena<u8>) -> Interner {
        Interner {
            map: AHashMap::new(),
            vec: Vec::new(),
            arena,
        }
    }

    pub fn intern(&mut self, name: &str) -> u32 {
        if let Some(&idx) = self.map.get(name) {
            return idx;
        }
        let idx = self.vec.len() as u32;
        let name = self.arena.alloc_str(name);
        self.map.insert(name, idx);
        self.vec.push(name);

        debug_assert!(self.lookup(idx) == name);
        debug_assert!(self.intern(name) == idx);

        idx
    }

    pub fn exists(&self, string: &str) -> bool {
        self.map.contains_key(string)
    }

    pub fn get_existing(&self, name: &str) -> u32 {
        *self.map.get(name).expect("Interned string does not exist!")
    }

    pub fn lookup(&self, idx: u32) -> &str {
        self.vec[idx as usize]
    }
}
