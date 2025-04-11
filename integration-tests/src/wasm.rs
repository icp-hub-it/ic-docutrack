use std::path::Path;

pub enum Canister {
    Backend,
}

impl Canister {
    pub fn as_path(&self) -> &'static Path {
        match self {
            Canister::Backend => Path::new("../.artifact/backend.wasm.gz"),
        }
    }
}
