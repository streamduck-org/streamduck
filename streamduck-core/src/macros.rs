/// Shorthand for TypeId::of::<T>()
#[macro_export]
macro_rules! type_of {
    ($t:ty) => {
        std::any::TypeId::of::<$t>()
    }
}