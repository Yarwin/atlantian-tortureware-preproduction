use godot::meta::{GodotType, PropertyHintInfo};
use godot::prelude::*;
use godot::sys::GodotNullableFfi;

pub struct Required<T> {
    inner: Option<T>,
}

impl<T> Default for Required<T> {
    fn default() -> Self {
        Required { inner: None }
    }
}

impl<T> std::ops::Deref for Required<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match &self.inner {
            None => panic!(),
            Some(v) => v,
        }
    }
}

impl<T> std::ops::DerefMut for Required<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match &mut self.inner {
            None => panic!(),
            Some(v) => v,
        }
    }
}

impl<T: GodotConvert> GodotConvert for Required<T>
where
    Option<T::Via>: GodotType,
{
    type Via = Option<T::Via>;
}

impl<T: ToGodot> ToGodot for Required<T>
where
    Option<T::Via>: GodotType,
{
    type ToVia<'v> = Self::Via where T: 'v;

    fn to_godot(&self) -> Self::Via {
        unimplemented!();
    }

    fn to_variant(&self) -> Variant {
        unimplemented!();
    }
}

impl<T: FromGodot> FromGodot for Required<T>
where
    Option<T::Via>: GodotType,
{
    fn try_from_godot(via: Self::Via) -> Result<Self, ConvertError> {
        match Option::<T>::try_from_godot(via) {
            Ok(val) => Ok(Required { inner: val }),
            Err(e) => Err(e),
        }
    }

    fn from_godot(via: Self::Via) -> Self {
        Required {
            inner: Option::<T>::from_godot(via),
        }
    }

    fn try_from_variant(variant: &Variant) -> Result<Self, ConvertError> {
        Ok(Required {
            inner: Option::<T>::try_from_variant(variant)?,
        })
    }

    fn from_variant(variant: &Variant) -> Self {
        Required {
            inner: Option::<T>::from_variant(variant),
        }
    }
}

impl<T> Var for Required<T>
where
    T: Var + FromGodot,
    Option<T>: GodotConvert<Via = Option<T::Via>>,
    Required<T>: GodotConvert<Via = Option<T::Via>>,
{
    fn get_property(&self) -> Self::Via {
        Option::<T>::get_property(&self.inner)
    }

    fn set_property(&mut self, value: Self::Via) {
        Option::<T>::set_property(&mut self.inner, value as Self::Via);
    }
}

impl<T> Export for Required<T>
where
    T: Export + GodotType,
    T::Ffi: GodotNullableFfi,
    Option<T>: Var,
    Required<T>: Var,
{
    fn export_hint() -> PropertyHintInfo {
        Option::<T>::export_hint()
    }
}
