pub mod types_api;
pub mod types_api2;

pub mod types_all_data;
pub mod types_api2_mol2;
pub mod types_array;
pub mod types_moleculec_check;
pub mod types_option;
pub mod types_struct;
pub mod types_table;
pub mod types_vec;

use molecule::lazy_reader::Cursor;
use molecule::prelude::{Builder, Entity};
use rand::{rngs::ThreadRng, thread_rng, Rng};
use std::fmt::Debug;

pub use types_api2_mol2::*;
pub use types_array::*;
pub use types_moleculec_check::*;
pub use types_option::*;
pub use types_struct::*;
pub use types_table::*;
pub use types_vec::*;

pub trait BaseTypes {
    fn new_rng(rng: &mut ThreadRng, config: &TypesConfig) -> Self;
}
impl BaseTypes for u8 {
    fn new_rng(rng: &mut ThreadRng, _config: &TypesConfig) -> Self {
        rng.gen()
    }
}

pub enum OptionFillType {
    FillRand,
    FillSome,
    FillNone,
}
pub struct TypesConfig {
    pub option_fill: OptionFillType,
    pub large_vec: bool,
    pub min_size: bool,
}

impl Default for TypesConfig {
    fn default() -> Self {
        Self {
            option_fill: OptionFillType::FillRand,
            large_vec: false,
            min_size: false,
        }
    }
}

#[derive(Debug)]
pub enum TypesCheckErr {
    Lenght(String),
    Data(String),
    Opt(String),
    Mol2Err(String),
}
pub type ResCheckErr = Result<(), TypesCheckErr>;

impl TypesCheckErr {
    pub fn to(&self, des: String) -> Self {
        match self {
            Self::Lenght(_) => Self::Lenght(des),
            Self::Data(_) => Self::Data(des),
            Self::Opt(_) => Self::Opt(des),
            Self::Mol2Err(_) => Self::Mol2Err(des),
        }
    }
    pub fn as_str(&self) -> &str {
        match self {
            Self::Lenght(v) => v,
            Self::Data(v) => v,
            Self::Opt(v) => v,
            Self::Mol2Err(v) => v,
        }
    }
    pub fn check_length(l1: usize, l2: usize) -> Result<(), Self> {
        if l1 == l2 {
            Ok(())
        } else {
            Err(Self::Lenght(format!("different length : {} {}", l1, l2)))
        }
    }

    pub fn check_1_data<T: Eq + Debug>(d1: &T, d2: &T) -> Result<(), Self> {
        if d1 == d2 {
            Ok(())
        } else {
            Err(Self::Data(format!(
                "different data: {:02x?}  {:02x?}",
                d1, d2
            )))
        }
    }

    pub fn check_data<T1: Eq + Debug, T: Mol2Vec<RetType = T1>>(d1: &T, d2: &[T1]) -> ResCheckErr {
        TypesCheckErr::check_length(d1.mol_len()?, d2.len())?;

        for i in 0..d1.mol_len()? {
            TypesCheckErr::check_1_data(&d1.mol_get(i)?, &d2[i])?;
        }

        Ok(())
    }

    pub fn check_mol_data<T: Eq + Debug, T1: Mol2Vec<RetType = T>, T2: Mol2Vec<RetType = T>>(
        d1: &T1,
        d2: &T2,
    ) -> ResCheckErr {
        TypesCheckErr::check_length(d1.mol_len()?, d2.mol_len()?)?;
        for i in 0..d1.mol_len()? {
            TypesCheckErr::check_1_data(&d1.mol_get(i)?, &d2.mol_get(i)?)?;
        }
        Ok(())
    }

