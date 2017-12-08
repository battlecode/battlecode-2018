//! Research contains info on the different research branches.

#[derive(Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum Branch {
    Economist,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
