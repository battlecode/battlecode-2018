//! Research is an invaluable asset to your army that upgrades and unlocks
//! capabilities of entities. Each upgrade takes a fixed number of rounds to
//! complete.

/// The entire research tree consists of multiple linear branches. Each branch
/// has an associated level, where Level 0 represents no research yet.
/// Performing an upgrade in a research branch will unlock the upgrade at the
/// next level in the branch.
#[derive(Debug, Serialize, Deserialize, Hash, PartialEq, Eq, Clone, Copy)]
pub enum Branch {
    Economist,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
