#[derive(Debug, Clone, PartialEq)]
pub enum TypeBase {
    Int,
    Double,
    Char,
    Struct,
    Void,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Var,
    Fn,
    ExtFn,
    Struct,
    Param,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MemClass {
    Global,
    Arg,
    Local,
    NotApplicable, //used for symbols where memory class doesn't apply (e.g., structs, functions)
}

#[derive(Debug, Clone)]
pub struct Type {
    pub tb: TypeBase,
    pub struct_name: Option<String>, // Safely stores the name if tb == Struct
    pub elements: i32,               // <0 for non-array, 0 for size-less array, >0 for sized array
}

impl Type {
    // quick helper to create a default empty type
    pub fn new() -> Self {
        Self {
            tb: TypeBase::Void,
            struct_name: None,
            elements: -1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub mem: MemClass,
    pub type_info: Type,
    pub depth: i32, // 0 = global, 1 = function, 2+ = nested blocks
    
    pub args: Option<Vec<Symbol>>,    // For functions
    pub locals: Option<Vec<Symbol>>,  // For functions
    pub members: Option<Vec<Symbol>>, // For structs
}

impl Symbol {
    // Constructor for a new basic symbol
    pub fn new(name: String, kind: SymbolKind, depth: i32) -> Self {
        Self {
            name,
            kind,
            mem: MemClass::NotApplicable,
            type_info: Type::new(),
            depth,
            args: None,
            locals: None,
            members: None,
        }
    }
}