//! Legacy Anchor IDL format support (pre-0.29)
//!
//! This module handles parsing and converting legacy IDL formats
//! to the canonical (0.1.0 spec) format.

use serde::Deserialize;

use super::types::{
    Idl, IdlAccount, IdlAccountItem, IdlAccountRef, IdlEnumFields, IdlEnumVariant, IdlError,
    IdlEventRef, IdlField, IdlInstruction, IdlMetadata, IdlType, IdlTypeComplex, IdlTypeDef,
    IdlTypeDefTy,
};

#[derive(Debug, Clone, Deserialize)]
pub struct LegacyIdl {
    /// Program name (at root level in legacy)
    pub name: String,

    /// Program version (at root level in legacy)
    pub version: String,

    /// Legacy metadata (different structure - has address, origin, etc.)
    #[serde(default)]
    pub metadata: Option<LegacyMetadata>,

    /// Instructions
    #[serde(default)]
    pub instructions: Vec<LegacyInstruction>,

    /// Account type definitions (full structs, not refs)
    #[serde(default)]
    pub accounts: Vec<LegacyTypeDef>,

    /// Custom types
    #[serde(default)]
    pub types: Vec<LegacyTypeDef>,

    /// Events (full definitions in legacy)
    #[serde(default)]
    pub events: Vec<LegacyEvent>,

