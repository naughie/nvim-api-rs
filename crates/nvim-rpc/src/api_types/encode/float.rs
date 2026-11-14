use super::LikeBaseType;
use super::sys;

impl LikeBaseType<sys::Float> for f32 {}
impl LikeBaseType<sys::Float> for f64 {}

impl LikeBaseType<sys::Float> for &f32 {}
impl LikeBaseType<sys::Float> for &f64 {}

impl LikeBaseType<sys::Object> for f32 {}
impl LikeBaseType<sys::Object> for f64 {}

impl LikeBaseType<sys::Object> for &f32 {}
impl LikeBaseType<sys::Object> for &f64 {}
