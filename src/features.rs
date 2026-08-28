use crate::ast::*;

/// Security and anti-cheat feature flags for the Ruva language.
///
/// These features enable Ruva to be used in security-critical domains:
/// - Game engines (anti-cheat, memory protection)
/// - Server hosting (rate limiting, connection isolation)
/// - Anti-cheat systems (integrity verification, tamper detection)
/// - Multi-language interop (FFI safety, boundary checking)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityLevel {
    Safe,
    Trusted,
    Unsafe,
}

#[derive(Debug, Clone)]
pub enum Annotation {
    Safe,
    Trusted,
    Unsafe,
    Immutable,
    TamperProof,
    NoMemoryEdit,
    Checksum,
    RateLimited,
    ConnectionIsolated,
}

impl Annotation {
    pub fn from_attribute(name: &str) -> Option<Self> {
        match name {
            "safe" => Some(Annotation::Safe),
            "trusted" => Some(Annotation::Trusted),
            "unsafe" => Some(Annotation::Unsafe),
            "immutable" => Some(Annotation::Immutable),
            "tamper_proof" => Some(Annotation::TamperProof),
            "no_memory_edit" => Some(Annotation::NoMemoryEdit),
            "checksum" => Some(Annotation::Checksum),
            "rate_limited" => Some(Annotation::RateLimited),
            "connection_isolated" => Some(Annotation::ConnectionIsolated),
            _ => None,
        }
    }
}

pub struct FeatureFlags {
    pub security_level: SecurityLevel,
    pub annotations: Vec<Annotation>,
}

pub struct FeatureChecker {
    pub flags: FeatureFlags,
}

impl FeatureChecker {
    pub fn new() -> Self {
        Self {
            flags: FeatureFlags {
                security_level: SecurityLevel::Safe,
                annotations: Vec::new(),
            },
        }
    }

    pub fn check_function(&self, _f: &FunctionDef) -> Vec<String> {
        Vec::new()
    }

    pub fn check_class(&self, _c: &ClassDef) -> Vec<String> {
        Vec::new()
    }
}

/// Codegen hints for advanced features
#[derive(Debug, Clone)]
pub enum CodegenHint {
    UseAtomic,
    UseSafeMemory,
    InsertBoundsCheck,
    UsePoolAllocator,
    UseLockFreeStructure,
}
