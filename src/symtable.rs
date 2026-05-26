use crate::symbol::Symbol;

pub struct SymTable {
    pub symbols: Vec<Symbol>,
    pub current_depth: i32,
    
    pub owner_idx: Option<usize>, 
}

impl SymTable {
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
            current_depth: 0,
            owner_idx: None,
        }
    }

    // Opens a new scope 
    pub fn push_domain(&mut self) {
        self.current_depth += 1;
    }

    // Closes a scope and destroys all variables created inside it 
    pub fn drop_domain(&mut self) {
        // Rust's `retain` keeps only the elements that match the condition.
        // We throw away any symbol whose depth matches the current (closing) depth.
        self.symbols.retain(|s| s.depth < self.current_depth);
        self.current_depth -= 1;
    }

    // Searches for a symbol by name. 
    // We search backwards (.rev()) so that inner-scope variables 
    // shadow outer-scope variables with the same name.
    pub fn find_symbol(&self, name: &str) -> Option<&Symbol> {
        self.symbols.iter().rev().find(|s| s.name == name)
    }

    // Same as above, but allows us to modify the symbol (like adding args to a function)
    pub fn find_symbol_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        self.symbols.iter_mut().rev().find(|s| s.name == name)
    }

    // Adds a new symbol to the table
    pub fn add_symbol(&mut self, mut sym: Symbol) -> Result<(), String> {
        // Rule: You cannot define two variables with the SAME name at the SAME depth.
        // (However, an inner block *can* redefine a global variable) 
        if self.symbols.iter().any(|s| s.name == sym.name && s.depth == self.current_depth) {
            return Err(format!("Symbol redefinition: {}", sym.name));
        }
        
        sym.depth = self.current_depth;
        self.symbols.push(sym);
        Ok(())
    }

    // Helper to set the 'owner' to the most recently added struct/function
    pub fn set_owner_to_last(&mut self) {
        if !self.symbols.is_empty() {
            self.owner_idx = Some(self.symbols.len() - 1);
        }
    }

    // Helper to clear the owner when we finish parsing a struct/function
    pub fn clear_owner(&mut self) {
        self.owner_idx = None;
    }
}