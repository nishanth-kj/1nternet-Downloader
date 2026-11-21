/// Base trait for domain enums featuring code and value pair mappings
pub trait BaseEnum<C, V>: Sized + Send + Sync {
    /// Returns the unique numeric or typed code
    fn code(&self) -> C;

    /// Returns the canonical display or protocol string value
    fn value(&self) -> V;

    /// Resolve an enum variant from its code
    fn from_code(code: C) -> Option<Self>;

    /// Resolve an enum variant from its string value
    fn from_value(value: &str) -> Option<Self>;
}

/// Macro that defines code-value enums taking parameters like a constructor.
/// To add new values, simply append a new line: `Variant(code, "Value"),`
#[macro_export]
macro_rules! define_code_value_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident ($code_type:ty, $val_type:ty) {
            $(
                $(#[$vmeta:meta])*
                $variant:ident($code:expr, $val:expr)
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
        $vis enum $name {
            $(
                $(#[$vmeta])*
                $variant,
            )*
        }

        impl $name {
            pub const ALL: &'static [Self] = &[
                $( Self::$variant, )*
            ];

            pub const fn code(&self) -> $code_type {
                match self {
                    $( Self::$variant => $code, )*
                }
            }

            pub const fn value(&self) -> $val_type {
                match self {
                    $( Self::$variant => $val, )*
                }
            }

            pub fn from_code(code: $code_type) -> Option<Self> {
                match code {
                    $( c if c == $code => Some(Self::$variant), )*
                    _ => None,
                }
            }

            pub fn from_value(val: &str) -> Option<Self> {
                $(
                    if val.eq_ignore_ascii_case($val) {
                        return Some(Self::$variant);
                    }
                )*
                None
            }

            /// Resolve from numeric integer using BaseEnum mapping, falling back to final variant
            pub fn from_i32(val: i32) -> Self {
                Self::from_code(val as $code_type).unwrap_or(Self::ALL[Self::ALL.len() - 1])
            }

            /// Return code as i32
            pub fn to_i32(&self) -> i32 {
                self.code() as i32
            }

            /// Return display string representation
            pub fn as_str(&self) -> $val_type {
                self.value()
            }
        }

        impl $crate::utils::BaseEnum<$code_type, $val_type> for $name {
            fn code(&self) -> $code_type {
                self.code()
            }

            fn value(&self) -> $val_type {
                self.value()
            }

            fn from_code(code: $code_type) -> Option<Self> {
                Self::from_code(code)
            }

            fn from_value(value: &str) -> Option<Self> {
                Self::from_value(value)
            }
        }
    };
}
