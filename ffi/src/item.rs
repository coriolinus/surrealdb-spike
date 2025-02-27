#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

use crate::{
    ChecklistId, Db, Result,
    marc::{Marc, marc},
};

pub type ItemId = String;

#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct Item {
    inner: checklist::Item,
}

impl From<checklist::Item> for Item {
    fn from(inner: checklist::Item) -> Self {
        Self { inner }
    }
}

impl Item {
    pub(crate) fn marc(inner: checklist::Item) -> Marc<Self> {
        marc(Self { inner })
    }
}

async fn new_impl(db: &Db, checklist_id: ChecklistId, item: &str) -> Result<Item> {
    let checklist_id = checklist_id.parse()?;
    checklist::Item::new(db, checklist_id, item.to_owned())
        .await
        .map(Into::into)
        .map_err(Into::into)
}

#[cfg(feature = "uniffi")]
#[uniffi::export]
pub async fn item_new(db: &Db, checklist_id: ChecklistId, item: &str) -> Result<Item> {
    new_impl(db, checklist_id, item).await
}

async fn load_impl(db: &Db, item_id: ItemId) -> Result<Option<Marc<Item>>> {
    let item_id = item_id.parse()?;
    checklist::Item::load(db, item_id)
        .await
        .map(|option| option.map(Item::marc))
        .map_err(Into::into)
}

#[cfg(feature = "uniffi")]
#[uniffi::export]
pub async fn item_load(db: &Db, item_id: ItemId) -> Result<Option<Marc<Item>>> {
    load_impl(db, item_id).await
}

async fn delete_impl(db: &Db, item_id: ItemId) -> Result<()> {
    let item_id = item_id.parse()?;
    checklist::Item::delete(db, item_id)
        .await
        .map_err(Into::into)
}

#[cfg(feature = "uniffi")]
#[uniffi::export]
pub async fn item_delete(db: &Db, item_id: ItemId) -> Result<()> {
    delete_impl(db, item_id).await
}

// associated functions cannot be exported via uniffi
#[cfg(not(feature = "uniffi"))]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl Item {
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub async fn new(db: &Db, checklist_id: ChecklistId, item: &str) -> Result<Self> {
        new_impl(db, checklist_id, item).await
    }

    pub async fn load(db: &Db, item_id: ItemId) -> Result<Option<Marc<Item>>> {
        load_impl(db, item_id).await
    }

    pub async fn delete(db: &Db, item_id: ItemId) -> Result<()> {
        delete_impl(db, item_id).await
    }
}

#[cfg_attr(feature = "uniffi", uniffi::export)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl Item {
    pub async fn is_set(&self, db: &Db) -> Result<bool> {
        self.inner.is_set(db).await.map_err(Into::into)
    }

    pub async fn set_checked(&self, db: &Db, checked: bool) -> Result<()> {
        self.inner
            .set_checked(db, checked)
            .await
            .map_err(Into::into)
    }

    pub fn id(&self) -> ItemId {
        self.inner.id.to_string()
    }

    pub fn checklist_id(&self) -> ChecklistId {
        self.inner.checklist.to_string()
    }

    pub fn item(&self) -> String {
        self.inner.item.clone().into_owned()
    }
}
