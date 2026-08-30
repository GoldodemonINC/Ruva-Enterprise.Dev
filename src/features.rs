use crate::ast::*;









#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SecurityLevel {
    Safe,
    Trusted,
    Unsafe,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
pub struct FeatureFlags {
    pub security_level: SecurityLevel,
    pub annotations: Vec<Annotation>,
}

#[allow(dead_code)]
pub struct FeatureChecker {
    pub flags: FeatureFlags,
}

#[allow(dead_code)]
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


#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum CodegenHint {
    UseAtomic,
    UseSafeMemory,
    InsertBoundsCheck,
    UsePoolAllocator,
    UseLockFreeStructure,
}

