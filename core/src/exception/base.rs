use std::any::Any;
use std::error::Error;
use std::fmt::{self, Debug, Display};
use std::sync::Arc;

/// A type-erased thread-safe container holding any situational payload object.
#[derive(Clone)]
pub struct AnyExceptionObject(pub Arc<dyn Any + Send + Sync>);

impl AnyExceptionObject {
    /// Wrap any value into an AnyExceptionObject
    pub fn new<T: 'static + Send + Sync>(value: T) -> Self {
        Self(Arc::new(value))
    }

    /// Attempt to downcast the stored object to a reference of concrete type `T`
    pub fn downcast_ref<T: 'static + Send + Sync>(&self) -> Option<&T> {
        self.0.downcast_ref::<T>()
    }
}

impl Debug for AnyExceptionObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AnyExceptionObject({:?})", self.0.type_id())
    }
}

/// Base trait for all exceptions across 1nternet Downloader.
/// Every exception provides:
/// - `code`: numeric error code
/// - `value`: canonical error string/value
/// - `object`: situational context object (can be anything for that situation!)
pub trait BaseException: Error + Send + Sync {
    /// Return the numeric error code
    fn code(&self) -> i32;

    /// Return the canonical string identifier/value
    fn value(&self) -> &str;

    /// Return the situational object reference, if any
    fn object(&self) -> Option<&(dyn Any + Send + Sync)> {
        None
    }

    /// Convenience helper to downcast the situational object to type `T`
    fn downcast_object<T: 'static>(&self) -> Option<&T> {
        self.object().and_then(|obj| obj.downcast_ref::<T>())
    }
}

/// A generic base error holding code, value, message, and an arbitrary situational object.
#[derive(Debug, Clone)]
pub struct BaseError {
    pub code: i32,
    pub value: &'static str,
    pub message: String,
    pub object: Option<AnyExceptionObject>,
}

impl BaseError {
    pub fn new(code: i32, value: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            value,
            message: message.into(),
            object: None,
        }
    }

    pub fn with_object<T: 'static + Send + Sync>(mut self, object: T) -> Self {
        self.object = Some(AnyExceptionObject::new(object));
        self
    }
}

impl Display for BaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{} - {}] {}", self.code, self.value, self.message)
    }
}

impl Error for BaseError {}

impl BaseException for BaseError {
    fn code(&self) -> i32 {
        self.code
    }

    fn value(&self) -> &str {
        self.value
    }

    fn object(&self) -> Option<&(dyn Any + Send + Sync)> {
        self.object.as_ref().map(|o| &*o.0 as &(dyn Any + Send + Sync))
    }
}
