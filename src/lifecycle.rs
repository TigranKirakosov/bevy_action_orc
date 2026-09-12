use bevy::prelude::*;
use std::marker::PhantomData;

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct Started<T: FromReflect + TypePath + Default>(#[reflect(ignore)] PhantomData<T>);

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct Finished<T: FromReflect + TypePath + Default>(#[reflect(ignore)] PhantomData<T>);
