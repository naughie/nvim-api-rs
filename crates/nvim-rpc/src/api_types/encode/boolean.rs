use super::LikeBaseType;
use super::sys;

impl LikeBaseType<sys::Bool> for bool {}
impl LikeBaseType<sys::Bool> for &bool {}

impl LikeBaseType<sys::Object> for bool {}
impl LikeBaseType<sys::Object> for &bool {}
