#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

use crate::{
    Db, Item, Result,
    marc::{Marc, marc},
};

pub type ChecklistId = String;

#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct Checklist {
    inner: checklist::Checklist,
}

impl From<checklist::Checklist> for Checklist {
    fn from(inner: checklist::Checklist) -> Self {
        Self { inner }
    }
}

impl Checklist {
    pub(crate) fn marc(inner: checklist::Checklist) -> Marc<Self> {
        marc(Self { inner })
    }
}

async fn new_impl(db: &Db, name: &str) -> Result<Checklist> {
    checklist::Checklist::new(db, name.to_owned())
        .await
        .map(Into::into)
        .map_err(Into::into)
}

#[cfg(feature = "uniffi")]
#[uniffi::export]
pub async fn checklist_new(db: &Db, name: &str) -> Result<Checklist> {
    new_impl(db, name).await
}

async fn load_impl(db: &Db, id: ChecklistId) -> Result<Option<Marc<Checklist>>> {
    let id = id.parse()?;
    checklist::Checklist::load(db, id)
        .await
        .map(|option| option.map(Checklist::marc))
        .map_err(Into::into)
}

#[cfg(feature = "uniffi")]
#[uniffi::export]
pub async fn checklist_load(db: &Db, id: ChecklistId) -> Result<Option<Marc<Checklist>>> {
    load_impl(db, id).await
}

async fn all_impl(db: &Db) -> Result<Vec<Marc<Checklist>>> {
    checklist::Checklist::all(db)
        .await
        .map(|ok| ok.into_iter().map(Checklist::marc).collect())
        .map_err(Into::into)
}

#[cfg(feature = "uniffi")]
#[uniffi::export]
pub async fn checklist_all(db: &Db) -> Result<Vec<Marc<Checklist>>> {
    all_impl(db).await
}

async fn delete_impl(db: &Db, id: ChecklistId) -> Result<()> {
    let id = id.parse()?;
    checklist::Checklist::delete(db, id)
        .await
        .map_err(Into::into)
}

#[cfg(feature = "uniffi")]
#[uniffi::export]
pub async fn checklist_delete(db: &Db, id: ChecklistId) -> Result<()> {
    delete_impl(db, id).await
}

// associated functions cannot be exported via uniffi
#[cfg(not(feature = "uniffi"))]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl Checklist {
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub async fn new(db: &Db, name: &str) -> Result<Self> {
        new_impl(db, name).await
    }

    pub async fn load(db: &Db, id: ChecklistId) -> Result<Option<Marc<Checklist>>> {
        load_impl(db, id).await
    }

    pub async fn all(db: &Db) -> Result<Vec<Marc<Checklist>>> {
        all_impl(db).await
    }

    pub async fn delete(db: &Db, id: ChecklistId) -> Result<()> {
        delete_impl(db, id).await
    }
}

#[cfg_attr(feature = "uniffi", uniffi::export)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl Checklist {
    pub async fn items(&self, db: &Db) -> Result<Vec<Marc<Item>>> {
        self.inner
            .items(db)
            .await
            .map(|items| items.into_iter().map(Item::marc).collect())
            .map_err(Into::into)
    }

    pub fn id(&self) -> ChecklistId {
        self.inner.id.to_string()
    }

    pub fn name(&self) -> String {
        self.inner.name.clone().into_owned()
    }
}
