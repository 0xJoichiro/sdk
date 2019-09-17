use IDLType;
use types::*;
use Serializer;

macro_rules! primitive_impl {
    ($t:ty, $id:tt, $method:ident $($cast:tt)*) => {
        impl IDLType for $t {
            fn id() -> TypeId { TypeId::of::<$t>() }            
            fn _ty() -> Type { Type::$id }
            fn idl_serialize<S>(&self, serializer: S) -> Result<(), S::Error> where S: Serializer {
                serializer.$method(*self $($cast)*)
            }
        }
    };
}

primitive_impl!(bool, Bool, serialize_bool);
primitive_impl!(i8, Int, serialize_int as i64);
primitive_impl!(i16, Int, serialize_int as i64);
primitive_impl!(i32, Int, serialize_int as i64);
primitive_impl!(i64, Int, serialize_int);
primitive_impl!(isize, Int, serialize_int as i64);
primitive_impl!(u8, Nat, serialize_nat as u64);
primitive_impl!(u16, Nat, serialize_nat as u64);
primitive_impl!(u32, Nat, serialize_nat as u64);
primitive_impl!(u64, Nat, serialize_nat);
primitive_impl!(usize, Nat, serialize_nat as u64);
primitive_impl!(&str, Text, serialize_text);
primitive_impl!((), Null, serialize_null);

impl IDLType for String {
    fn id() -> TypeId { TypeId::of::<String>() }
    fn _ty() -> Type { Type::Text }
    fn idl_serialize<S>(&self, serializer: S) -> Result<(), S::Error> where S: Serializer {
        serializer.serialize_text(self)
    }    
}

impl<T: Sized> IDLType for Option<T> where T: IDLType {
    fn id() -> TypeId { TypeId::of::<Option<T>>() }
    fn _ty() -> Type { Type::Opt(Box::new(T::ty())) }
    fn idl_serialize<S>(&self, serializer: S) -> Result<(), S::Error> where S: Serializer {
        serializer.serialize_option(self.as_ref())
    }
}

impl<T> IDLType for Vec<T> where T: IDLType {
    fn id() -> TypeId { TypeId::of::<Vec<T>>() }        
    fn _ty() -> Type { Type::Vec(Box::new(T::ty())) }
    fn idl_serialize<S>(&self, serializer: S) -> Result<(), S::Error> where S: Serializer {
        let mut ser = serializer.serialize_vec(self.len())?;
        for e in self.iter() {
            super::Compound::serialize_field(&mut ser, &e)?;
        };
        Ok(())
    }
}

impl<T> IDLType for [T] where T: IDLType {
    fn id() -> TypeId { TypeId::of::<[T]>() }
    fn _ty() -> Type { Type::Vec(Box::new(T::ty())) }
    fn idl_serialize<S>(&self, serializer: S) -> Result<(), S::Error> where S: Serializer {
        let mut ser = serializer.serialize_vec(self.len())?;
        for e in self.iter() {
            super::Compound::serialize_field(&mut ser, &e)?;
        };
        Ok(())
    }    
}

impl<T,E> IDLType for Result<T,E> where T: IDLType, E: IDLType {
    fn id() -> TypeId { TypeId::of::<Result<T,E>>() }
    fn _ty() -> Type {
        Type::Variant(vec![
            // Make sure the field id is sorted by idl_hash
            Field{ id: "Ok".to_owned(), hash: 17724u32, ty: T::ty() },
            Field{ id: "Err".to_owned(),hash: 3456837u32, ty: E::ty() }]
        )
    }
    fn idl_serialize<S>(&self, serializer: S) -> Result<(), S::Error> where S: Serializer {
        match *self {
            Result::Ok(ref v) => {
                let mut ser = serializer.serialize_variant(0)?;
                super::Compound::serialize_field(&mut ser, v)
            },
            Result::Err(ref e) => {
                let mut ser = serializer.serialize_variant(1)?;
                super::Compound::serialize_field(&mut ser, e)
            },
        }
    }
}

impl<T> IDLType for Box<T> where T: ?Sized + IDLType {
    fn id() -> TypeId { TypeId::of::<Box<T>>() }
    fn _ty() -> Type { T::ty() }
    fn idl_serialize<S>(&self, serializer: S) -> Result<(), S::Error> where S: Serializer {
        (**self).idl_serialize(serializer)
    }    
}

impl<'a,T> IDLType for &'a T where T: 'a + ?Sized + IDLType {
    fn id() -> TypeId { TypeId::of::<&T>() } // ignore lifetime
    fn _ty() -> Type { T::ty() }
    fn idl_serialize<S>(&self, serializer: S) -> Result<(), S::Error> where S: Serializer {
        (**self).idl_serialize(serializer)
    }    
}
