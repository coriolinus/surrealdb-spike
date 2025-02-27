use std::{borrow::Cow, path::Path, str::FromStr};

use futures::{TryFutureExt as _, future::try_join_all};
use surrealdb::{RecordId, RecordIdKey};

// `local::Db` specifies an embedded database.
type Database = surrealdb::Surreal<surrealdb::engine::local::Db>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{context}: {inner}")]
    Surreal {
        context: &'static str,
        #[source]
        inner: surrealdb::Error,
    },
    #[error("this item is not present in the db; it may have been deleted")]
    MissingItem,
    #[error("wrong record id type: expected \"{expected}\"; got \"{got}\"")]
    WrongRecordId { expected: &'static str, got: String },
    #[error("creating a {resource} did not return an instance of that resource")]
    FailedCreate { resource: &'static str },
    #[error("updating a {resource} did not return an instance of that resource")]
    FailedUpdate { resource: &'static str },
    #[error("a live {resource} had an unset id field")]
    MissingId { resource: &'static str },
}

impl Error {
    pub(crate) fn surreal(context: &'static str) -> impl FnOnce(surrealdb::Error) -> Self {
        move |inner| Self::Surreal { context, inner }
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub struct Db {
    inner: Database,
    _encryption_key: Vec<u8>,
}

impl Db {
    pub async fn new(path: impl AsRef<Path>, encryption_key: &[u8]) -> Result<Self> {
        // In the real implementation we could do transparent item-level encryption and decryption;
        // we have an implementation which does this in the indexdb on wasm already.
        //
        // Alternatively, though `Connection` is a sealed trait in surrealdb, we could potentially
        // implement and contribute an encrypted connection wrapper which does block-level encryption
        // instead of worrying about file-level encryption.
        //
        // For the purpose of this spike, we will not do any of that, and just pretend that it's already accomplished.
        let encryption_key = encryption_key.to_owned();

        let mut capabilities = surrealdb::opt::capabilities::Capabilities::default();
        capabilities.allow_experimental_feature(
            // pseudo joins / back-references
            surrealdb::opt::capabilities::ExperimentalFeature::RecordReferences,
        );
        let config = surrealdb::opt::Config::new()
            .strict()
            .capabilities(capabilities);
        let inner = Database::new::<surrealdb::engine::local::RocksDb>((path.as_ref(), config))
            .await
            .map_err(Error::surreal("connecting to database"))?;
        inner
            .use_ns("wire")
            .use_db("checklist")
            .await
            .map_err(Error::surreal("seelecting database"))?;

        let db = Self {
            inner,
            _encryption_key: encryption_key,
        };
        db.ensure_schema().await?;

        Ok(db)
    }

    async fn ensure_schema(&self) -> Result<()> {
        const SCHEMA: &str = include_str!("schema.surreal");
        for command in SCHEMA.split("\n\n") {
            self.inner
                .query(command)
                .await
                .map_err(Error::surreal("executing schema"))?;
        }

        Ok(())
    }
}

const CHECKLIST_TABLE: &str = "checklist";

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    derive_more::Constructor,
    derive_more::Deref,
    derive_more::From,
    derive_more::Into,
    derive_more::Display,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(try_from = "RecordId", into = "RecordId")]
pub struct ChecklistId(RecordIdKey);

impl FromStr for ChecklistId {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self(s.into()))
    }
}

impl From<ChecklistId> for RecordId {
    fn from(value: ChecklistId) -> Self {
        RecordId::from_table_key(CHECKLIST_TABLE, value.0)
    }
}

impl TryFrom<RecordId> for ChecklistId {
    type Error = Error;

    fn try_from(value: RecordId) -> std::result::Result<Self, Self::Error> {
        if value.table() != CHECKLIST_TABLE {
            return Err(Error::WrongRecordId {
                expected: CHECKLIST_TABLE,
                got: value.table().to_owned(),
            });
        }
        Ok(Self(value.key().to_owned()))
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Checklist {
    pub id: Option<ChecklistId>,
    pub name: Cow<'static, str>,
    pub items: Vec<ItemId>,
}

impl Checklist {
    pub async fn new(db: &Db, name: impl Into<Cow<'static, str>>) -> Result<Self> {
        let id = None;
        let name = name.into();
        let items = Vec::new();

        let checklist = Self { id, name, items };

        db.inner
            .create(CHECKLIST_TABLE)
            .content(checklist)
            .await
            .map_err(Error::surreal("creating checklist"))?
            .ok_or(Error::FailedCreate {
                resource: CHECKLIST_TABLE,
            })
    }

    pub async fn load(db: &Db, id: ChecklistId) -> Result<Option<Self>> {
        let resource = RecordId::from(id);
        db.inner
            .select(resource)
            .await
            .map_err(Error::surreal("loading checklist"))
    }

    pub async fn all(db: &Db) -> Result<Vec<Self>> {
        db.inner
            .select(CHECKLIST_TABLE)
            .await
            .map_err(Error::surreal("loading all checklists"))
    }

    pub async fn delete(db: &Db, id: ChecklistId) -> Result<()> {
        let resource = RecordId::from(id);
        db.inner
            .delete::<Option<Checklist>>(resource)
            .await
            .map_err(Error::surreal("deleting checklist"))?;
        Ok(())
    }

    pub async fn items(&self, db: &Db) -> Result<Vec<Item>> {
        let id = self.id.clone().ok_or(Error::MissingId {
            resource: CHECKLIST_TABLE,
        })?;
        let fresh = Self::load(db, id).await?.ok_or(Error::MissingItem)?;
        try_join_all(fresh.items.into_iter().map(|id| {
            Item::load(db, id).and_then(async |maybe_item| maybe_item.ok_or(Error::MissingItem))
        }))
        .await
    }
}

const ITEM_TABLE: &str = "item";

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    derive_more::Constructor,
    derive_more::Deref,
    derive_more::From,
    derive_more::Into,
    derive_more::Display,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(try_from = "RecordId", into = "RecordId")]
pub struct ItemId(RecordIdKey);

impl FromStr for ItemId {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self(s.into()))
    }
}

impl From<ItemId> for RecordId {
    fn from(value: ItemId) -> Self {
        RecordId::from_table_key(ITEM_TABLE, value.0)
    }
}

impl TryFrom<RecordId> for ItemId {
    type Error = Error;

    fn try_from(value: RecordId) -> std::result::Result<Self, Self::Error> {
        if value.table() != ITEM_TABLE {
            return Err(Error::WrongRecordId {
                expected: ITEM_TABLE,
                got: value.table().to_owned(),
            });
        }
        Ok(Self(value.key().to_owned()))
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Item {
    pub id: Option<ItemId>,
    pub checklist: ChecklistId,
    pub item: Cow<'static, str>,
    checked: bool,
}

impl Item {
    pub async fn new(
        db: &Db,
        checklist: ChecklistId,
        item: impl Into<Cow<'static, str>>,
    ) -> Result<Self> {
        let id = None;
        let item = item.into();
        let checked = false;

        let item = Self {
            id,
            checklist,
            item,
            checked,
        };

        db.inner
            .create(ITEM_TABLE)
            .content(item)
            .await
            .map_err(Error::surreal("creating item"))?
            .ok_or(Error::FailedCreate {
                resource: ITEM_TABLE,
            })
    }

    pub async fn load(db: &Db, id: ItemId) -> Result<Option<Self>> {
        let resource = RecordId::from(id);
        db.inner
            .select(resource)
            .await
            .map_err(Error::surreal("loading item"))
    }

    pub async fn delete(db: &Db, id: ItemId) -> Result<()> {
        let resource = RecordId::from(id);
        db.inner
            .delete::<Option<Self>>(resource)
            .await
            .map_err(Error::surreal("deleting item"))?;
        Ok(())
    }

    pub async fn is_set(&self, db: &Db) -> Result<bool> {
        let id = self.id.clone().ok_or(Error::MissingId {
            resource: ITEM_TABLE,
        })?;
        Self::load(db, id)
            .await
            .map(|maybe_item| maybe_item.is_some_and(|item| item.checked))
    }

    pub async fn set_checked(&mut self, db: &Db, checked: bool) -> Result<()> {
        let id = RecordId::from(self.id.clone().ok_or(Error::MissingId {
            resource: ITEM_TABLE,
        })?);

        self.checked = checked;

        db.inner
            .update::<Option<Self>>(id)
            .content(self.clone())
            .await
            .map_err(Error::surreal("updating checked item"))?
            .ok_or(Error::FailedUpdate {
                resource: ITEM_TABLE,
            })?;

        Ok(())
    }
}
