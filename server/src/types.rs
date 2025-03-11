use spacetimedb::{Identity, SpacetimeType};

#[derive(SpacetimeType, PartialEq)]
pub enum Permission {
    /// Read guild channel by id
    Read(i128),
    /// Write to guild channel by id
    Write(i128),
}

#[derive(SpacetimeType)]
pub struct TwoUsers {
    pub id_a: Identity,
    pub id_b: Identity,
}

impl TwoUsers {
    pub fn new(id_a: Identity, id_b: Identity) -> Self {
        let a = id_a.min(id_b);
        let b = id_a.max(id_b);

        Self { id_a: a, id_b: b }
    }
}
