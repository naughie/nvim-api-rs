use super::LikeBaseType;
use super::sys;

impl LikeBaseType<sys::String> for &str {}
impl LikeBaseType<sys::Object> for &str {}
impl LikeBaseType<sys::String> for &&str {}
impl LikeBaseType<sys::Object> for &&str {}

impl LikeBaseType<sys::String> for &[u8] {}
impl LikeBaseType<sys::Object> for &[u8] {}
impl LikeBaseType<sys::String> for Vec<u8> {}
impl LikeBaseType<sys::Object> for Vec<u8> {}
impl LikeBaseType<sys::String> for &Vec<u8> {}
impl LikeBaseType<sys::Object> for &Vec<u8> {}

impl<I: ExactSizeIterator<Item = u8>> LikeBaseType<sys::String> for super::mp_prim::BinIter<I> {}
impl<I: ExactSizeIterator<Item = u8>> LikeBaseType<sys::Object> for super::mp_prim::BinIter<I> {}

impl LikeBaseType<sys::String> for String {}
impl LikeBaseType<sys::Object> for String {}

impl LikeBaseType<sys::String> for &String {}
impl LikeBaseType<sys::Object> for &String {}