    pub fn check_option<T1, T2>(d1: &Option<T1>, d2: &Option<T2>) -> ResCheckErr {
        if d1.is_some() != d2.is_some() {
            return Err(TypesCheckErr::Opt(format!(
                "different option: {:?}  {:?}",
                d1.is_some(),
                d2.is_some()
            )));
        }
        Ok(())
    }
}
impl From<molecule::lazy_reader::Error> for TypesCheckErr {
    fn from(value: molecule::lazy_reader::Error) -> Self {
        use molecule::lazy_reader::Error::*;
        match value {
            Common(v) => Self::Mol2Err(format!("Common({})", v)),
            TotalSize(v) => Self::Mol2Err(format!("TotalSize({})", v)),
            Header(v) => Self::Mol2Err(format!("Header({})", v)),
            Offset(v) => Self::Mol2Err(format!("Offset({})", v)),
            UnknownItem(v) => Self::Mol2Err(format!("UnknownItem({})", v)),
            OutOfBound(v) => Self::Mol2Err(format!("OutOfBound({})", v)),
            FieldCount(v) => Self::Mol2Err(format!("FieldCount({})", v)),
            Data(v) => Self::Mol2Err(format!("Data({})", v)),
            Overflow(v) => Self::Mol2Err(format!("Overflow({})", v)),
            Read(v) => Self::Mol2Err(format!("Read({})", v)),
            Verify(v) => Self::Mol2Err(format!("Verify({})", v)),
            Unknow(v) => Self::Mol2Err(format!("Unknow({})", v)),
        }
    }
}
impl From<std::convert::Infallible> for TypesCheckErr {
    fn from(value: std::convert::Infallible) -> Self {
        Self::Mol2Err(format!("conver failed: {:?}", value))
    }
}

pub enum TypesUnionA {
    Byte(TypesArray<u8, 1>),
    Word(TypesArrayWord),
    StructA(TypesStructA),
    Bytes(TypesVec<u8>),
    Words(TypesVec<TypesArrayWord>),
    Table0(TypesTable0),
    Table6(TypesTable6),
    Table6Opt(TypesOption<TypesTable6>),
}
impl BaseTypes for TypesUnionA {
    fn new_rng(rng: &mut ThreadRng, config: &TypesConfig) -> Self {
        let v = if config.min_size {
            0 // Self::Byte
        } else {
            rng.gen_range(0..8)
        };
        match v {
            0 => Self::Byte(TypesArray::new_rng(rng, config)),
            1 => Self::Word(TypesArrayWord::new_rng(rng, config)),
            2 => Self::StructA(TypesStructA::new_rng(rng, config)),
            3 => Self::Bytes(TypesVec::new_rng(rng, config)),
            4 => Self::Words(TypesVec::new_rng(rng, config)),
            5 => Self::Table0(TypesTable0::new_rng(rng, config)),
            6 => Self::Table6(TypesTable6::new_rng(rng, config)),
            7 => Self::Table6Opt(TypesOption::new_rng(rng, config)),

            _ => panic!("unknow error"),
        }
    }
}
impl Default for TypesUnionA {
    fn default() -> Self {
        Self::new_rng(&mut thread_rng(), &TypesConfig::default())
    }
}
impl TypesUnionA {
    pub fn to_mol(&self) -> types_api::UnionA {
        let t = match self {
            Self::Byte(v) => types_api::UnionAUnion::Byte(v.to_mol()),
            Self::Word(v) => types_api::UnionAUnion::Word(v.to_mol2()),
            Self::StructA(v) => types_api::UnionAUnion::StructA(v.to_mol()),
            Self::Bytes(v) => types_api::UnionAUnion::Bytes(v.to_mol()),
            Self::Words(v) => types_api::UnionAUnion::Words(v.to_mol()),
            Self::Table0(v) => types_api::UnionAUnion::Table0(v.to_mol()),
            Self::Table6(v) => types_api::UnionAUnion::Table6(v.to_mol()),
            Self::Table6Opt(v) => types_api::UnionAUnion::Table6Opt(v.to_mol()),
        };
        types_api::UnionA::new_builder().set(t).build()
    }

