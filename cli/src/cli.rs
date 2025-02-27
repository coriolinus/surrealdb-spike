use std::{os::unix::ffi::OsStrExt, path::PathBuf};

use anyhow::{Context, Result};
use checklist::{ChecklistId, ItemId};
use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub noun: Noun,

    /// Path to the database
    ///
    /// Default: "$XDG_DATA_HOME" if set or "$HOME/.local/share", then "checklist/db.sqlite3"
    #[arg(short, long)]
    path: Option<PathBuf>,

    /// Path to file containing encryption key for data at rest
    ///
    /// This file can contain arbitrary bytes which comprise the key for the database
    #[arg(short = 'E', long)]
    encryption_key_file: Option<PathBuf>,

    /// Encryption key for data at rest
    ///
    /// Default: "$USER@$NAME"
    #[arg(short, long, conflicts_with = "encryption_key_file")]
    encryption_key: Option<String>,
}

impl Cli {
    pub(crate) fn path(&self) -> Result<PathBuf> {
        if let Some(path) = &self.path {
            return Ok(path.clone());
        }

        Ok(dirs::data_local_dir()
            .context("data local dir must exist on this system")?
            .join("checklist/surrealdb"))
    }

    pub(crate) fn encryption_key(&self) -> Result<Vec<u8>> {
        if let Some(path) = &self.encryption_key_file {
            return std::fs::read(path).context("reading encryption key from file");
        }

        if let Some(key) = &self.encryption_key {
            return Ok(key.as_bytes().to_owned());
        }

        let mut out = Vec::new();
        out.extend_from_slice(std::env::var_os("USER").unwrap_or_default().as_bytes());
        out.push(b'@');
        out.extend_from_slice(std::env::var_os("NAME").unwrap_or_default().as_bytes());

        Ok(out)
    }
}

#[derive(Debug, Subcommand)]
pub enum Noun {
    /// Manage lists
    List(ListVerbAction),

    /// Manage items
    Item(ItemVerbAction),
}

#[derive(Debug, Args)]
pub struct ListVerbAction {
    #[command(subcommand)]
    pub verb: ListVerb,
}

#[derive(Debug, Subcommand)]
pub enum ListVerb {
    /// Show all checklists
    ShowAll(ShowAllChecklists),

    /// Create a new checklist
    New(NewChecklist),

    /// Delete a checklist
    Remove(RemoveChecklist),
}

#[derive(Debug, Args)]
pub struct ShowAllChecklists {}

#[derive(Debug, Args)]
pub struct NewChecklist {
    /// Name of this checklist
    pub name: String,
}

#[derive(Debug, Args)]
pub struct RemoveChecklist {
    /// Id of the checklist to remove
    pub id: ChecklistId,
}

#[derive(Debug, Args)]
pub struct ItemVerbAction {
    #[command(subcommand)]
    pub verb: ItemVerb,
}

#[derive(Debug, Subcommand)]
pub enum ItemVerb {
    /// Show all items in a checklist
    ShowAll(ShowAllItems),

    /// Create a new item in a checklist
    New(NewItem),

    /// Delete an item in a checklist
    Remove(RemoveItem),

    /// Toggle completion status of an item in a checklist
    Toggle(ToggleItem),
}

#[derive(Debug, Args)]
pub struct ShowAllItems {
    /// Checklist Id for items to show
    pub checklist_id: ChecklistId,

    /// When set, omit the item header
    #[arg(short, long)]
    pub omit_header: bool,
}

#[derive(Debug, Args)]
pub struct NewItem {
    /// Checklist Id in which to put this item
    pub checklist_id: ChecklistId,

    /// Name of this item
    pub name: String,
}

#[derive(Debug, Args)]
pub struct RemoveItem {
    /// Id of the item to remove
    pub id: ItemId,
}

#[derive(Debug, Args)]
pub struct ToggleItem {
    /// Id of the item to toggle
    pub id: ItemId,
}
