#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
#[cfg_attr(feature = "uniffi", uniffi(flat_error))]
#[cfg(not(feature = "wasm"))]
// errors in uniffi must be enums
pub enum Error {
    #[error(transparent)]
    Inner(#[from] checklist::Error),
}

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
#[wasm_bindgen]
#[cfg(feature = "wasm")]
pub struct Error(#[from] checklist::Error);

pub type Result<T, E = Error> = std::result::Result<T, E>;
