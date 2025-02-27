#[cfg(all(feature = "wasm", feature = "uniffi"))]
compile_error!(
    "can't build this crate for uniffi and wasm simultaneously; their Error types are incompatible"
);

mod checklist;
mod error;
mod item;
pub(crate) mod marc;

use ::checklist as libchecklist;
use std::ops::Deref;

pub use checklist::{Checklist, ChecklistId};
pub use error::{Error, Result};
pub use item::{Item, ItemId};

#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!("checklist_ffi");

#[cfg(feature = "uniffi")]
pub use checklist::{checklist_all, checklist_delete, checklist_load, checklist_new};

#[cfg(feature = "uniffi")]
pub use item::{item_delete, item_load, item_new};

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct Db {
    inner: libchecklist::Db,
}

#[cfg_attr(feature = "uniffi", uniffi::export)]
pub async fn db_new(path: &str, encryption_key: Vec<u8>) -> Result<Db> {
    libchecklist::Db::new(path, &encryption_key)
        .await
        .map(|inner| Db { inner })
        .map_err(Into::into)
}

#[cfg_attr(feature = "uniffi", uniffi::export)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl Db {
    #[cfg_attr(feature = "uniffi", uniffi::constructor)]
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub async fn new(path: &str, encryption_key: Vec<u8>) -> Result<Db> {
        db_new(path, encryption_key).await
    }
}

impl Deref for Db {
    type Target = libchecklist::Db;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
