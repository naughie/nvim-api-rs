use super::LikeBaseType;
use super::sys;

impl LikeBaseType<sys::Int> for u8 {}
impl LikeBaseType<sys::Int> for u16 {}
impl LikeBaseType<sys::Int> for u32 {}
impl LikeBaseType<sys::Int> for u64 {}
#[cfg(any(
    target_pointer_width = "16",
    target_pointer_width = "32",
    target_pointer_width = "64"
))]
impl LikeBaseType<sys::Int> for usize {}

impl LikeBaseType<sys::Int> for i8 {}
impl LikeBaseType<sys::Int> for i16 {}
impl LikeBaseType<sys::Int> for i32 {}
impl LikeBaseType<sys::Int> for i64 {}
#[cfg(any(
    target_pointer_width = "16",
    target_pointer_width = "32",
    target_pointer_width = "64"
))]
impl LikeBaseType<sys::Int> for isize {}

impl LikeBaseType<sys::Int> for &u8 {}
impl LikeBaseType<sys::Int> for &u16 {}
impl LikeBaseType<sys::Int> for &u32 {}
impl LikeBaseType<sys::Int> for &u64 {}
#[cfg(any(
    target_pointer_width = "16",
    target_pointer_width = "32",
    target_pointer_width = "64"
))]
impl LikeBaseType<sys::Int> for &usize {}

impl LikeBaseType<sys::Int> for &i8 {}
impl LikeBaseType<sys::Int> for &i16 {}
impl LikeBaseType<sys::Int> for &i32 {}
impl LikeBaseType<sys::Int> for &i64 {}
#[cfg(any(
    target_pointer_width = "16",
    target_pointer_width = "32",
    target_pointer_width = "64"
))]
impl LikeBaseType<sys::Int> for &isize {}

impl LikeBaseType<sys::Object> for u8 {}
impl LikeBaseType<sys::Object> for u16 {}
impl LikeBaseType<sys::Object> for u32 {}
impl LikeBaseType<sys::Object> for u64 {}
#[cfg(any(
    target_pointer_width = "16",
    target_pointer_width = "32",
    target_pointer_width = "64"
))]
impl LikeBaseType<sys::Object> for usize {}

impl LikeBaseType<sys::Object> for i8 {}
impl LikeBaseType<sys::Object> for i16 {}
impl LikeBaseType<sys::Object> for i32 {}
impl LikeBaseType<sys::Object> for i64 {}
#[cfg(any(
    target_pointer_width = "16",
    target_pointer_width = "32",
    target_pointer_width = "64"
))]
impl LikeBaseType<sys::Object> for isize {}

impl LikeBaseType<sys::Object> for &u8 {}
impl LikeBaseType<sys::Object> for &u16 {}
impl LikeBaseType<sys::Object> for &u32 {}
impl LikeBaseType<sys::Object> for &u64 {}
#[cfg(any(
    target_pointer_width = "16",
    target_pointer_width = "32",
    target_pointer_width = "64"
))]
impl LikeBaseType<sys::Object> for &usize {}

impl LikeBaseType<sys::Object> for &i8 {}
impl LikeBaseType<sys::Object> for &i16 {}
impl LikeBaseType<sys::Object> for &i32 {}
impl LikeBaseType<sys::Object> for &i64 {}
#[cfg(any(
    target_pointer_width = "16",
    target_pointer_width = "32",
    target_pointer_width = "64"
))]
impl LikeBaseType<sys::Object> for &isize {}