    pub fn check(&self, d: &types_api2::UnionA) -> ResCheckErr {
        // let item_id = d.item_id();

        match self {
            Self::Byte(v) => match d {
                types_api2::UnionA::Byte(v2) => v.check(v2),
                _ => Err(TypesCheckErr::Data(format!("check union type is failed"))),
            },
            Self::Word(v) => match d {
                types_api2::UnionA::Word(v2) => v.check2(v2),
                _ => Err(TypesCheckErr::Data(format!("check union type is failed"))),
            },
            Self::StructA(v) => match d {
                types_api2::UnionA::StructA(v2) => v.check(v2),
                _ => Err(TypesCheckErr::Data(format!("check union type is failed"))),
            },
            Self::Bytes(v) => match d {
                types_api2::UnionA::Bytes(v2) => {
                    v.check(&v2.clone().try_into()?)
                }
                _ => Err(TypesCheckErr::Data(format!("check union type is failed"))),
            },
            Self::Words(v) => match d {
                types_api2::UnionA::Words(v2) => v.check(v2),
                _ => Err(TypesCheckErr::Data(format!("check union type is failed"))),
            },
            Self::Table0(v) => match d {
                types_api2::UnionA::Table0(v2) => v.check(v2),
                _ => Err(TypesCheckErr::Data(format!("check union type is failed"))),
            },
            Self::Table6(v) => match d {
                types_api2::UnionA::Table6(v2) => v.check(v2),
                _ => Err(TypesCheckErr::Data(format!("check union type is failed"))),
            },
            Self::Table6Opt(v) => match d {
                types_api2::UnionA::Table6Opt(v2) => v.check(v2),
                _ => Err(TypesCheckErr::Data(format!("check union type is failed"))),
            },
        }
    }
}

pub enum TypesUnionB {
    Byte(TypesArray<u8, 1>), // 2
    Word(TypesArrayWord),    //4
}
impl BaseTypes for TypesUnionB {
    fn new_rng(rng: &mut ThreadRng, config: &TypesConfig) -> Self {
        let v = if config.min_size {
            0 // Self::Byte
        } else {
            rng.gen_range(0..1)
        };
        match v {
            0 => Self::Byte(TypesArray::new_rng(rng, config)),
            1 => Self::Word(TypesArrayWord::new_rng(rng, config)),

            _ => panic!("unknow error"),
        }
    }
}
impl Default for TypesUnionB {
    fn default() -> Self {
        Self::new_rng(&mut thread_rng(), &TypesConfig::default())
    }
}
impl TypesUnionB {
    pub fn to_mol(&self) -> types_api::UnionB {
        let t = match self {
            Self::Byte(v) => types_api::UnionBUnion::Byte(v.to_mol()),
            Self::Word(v) => types_api::UnionBUnion::Word(v.to_mol2()),
        };
        types_api::UnionB::new_builder().set(t).build()
    }

    pub fn check(&self, d: &types_api2::UnionB) -> ResCheckErr {
        // let item_id = d.item_id();

        match self {
            Self::Byte(v) => match d {
                types_api2::UnionB::Byte(v2) => v.check(v2),
                _ => Err(TypesCheckErr::Data(format!("check union type is failed"))),
            },
            Self::Word(v) => match d {
                types_api2::UnionB::Word(v2) => v.check2(v2),
                _ => Err(TypesCheckErr::Data(format!("check union type is failed"))),
            },
        }
    }
}

pub enum TypesUnionD {
    Word(TypesArrayWord),    //2
    Byte(TypesArray<u8, 1>), // 4
}
impl BaseTypes for TypesUnionD {
    fn new_rng(rng: &mut ThreadRng, config: &TypesConfig) -> Self {
        let v = if config.min_size {
            0 // Self::Byte
        } else {
            rng.gen_range(0..1)
        };
        match v {
            0 => Self::Word(TypesArrayWord::new_rng(rng, config)),
            1 => Self::Byte(TypesArray::new_rng(rng, config)),

            _ => panic!("unknow error"),
        }
    }
}
impl Default for TypesUnionD {
    fn default() -> Self {
        Self::new_rng(&mut thread_rng(), &TypesConfig::default())
    }
}
impl TypesUnionD {
    pub fn to_mol(&self) -> types_api::UnionD {
        let t = match self {
            Self::Word(v) => types_api::UnionDUnion::Word(v.to_mol2()),
            Self::Byte(v) => types_api::UnionDUnion::Byte(v.to_mol()),
        };
        types_api::UnionD::new_builder().set(t).build()
    }

    pub fn check(&self, d: &types_api2::UnionD) -> ResCheckErr {
        // let item_id = d.item_id();

        match self {
            Self::Word(v) => match d {
                types_api2::UnionD::Word(v2) => v.check2(v2),
                _ => Err(TypesCheckErr::Data(format!("check union type is failed"))),
            },
            Self::Byte(v) => match d {
                types_api2::UnionD::Byte(v2) => v.check(v2),
                _ => Err(TypesCheckErr::Data(format!("check union type is failed"))),
            },
        }
    }
}
