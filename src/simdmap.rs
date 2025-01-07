use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    simd::prelude::SimdPartialEq,
    simd::u8x8,
};

use crate::commons::{Group, Metadata, H1, H2, HASH};

pub struct SimdTable<K, V> {
    ctrl: Vec<Metadata>,
    groups: Vec<Group<K, V>>,
    group_count: usize,
    cap: usize,
}

impl<K: Hash + Eq + Clone, V: Clone> SimdTable<K, V> {
    pub const MASK_H1: HASH = 0xFFFF_FFFF_FFFF_FF80;
    pub const MASK_H2: HASH = 0x7F;
    pub const MASK_EXISTS: H2 = 0b1000_0000;
    pub const MASK_DATA: H2 = 0b0111_1111;
    pub const EMPTY: H2 = 0b1000_0000;
    pub const DELETED: H2 = 0b1111_1110;

    pub fn new(cap: usize) -> Self {
        // group_count = ceil(cap / 8) 과 동일
        let group_count = ((cap + 7) / 8);

        SimdTable {
            ctrl: vec![[Self::EMPTY; 8]; group_count],
            groups: vec![
                Group {
                    keys: [None, None, None, None, None, None, None, None],
                    values: [None, None, None, None, None, None, None, None],
                };
                group_count
            ],
            group_count,
            cap,
        }
    }

    fn start_probe(&self, hash: H1) -> usize {
        (hash as usize) % self.group_count
    }

    // 해시 계산 (간단한 해시 함수)
    fn hash(&self, key: &K) -> HASH {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish() as HASH
    }

    // 해시된 값을 상위 57비트와 하위 7비트로 분리 각각 H1, H2로 반환
    fn hash_split(&self, hash: HASH) -> (H1, H2) {
        let h1 = (hash & Self::MASK_H1) >> 7 as H1;
        let h2 = (hash & Self::MASK_H2) as H2;
        (h1, h2)
    }

    fn match_h2(ctrl: &Metadata, h2: H2) -> [bool; 8] {
        let mut result = [false; 8];
        let h2x8 = u8x8::splat(h2);
        let ctrl = u8x8::from_slice(ctrl);
        let matches = h2x8.simd_eq(ctrl);
        matches.to_array()
    }

    fn empty_meta(ctrl: &Metadata, h2: H2) -> [bool; 8] {
        let mut result = [false; 8];
        for i in 0..8 {
            if (ctrl[i] & Self::MASK_EXISTS) == Self::EMPTY {
                result[i] = true;
            }
        }
        result
    }

    // 빈 슬롯 탐색
    pub fn insert(&mut self, k: K, v: V) {
        let hash = self.hash(&k);
        let (h1, h2) = self.hash_split(hash as HASH);
        let group_start = self.start_probe(h1);

        for i in 0..self.group_count {
            let group_idx = (group_start + i) % self.group_count;
            let ctrl_matches = Self::match_h2(&self.ctrl[group_idx], h2);

            // ctrl에 일치하는 값이 있는지 확인
            for j in 0..8 {
                if ctrl_matches[j] {
                    // 만약 해당 슬롯에 정확히 일치하는 키가 있는 경우 값 대치
                    if let Some(key) = &self.groups[group_idx].keys[j] {
                        if key == &k {
                            self.groups[group_idx].values[j] = Some(v);
                            return;
                        }
                    }
                }
            }

            // 슬롯이 비어 있는 경우
            let empty_slots = Self::empty_meta(&self.ctrl[group_idx], Self::EMPTY);
            for j in 0..8 {
                if empty_slots[j] {
                    self.ctrl[group_idx][j] = h2;
                    self.groups[group_idx].keys[j] = Some(k);
                    self.groups[group_idx].values[j] = Some(v);
                    return;
                }
            }
        }
    }

    pub fn get(&self, k: &K) -> Option<&V> {
        let hash = self.hash(k);
        let (h1, h2) = self.hash_split(hash as HASH);
        let group_start = self.start_probe(h1);

        for i in 0..self.group_count {
            let group_idx = (group_start + i) % self.group_count;
            let ctrl_matches = Self::match_h2(&self.ctrl[group_idx], h2);

            // ctrl에 일치하는 값이 있는지 확인
            for j in 0..8 {
                if ctrl_matches[j] {
                    if let Some(key) = &self.groups[group_idx].keys[j] {
                        if key == k {
                            return self.groups[group_idx].values[j].as_ref();
                        }
                    }
                }
            }
        }
        None
    }
}