    /// Errors
    #[serde(default)]
    pub errors: Vec<IdlError>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LegacyMetadata {
    /// Program address (in legacy metadata)
    #[serde(default)]
    pub address: Option<String>,

    #[serde(default)]
    pub origin: Option<String>,

    #[serde(default)]
    #[serde(rename = "binaryVersion")]
    pub binary_version: Option<String>,

    #[serde(default)]
    #[serde(rename = "libVersion")]
    pub lib_version: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LegacyInstruction {
    pub name: String,

    #[serde(default)]
    pub docs: Vec<String>,

    #[serde(default)]
    pub accounts: Vec<LegacyInstructionAccount>,

    #[serde(default)]
    pub args: Vec<LegacyField>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LegacyInstructionAccount {
    pub name: String,

    #[serde(default)]
    #[serde(rename = "isMut")]
    pub is_mut: bool,

    #[serde(default)]
    #[serde(rename = "isSigner")]
    pub is_signer: bool,

    #[serde(default)]
    #[serde(rename = "isOptional")]
    pub is_optional: bool,

    #[serde(default)]
    pub docs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LegacyTypeDef {
    pub name: String,

    #[serde(default)]
    pub docs: Vec<String>,

    #[serde(rename = "type")]
    pub ty: LegacyTypeDefTy,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum LegacyTypeDefTy {
    Struct {
        #[serde(default)]
        fields: Vec<LegacyField>,
    },
    Enum {
        #[serde(default)]
        variants: Vec<LegacyEnumVariant>,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct LegacyEnumVariant {
    pub name: String,

    #[serde(default)]
    pub fields: Option<Vec<LegacyField>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LegacyField {
    pub name: String,

    #[serde(default)]
    pub docs: Vec<String>,

    #[serde(rename = "type")]
    pub ty: LegacyType,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum LegacyType {
    /// Primitive type as string (u8, u64, bool, publicKey, etc.)
    Primitive(String),

    /// Complex type as object
    Complex(LegacyTypeComplex),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LegacyTypeComplex {
    /// Vec<T>
    Vec(Box<LegacyType>),

    /// Option<T>
    Option(Box<LegacyType>),

    /// [T; N] - array with size
    Array(Box<LegacyType>, usize),

    /// Reference to a defined type - legacy uses string directly
    Defined(String),
}

#[derive(Debug, Clone, Deserialize)]
pub struct LegacyEvent {
    pub name: String,

    #[serde(default)]
    pub fields: Vec<LegacyField>,
}

impl From<LegacyIdl> for Idl {
    fn from(legacy: LegacyIdl) -> Self {
        let address = legacy
            .metadata
            .as_ref()
            .and_then(|m| m.address.clone())
            .unwrap_or_default();

        Idl {
            address,
            metadata: IdlMetadata {
                name: legacy.name,
                version: legacy.version,
                spec: "legacy".to_string(),
                description: None,
            },
            instructions: legacy.instructions.into_iter().map(Into::into).collect(),
            accounts: legacy
                .accounts
                .iter()
                .map(|a| IdlAccountRef {
                    name: a.name.clone(),
                    discriminator: vec![],
                })
                .collect(),
            types: legacy
                .accounts
                .into_iter()
                .chain(legacy.types)
                .map(Into::into)
                .collect(),
            events: legacy
                .events
                .iter()
                .map(|e| IdlEventRef {
                    name: e.name.clone(),
                    discriminator: vec![],
                })
                .collect(),
            errors: legacy.errors,
        }
    }
}

impl From<LegacyInstruction> for IdlInstruction {
    fn from(legacy: LegacyInstruction) -> Self {
        IdlInstruction {
            name: legacy.name,
            discriminator: vec![],
            accounts: legacy
                .accounts
                .into_iter()
                .map(|a| IdlAccountItem::Single(a.into()))
                .collect(),
            args: legacy.args.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<LegacyInstructionAccount> for IdlAccount {
    fn from(legacy: LegacyInstructionAccount) -> Self {
        IdlAccount {
            name: legacy.name,
            writable: legacy.is_mut,
            signer: legacy.is_signer,
            optional: legacy.is_optional,
            address: None,
            pda: None,
        }
    }
}

impl From<LegacyTypeDef> for IdlTypeDef {
    fn from(legacy: LegacyTypeDef) -> Self {
        IdlTypeDef {
            name: legacy.name,
            ty: legacy.ty.into(),
        }
    }
}

impl From<LegacyTypeDefTy> for IdlTypeDefTy {
    fn from(legacy: LegacyTypeDefTy) -> Self {
        match legacy {
            LegacyTypeDefTy::Struct { fields } => IdlTypeDefTy::Struct {
                fields: fields.into_iter().map(Into::into).collect(),
            },
            LegacyTypeDefTy::Enum { variants } => IdlTypeDefTy::Enum {
                variants: variants.into_iter().map(Into::into).collect(),
            },
        }
    }
}

impl From<LegacyEnumVariant> for IdlEnumVariant {
    fn from(legacy: LegacyEnumVariant) -> Self {
        IdlEnumVariant {
            name: legacy.name,
            fields: legacy
                .fields
                .map(|fields| IdlEnumFields::Named(fields.into_iter().map(Into::into).collect())),
        }
    }
}

impl From<LegacyField> for IdlField {
    fn from(legacy: LegacyField) -> Self {
        IdlField {
            name: legacy.name,
            ty: legacy.ty.into(),
        }
    }
}

impl From<LegacyType> for IdlType {
    fn from(legacy: LegacyType) -> Self {
        match legacy {
            LegacyType::Primitive(s) => {
                if s == "publicKey" {
                    IdlType::Primitive("pubkey".to_string())
                } else {
                    IdlType::Primitive(s)
                }
            }
            LegacyType::Complex(complex) => IdlType::Complex(complex.into()),
        }
    }
}

impl From<LegacyTypeComplex> for IdlTypeComplex {
    fn from(legacy: LegacyTypeComplex) -> Self {
        match legacy {
            LegacyTypeComplex::Vec(inner) => IdlTypeComplex::Vec(Box::new((*inner).into())),
            LegacyTypeComplex::Option(inner) => IdlTypeComplex::Option(Box::new((*inner).into())),
            LegacyTypeComplex::Array(inner, size) => {
                IdlTypeComplex::Array(Box::new((*inner).into()), size)
            }
            LegacyTypeComplex::Defined(name) => IdlTypeComplex::Defined { name },
        }
    }
}
