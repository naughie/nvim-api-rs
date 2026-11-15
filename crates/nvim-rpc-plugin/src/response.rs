use crate::mp::encode::Encode;
use crate::mp::encode::primitive::State;

use std::convert::Infallible;

/// Converts a handler's result into success or error representations for an RPC reply.
///
/// `Result<T, E>` is supported when both branches implement `Encode`.
/// `()` is shorthand for `Ok(())`; both it and `Ok::<(), Infallible>(())` encode
/// a successful empty array. Use the explicit `Nil` primitive for a nil result.
/// Application response types can implement this trait by converting their
/// fields into built-in representations or untagged alternatives.
/// [`crate::mp::encode::Untagged`] combines alternatives such as an integer and
/// a string without a wire tag.
/// Responses move between handler and writer tasks and therefore require
/// `Send + 'static`.
///
/// ```
/// use nvim_rpc_plugin::IntoResponse;
/// use std::convert::Infallible;
///
/// struct Sum(i32);
///
/// impl IntoResponse for Sum {
///     type Value = i32;
///     type Error = Infallible;
///
///     fn into_response(self) -> Result<i32, Infallible> {
///         Ok(self.0)
///     }
/// }
/// ```
pub trait IntoResponse: Send + 'static {
    type Value: Encode<Encoder: State<Item: Send> + Send> + Send;
    type Error: Encode<Encoder: State<Item: Send> + Send> + Send;

    fn into_response(self) -> Result<Self::Value, Self::Error>;
}

impl IntoResponse for () {
    type Value = ();
    type Error = Infallible;

    fn into_response(self) -> Result<(), Infallible> {
        Ok(())
    }
}

impl<T, E> IntoResponse for Result<T, E>
where
    T: Encode<Encoder: State<Item: Send> + Send> + Send + 'static,
    E: Encode<Encoder: State<Item: Send> + Send> + Send + 'static,
{
    type Value = T;
    type Error = E;

    fn into_response(self) -> Self {
        self
    }
}
