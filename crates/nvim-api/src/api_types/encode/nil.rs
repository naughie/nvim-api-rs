use super::LikeBaseType;
use super::mp_prim;
use super::sys;

impl LikeBaseType<sys::Nil> for mp_prim::Nil {}
impl LikeBaseType<sys::Nil> for &mp_prim::Nil {}

impl LikeBaseType<sys::Object> for mp_prim::Nil {}
impl LikeBaseType<sys::Object> for &mp_prim::Nil {}
