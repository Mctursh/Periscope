//! Anchor IDL type definitions (0.29+ format)
//!
//! These types represent the structure of an Anchor IDL JSON file.

use serde::Deserialize;

/// Root IDL structure
#[derive(Debug, Clone, Deserialize)]
pub struct Idl {
    /// Program address (base58)
    pub address: String,

    /// IDL metadata
    pub metadata: IdlMetadata,

    /// Program instructions
    pub instructions: Vec<IdlInstruction>,

    /// Account discriminators (references to types)
    #[serde(default)]
    pub accounts: Vec<IdlAccountRef>,

    /// Custom types defined by the program
    #[serde(default)]
    pub types: Vec<IdlTypeDef>,

    /// Event discriminators (references to types)
    #[serde(default)]
    pub events: Vec<IdlEventRef>,

    /// Error codes defined by the program
    #[serde(default)]
    pub errors: Vec<IdlError>,
}

/// IDL metadata
#[derive(Debug, Clone, Deserialize)]
pub struct IdlMetadata {
    /// Program name
    pub name: String,

    /// Program version
    pub version: String,

    /// IDL spec version
    pub spec: String,

    /// Program description
    #[serde(default)]
    pub description: Option<String>,
}

/// Instruction definition
#[derive(Debug, Clone, Deserialize)]
pub struct IdlInstruction {
    /// Instruction name
    pub name: String,

    /// Discriminator bytes
    #[serde(default)]
    pub discriminator: Vec<u8>,

    /// Accounts required by this instruction
    pub accounts: Vec<IdlAccountItem>,

    /// Arguments to this instruction
    pub args: Vec<IdlField>,
}

/// Account item (can be a single account or nested group)
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum IdlAccountItem {
    /// Single account
    Single(IdlAccount),
    /// Nested group of accounts
    Group(IdlAccountGroup),
}

/// Single account in an instruction
#[derive(Debug, Clone, Deserialize)]
pub struct IdlAccount {
    /// Account name
    pub name: String,

    /// Whether this account is writable
    #[serde(default)]
    pub writable: bool,

    /// Whether this account must sign
    #[serde(default)]
    pub signer: bool,

    /// Whether this account is optional
    #[serde(default)]
    pub optional: bool,

    /// Account address (for known accounts like system program)
    #[serde(default)]
    pub address: Option<String>,

    /// PDA seeds if this is a PDA
    #[serde(default)]
    pub pda: Option<IdlPda>,
}

/// Group of accounts (nested)
#[derive(Debug, Clone, Deserialize)]
pub struct IdlAccountGroup {
    /// Group name
    pub name: String,

    /// Accounts in this group
    pub accounts: Vec<IdlAccountItem>,
}

/// PDA definition
#[derive(Debug, Clone, Deserialize)]
pub struct IdlPda {
    /// PDA seeds
    pub seeds: Vec<IdlSeed>,
}

/// PDA seed
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum IdlSeed {
    /// Constant seed (literal bytes)
    Const { value: serde_json::Value },
    /// Account seed (pubkey from another account)
    Account { path: String },
    /// Argument seed (value from instruction args)
    Arg { path: String },
}

/// Field definition (for args and struct fields)
#[derive(Debug, Clone, Deserialize)]
pub struct IdlField {
    /// Field name
    pub name: String,

    /// Field type
    #[serde(rename = "type")]
    pub ty: IdlType,
}

/// Type definition (struct or enum)
#[derive(Debug, Clone, Deserialize)]
pub struct IdlTypeDef {
    /// Type name
    pub name: String,

    /// Type definition
    #[serde(rename = "type")]
    pub ty: IdlTypeDefTy,
}

/// Type definition body
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum IdlTypeDefTy {
    /// Struct type
    Struct { fields: Vec<IdlField> },
    /// Enum type
    Enum { variants: Vec<IdlEnumVariant> },
}

/// Enum variant
#[derive(Debug, Clone, Deserialize)]
pub struct IdlEnumVariant {
    /// Variant name
    pub name: String,

    /// Variant fields (if tuple or struct variant)
    #[serde(default)]
    pub fields: Option<IdlEnumFields>,
}

/// Enum variant fields - can be tuple-style (unnamed) or struct-style (named)
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum IdlEnumFields {
    /// Tuple variant: fields are just types (e.g., ["u64", "pubkey"])
    Tuple(Vec<IdlType>),
    /// Struct variant: fields have names and types (e.g., [{"name": "x", "type": "u64"}])
    Named(Vec<IdlField>),
}

/// IDL type (primitives and composites)
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum IdlType {
    /// Primitive type as string (u8, u64, bool, pubkey, etc.)
    Primitive(String),

    /// Complex type
    Complex(IdlTypeComplex),
}

/// Complex IDL types
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IdlTypeComplex {
    /// Vec<T>
    Vec(Box<IdlType>),
    /// Option<T>
    Option(Box<IdlType>),
    /// [T; N]
    Array(Box<IdlType>, usize),
    /// Reference to a defined type
    Defined { name: String },
}

/// Account reference (root-level accounts array)
/// Just a discriminator reference - actual type is in `types`
#[derive(Debug, Clone, Deserialize)]
pub struct IdlAccountRef {
    /// Account type name
    pub name: String,

    /// Account discriminator bytes
    #[serde(default)]
    pub discriminator: Vec<u8>,
}

/// Event reference (root-level events array)
/// Just a discriminator reference - actual type is in `types`
#[derive(Debug, Clone, Deserialize)]
pub struct IdlEventRef {
    /// Event type name
    pub name: String,

    /// Event discriminator bytes
    #[serde(default)]
    pub discriminator: Vec<u8>,
}

/// Error definition
#[derive(Debug, Clone, Deserialize)]
pub struct IdlError {
    /// Error code
    pub code: u32,

    /// Error name
    pub name: String,

    /// Error message
    #[serde(default)]
    pub msg: Option<String>,
}
