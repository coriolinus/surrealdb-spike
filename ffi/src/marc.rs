/// Marc is a "maybe Arc".
///
/// Uniffi requires that we wrap our types in [`Arc`][std::sync::Arc] in several circumstances.
/// However, we don't fundamentally need or desire to perform this wrapping for our own purposes;
/// it is purely a uniffi implementation detail. In particular, wasm-bindgen doesn't work properly
/// when our stuff is Arc-wrapped; `Arc<T>` doesn't implement `Into<JsValue>`. So outside the
/// uniffi context, a `Marc<T>` is just `T`.
#[cfg(feature = "uniffi")]
pub(crate) type Marc<T> = std::sync::Arc<T>;

/// Marc is a "maybe Arc".
///
/// Uniffi requires that we wrap our types in [`Arc`][std::sync::Arc] in several circumstances.
/// However, we don't fundamentally need or desire to perform this wrapping for our own purposes;
/// it is purely a uniffi implementation detail. In particular, wasm-bindgen doesn't work properly
/// when our stuff is Arc-wrapped; `Arc<T>` doesn't implement `Into<JsValue>`. So outside the
/// uniffi context, a `Marc<T>` is just `T`.
#[cfg(not(feature = "uniffi"))]
pub(crate) type Marc<T> = T;

#[inline]
#[cfg(feature = "uniffi")]
pub(crate) fn marc<T>(t: T) -> Marc<T> {
    Marc::new(t)
}

#[inline]
#[cfg(not(feature = "uniffi"))]
pub(crate) fn marc<T>(t: T) -> Marc<T> {
    t
}
