extern crate alloc;
use arceos_api::random::ax_random;
use alloc::vec::Vec;
const INITIAL_CAPACITY: usize = 16;
const LOAD_FACTOR_NUM: usize = 7;
const LOAD_FACTOR_DEN: usize = 10;

///桶
pub struct Bucket<K, V> {
    data: Option<(K, V)>,
    deleted: bool,
}
///底层哈希表
pub struct HashMap<K, V> {
    map: Vec<Bucket<K, V>>,
    seed: u128,
    len: usize,
}

///哈希函数
fn hash<K: AsRef<[u8]>>(key: &K, seed: u128) -> u64 {
    let mut h = seed as u64;
    for &b in key.as_ref() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

impl<K, V> HashMap<K, V>
where
    K: AsRef<[u8]> + PartialEq,
{
    pub fn new() -> HashMap<K, V> {
        let seed = ax_random();
        let mut map = Vec::new();
        map.resize_with(INITIAL_CAPACITY, || Bucket {
            data: None,
            deleted: false,
        });
        HashMap { map, seed, len: 0 }
    }
    pub fn resize(&mut self){
        let new_cap = self.map.len() * 2;
        let old_map = core::mem::replace(&mut self.map, Vec::new());
        self.map.resize_with(new_cap, || Bucket {
            data: None,
            deleted: false,
        });
        self.len = 0;
        for bucket in old_map {
            if let Some((key, value)) = bucket.data {
                self.insert(key, value);
            }
        }
    }
    pub fn insert(&mut self, key: K, value: V) {
        if self.len * LOAD_FACTOR_DEN >= self.map.len() * LOAD_FACTOR_NUM {
            self.resize();
        }
        let h = hash(&key, self.seed);
        let mut index = (h as usize) % self.map.len();
        loop {
            if self.map[index].data.is_none() {
                // Empty 或 Delete 的位置都可以插入
                self.map[index].data = Some((key, value));
                self.map[index].deleted = false;
                self.len += 1;
                return;
            } else {
                // Occupied
                if self.map[index].data.as_ref().unwrap().0 == key {
                    // key 已存在，覆盖 value
                    self.map[index].data = Some((key, value));
                    return;
                }
                index = (index + 1) % self.map.len();
            }
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let h = hash(key, self.seed);
        let mut index = (h as usize) % self.map.len();
        loop {
            if self.map[index].data.is_none() {
                if !self.map[index].deleted {
                    // 真正的 Empty，key 不存在
                    return None;
                }
                // Delete 的位置，继续往后找
                index = (index + 1) % self.map.len();
            } else {
                if self.map[index].data.as_ref().unwrap().0 == *key {
                    return Some(&self.map[index].data.as_ref().unwrap().1);
                }
                index = (index + 1) % self.map.len();
            }
        }
    }

    pub fn remove(&mut self, key: &K) -> bool {
        if self.map.is_empty() {
            return false;
        }
        let h = hash(key, self.seed);
        let mut index = (h as usize) % self.map.len();
        loop {
            if self.map[index].data.is_none() {
                if !self.map[index].deleted {
                    return false;
                }
                index = (index + 1) % self.map.len();
            } else {
                if self.map[index].data.as_ref().unwrap().0 == *key {
                    self.map[index].data = None;
                    self.map[index].deleted = true;
                    self.len -= 1;
                    return true;
                }
                index = (index + 1) % self.map.len();
            }
        }
    }

    pub fn iter(&self) -> Iter<K, V> {
        Iter {
            map: &self.map,
            index: 0,
        }
    }
}

/// 迭代器
pub struct Iter<'a, K, V> {
    map: &'a [Bucket<K, V>],
    index: usize,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.map.len() {
            let bucket = &self.map[self.index];
            self.index += 1;
            if let Some((ref key, ref value)) = bucket.data {
                return Some((key, value));
            }
        }
        None
    }
}