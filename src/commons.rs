pub type HASH = u64;
pub type H1 = u64;
pub type H2 = u8;
pub type Metadata = [H2; 8];

#[derive(Debug, Clone)]
pub struct Group<K, V> {
    pub keys: [Option<K>; 8],
    pub values: [Option<V>; 8],
}