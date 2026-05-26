use crate::token::{Token, TokenCode};
use crate::symtable::SymTable; 
use crate::symbol::{Type, TypeBase, Symbol, SymbolKind, MemClass}; 

pub struct Parser{
    tokens: Vec<Token>,
    pos: usize,
    sym_table: SymTable,
}

impl Parser{
    pub fn new(tokens: Vec<Token>) -> Self {
        Self{tokens, 
            pos: 0,
            sym_table: SymTable::new(),
        }
    }

    //get current token
    fn crt_tk(&self) -> Option<&Token>{
        self.tokens.get(self.pos)
    }

    //consume token
    fn consume(&mut self, expected: TokenCode) -> bool{
        if let Some(tk) = self.crt_tk() {
            if tk.code == expected{
                self.pos += 1;
                return true;
            }
        }
        false
    }
    
    fn err(&self, msg: &str) -> ! {
        let line = self.crt_tk().map_or(0, |tk| tk.line);
        eprintln!("Syntax Error at line {}: {}", line, msg);
        std::process::exit(-1);
    }

    //special consume cases for checking IDs and constants

    fn consume_id(&mut self) -> bool{
        if let Some(Token{code: TokenCode::ID(_), .. }) = self.crt_tk() {
            self.pos += 1;
            return true;
        }
        false
    }

    // Extracts the actual string from an ID token
    fn consume_id_name(&mut self) -> Option<String> {
        if let Some(Token { code: TokenCode::ID(name), .. }) = self.crt_tk() {
            let id_name = name.clone();
            self.pos += 1;
            Some(id_name)
        } else {
            None
        }
    }

    fn consume_ct_int(&mut self) -> bool {
        if let Some(Token { code: TokenCode::CtInt(_), .. }) = self.crt_tk() {
            self.pos += 1;
            return true;
        }
        false
    }

    fn consume_ct_real(&mut self) -> bool {
        if let Some(Token { code: TokenCode::CtReal(_), .. }) = self.crt_tk() {
            self.pos += 1;
            return true;
        }
        false
    }

    fn consume_ct_char(&mut self) -> bool {
        if let Some(Token { code: TokenCode::CtChar(_), .. }) = self.crt_tk() {
            self.pos += 1;
            return true;
        }
        false
    }

    fn consume_ct_string(&mut self) -> bool {
        if let Some(Token { code: TokenCode::CtString(_), .. }) = self.crt_tk() {
            self.pos += 1;
            return true;
        }
        false
    }

    //Lexical rules

    //unit: (struct_def | fn_def | var_def)* END

    pub fn parse(&mut self){
        loop{
            if self.struct_def() { continue; }
            if self.fn_def() { continue; }
            if self.var_def() { continue; }
            break;
        }
        if !self.consume(TokenCode::END){
            self.err("Expected EOF, struct_def, fn_def or var_def");
        }
    }

    // struct_def: STRUCT ID LACC var_def* RACC SEMICOLON
    fn struct_def(&mut self) -> bool {
        let start_pos = self.pos;
        if self.consume(TokenCode::STRUCT) {
            
            //Use consume_id_name to get the struct's name
            if let Some(struct_name) = self.consume_id_name() {
                if self.consume(TokenCode::LACC) {
                    
                    //Add Struct to Symbol Table & Open Scope
                    let mut struct_sym = Symbol::new(struct_name.clone(), SymbolKind::Struct, self.sym_table.current_depth);
                    struct_sym.type_info.tb = TypeBase::Struct;
                    
                    if let Err(e) = self.sym_table.add_symbol(struct_sym) {
                        self.err(&e);
                    }

                    self.sym_table.push_domain(); // Open scope for struct members

                    while self.var_def() {} // Parse all internal variables (x, y)

                    if self.consume(TokenCode::RACC) {
                        
                        // Close Struct Scope 
                        self.sym_table.drop_domain();

                        if self.consume(TokenCode::SEMICOLON) {
                            return true;
                        } else { self.err("Missing ';' after struct definition"); }
                    } else { self.err("Missing '}' in struct definition"); }
                }
            } else { self.err("Missing ID after 'struct'"); }
        }
        self.pos = start_pos;
        false
    }

    // var_def: type_base ID array_decl? (COMMA ID array_decl?)* SEMICOLON
    fn var_def(&mut self) -> bool {
        let start_pos = self.pos;
        
        if let Some(base_type) = self.type_base() {
            if let Some(var_name) = self.consume_id_name() {
                
                let mut current_type = base_type.clone();
                self.array_decl(&mut current_type); // Modifies current_type if it's an array
                
                //Add the first variable to the Symbol Table
                let mut sym = Symbol::new(var_name, SymbolKind::Var, self.sym_table.current_depth);
                sym.type_info = current_type;
                if let Err(e) = self.sym_table.add_symbol(sym) {
                    self.err(&e);
                }
                
                // Loop to handle comma-separated variables
                while self.consume(TokenCode::COMMA) {
                    if let Some(next_var_name) = self.consume_id_name() {
                        
                        let mut next_type = base_type.clone();
                        self.array_decl(&mut next_type);
                        
                        //Add the next variable 
                        let mut next_sym = Symbol::new(next_var_name, SymbolKind::Var, self.sym_table.current_depth);
                        next_sym.type_info = next_type;
                        if let Err(e) = self.sym_table.add_symbol(next_sym) {
                            self.err(&e);
                        }
                    } else {
                        self.err("Missing variable name after ','");
                    }
                }

                if self.consume(TokenCode::SEMICOLON) {
                    return true;
                } else {
                    self.err("Missing ';' at the end of variable declaration");
                }
            }
        }
        
        self.pos = start_pos;
        false
    }

   // type_base [out Type *t]: INT | DOUBLE | CHAR | STRUCT ID
    fn type_base(&mut self) -> Option<Type> {
        let start_pos = self.pos;
        let mut t = Type::new();

        if self.consume(TokenCode::INT) {
            t.tb = TypeBase::Int;
            return Some(t);
        }
        if self.consume(TokenCode::DOUBLE) {
            t.tb = TypeBase::Double;
            return Some(t);
        }
        if self.consume(TokenCode::CHAR) {
            t.tb = TypeBase ::Char;
            return Some(t);
        }
        if self.consume(TokenCode::STRUCT) {
            if let Some(name) = self.consume_id_name() {
                
                //Check if struct exists
                if self.sym_table.find_symbol(&name).is_none() {
                    self.err(&format!("Undefined struct: {}", name));
                }
                t.tb = TypeBase::Struct;
                t.struct_name = Some(name);
                return Some(t);
            } else {
                self.err("Missing ID after 'struct'");
            }
        }

        self.pos = start_pos;
        None
    }
    
    // array_decl [inout Type *t]: LBRACKET expr? RBRACKET
    fn array_decl(&mut self, t: &mut Type) -> bool {
        let start_pos = self.pos;
        if self.consume(TokenCode::LBRACKET) {
            
            self.expr(); // We consume the math inside the brackets
            t.elements = 0; // Mark this type as an array

            if self.consume(TokenCode::RBRACKET) {
                return true;
            } else { self.err("Missing ']' in array declaration"); }
        }
        self.pos = start_pos;
        false
    }

    // fnDef: (type_base | VOID) ID LPAR (fnParam (COMMA fnParam)*)? RPAR stm_compound
    fn fn_def(&mut self) -> bool {
        let start_pos = self.pos;
        let mut has_type = false;
        let mut return_type = Type::new();
        
        if let Some(base_type) = self.type_base() {
            has_type = true;
            return_type = base_type;
        } else if self.consume(TokenCode::VOID) {
            has_type = true;
            return_type.tb = TypeBase::Void;
        }

        if has_type {
            if let Some(fn_name) = self.consume_id_name() {
                if self.consume(TokenCode::LPAR) {
                    
                    //Add function to global scope & open param scope
                    let mut fn_sym = Symbol::new(fn_name.clone(), SymbolKind::Fn, self.sym_table.current_depth);
                    fn_sym.type_info = return_type;
                    if let Err(e) = self.sym_table.add_symbol(fn_sym) {
                        self.err(&e);
                    }
                    
                    self.sym_table.push_domain(); // Open scope for arguments
                    

                    if self.fn_param() {
                        while self.consume(TokenCode::COMMA) {
                            if !self.fn_param() { self.err("Expected function parameter after ','"); }
                        }
                    }
                    if self.consume(TokenCode::RPAR) {
                        
                        // Pass 'false' because the domain was already pushed for the arguments!
                        if self.stm_compound(false) { 
                            
                            //Close param/local scope
                            self.sym_table.drop_domain();
                            
                            return true;
                        } else { self.err("Expected compound statement '{...}' for function body"); }
                    } else { self.err("Missing ')' in function definition"); }
                }
            }
        }
        self.pos = start_pos;
        false
    }

    // fnParam: type_base ID array_decl?
    fn fn_param(&mut self) -> bool {
        let start_pos = self.pos;
        if let Some(base_type) = self.type_base() {
            if let Some(var_name) = self.consume_id_name() {
                
                let mut current_type = base_type.clone();
                self.array_decl(&mut current_type); // optional array modification
                
                //Add parameter to the current scope
                let mut sym = Symbol::new(var_name, SymbolKind::Param, self.sym_table.current_depth);
                sym.type_info = current_type;
                if let Err(e) = self.sym_table.add_symbol(sym) {
                    self.err(&e);
                }
                
                return true;
            } else { self.err("Missing ID in function parameter"); }
        }
        self.pos = start_pos;
        false
    }

    // stm: stm_compound
    //     | IF LPAR expr RPAR stm (ELSE stm)?
    //     | WHILE LPAR expr RPAR stm
    //     | FOR LPAR expr? SEMICOLON expr? SEMICOLON expr? RPAR stm
    //     | BREAK SEMICOLON
    //     | RETURN expr? SEMICOLON
    //     | expr? SEMICOLON

    fn stm(&mut self) -> bool{
        let start_pos = self.pos;
        
        //stm_compound
        if self.stm_compound(true) {return true; }

        // IF LPAR expr RPAR stm (ELSE stm)?
        if self.consume(TokenCode::IF) {
            if !self.consume(TokenCode::LPAR) { self.err("Missing '(' after 'if'"); }
            if !self.expr() { self.err("Invalid expression in 'if' condition"); }
            if !self.consume(TokenCode::RPAR) { self.err("Missing ')' after 'if' condition"); }
            if !self.stm() { self.err("Missing statement for 'if' branch"); }
            if self.consume(TokenCode::ELSE) {
                if !self.stm() { self.err("Missing statement for 'else' branch"); }
            }
            return true;
        }

        // WHILE LPAR expr RPAR stm
        if self.consume(TokenCode::WHILE) {
            if !self.consume(TokenCode::LPAR) { self.err("Missing '(' after 'while'"); }
            if !self.expr() { self.err("Invalid expression in 'while' condition"); }
            if !self.consume(TokenCode::RPAR) { self.err("Missing ')' after 'while' condition"); }
            if !self.stm() { self.err("Missing statement for 'while' loop"); }
            return true;
        }

        // FOR LPAR expr? SEMICOLON expr? SEMICOLON expr? RPAR stm
        if self.consume(TokenCode::FOR) {
            if !self.consume(TokenCode::LPAR) { self.err("Missing '(' after 'for'"); }
            self.expr(); // optional
            if !self.consume(TokenCode::SEMICOLON) { self.err("Missing first ';' in 'for' loop"); }
            self.expr(); // optional
            if !self.consume(TokenCode::SEMICOLON) { self.err("Missing second ';' in 'for' loop"); }
            self.expr(); // optional
            if !self.consume(TokenCode::RPAR) { self.err("Missing ')' in 'for' loop"); }
            if !self.stm() { self.err("Missing statement for 'for' loop body"); }
            return true;
        }

        // BREAK SEMICOLON
        if self.consume(TokenCode::BREAK) {
            if self.consume(TokenCode::SEMICOLON) { return true; }
            self.err("Missing ';' after 'break'");
        }

        // RETURN expr? SEMICOLON
        if self.consume(TokenCode::RETURN) {
            self.expr(); // optional
            if self.consume(TokenCode::SEMICOLON) { return true; }
            self.err("Missing ';' after 'return'");
        }

        // expr? SEMICOLON
        if self.expr() {
            if self.consume(TokenCode::SEMICOLON) { return true; }
            self.err("Missing ';' after expression statement");
        } else if self.consume(TokenCode::SEMICOLON) {
            return true; // empty statement
        }

        self.pos = start_pos;
        false
    }

    // stm_compound [in bool newDomain]: LACC (var_def | stm)* RACC
    fn stm_compound(&mut self, new_domain: bool) -> bool {
        let start_pos = self.pos;
        if self.consume(TokenCode::LACC) {
            
            // Open scope 
            if new_domain {
                self.sym_table.push_domain();
            }

            loop {
                if self.var_def() { continue; }
                if self.stm() { continue; }
                break;
            }
            if self.consume(TokenCode::RACC) {
                
                // Close scope
                if new_domain {
                    self.sym_table.drop_domain();
                }
                
                return true;
            } else { self.err("Missing '}' to close compound statement"); }
        }
        self.pos = start_pos;
        false
    }
    //Expressions

    //expr: expr_assign

    fn expr(&mut self) -> bool{
        self.expr_assign()
    }

    //expr_assign: _u ASSIGN expr_assign | expr_or

    fn expr_assign(&mut self) -> bool{
        let start_pos = self.pos;

        if self._u() {
            if self.consume(TokenCode::ASSIGN){
                if self.expr_assign(){
                    return true;
                }
                self.err("Missing expression after '='");
            }
        }
        self.pos = start_pos; // Backtrack and try expr_or

        if self.expr_or() {
            return true;
        }
        
        self.pos = start_pos;
        false
    }

    //expr_or: expr_and (OR expr_and)*

    fn expr_or(&mut self) -> bool{
        let start_pos = self.pos;

        if self.expr_and(){
            loop{
                if self.consume(TokenCode::OR){
                    if self.expr_and(){
                        continue;
                    }
                    self.err("Expected expression after '||'");
                }
                break;
            }
            return true;
        }
        self.pos = start_pos;
        false  
    }

    //expr_and: expr_eq (AND expr_eq)*

    fn expr_and(&mut self) -> bool{
        let start_pos = self.pos;

        if self.expr_eq(){
            loop{
                if self.consume(TokenCode::AND){
                    if self.expr_eq(){
                        continue;
                    }
                    self.err("Expected expression after '&&'");
                }
                break;
            }
            return true;
        }
        self.pos = start_pos;
        false  
    }

    //expr_eq: expr_rel ((EQUAL | NOTEQ) expr_rel)*

    fn expr_eq(&mut self) -> bool{
        let start_pos = self.pos;

        if self.expr_rel(){
            loop{
                if self.consume(TokenCode::EQUAL) || self.consume(TokenCode::NOTEQ){
                    if self.expr_rel(){
                        continue;
                    }
                    self.err("Missing expression after '==' or '!='");
                }
                break;
            }
            return true;
        }
        self.pos = start_pos;
        false  
    }

    //expr_rel: expr_add ((LESS | LESSEQ | GREATER | GREATEREQ) expr_add)*

    fn expr_rel(&mut self) -> bool {
        let start_pos = self.pos;
        if self.expr_add() {
            loop {
                if self.consume(TokenCode::LESS) || self.consume(TokenCode::LESSEQ) ||
                self.consume(TokenCode::GREATER) || self.consume(TokenCode::GREATEREQ) {
                    if self.expr_add() { continue; }
                    self.err("Missing expression after relational operator");
                }
                break;
            }
            return true;
        }
        self.pos = start_pos;
        false
    }

    //expr_add: expr_mul ((ADD | SUB) expr_mul)*

    fn expr_add(&mut self) -> bool{
        let start_pos = self.pos;

        if self.expr_mul(){
            loop{
                if self.consume(TokenCode::ADD) || self.consume(TokenCode::SUB){
                    if self.expr_mul(){
                        continue;
                    }
                    self.err("Expected expression after '+' or '-'");
                }
                break;

            }
            return true;
        }
        self.pos = start_pos;
        false
    }

    //expr_mul: expr_cast ((MUL | DIV) expr_cast)*

    fn expr_mul(&mut self) -> bool{
        let start_pos = self.pos;

        if self.expr_cast(){
            loop{
                if self.consume(TokenCode::MUL) || self.consume(TokenCode::DIV){
                    if self.expr_cast(){
                        continue;
                    }
                    self.err("Expected expression after '*' or '/'");
                }
                break;

            }
            return true;
        }
        self.pos = start_pos;
        false
    }

    // expr_cast: LPAR type_base array_decl? RPAR expr_cast | expr_unary
    fn expr_cast(&mut self) -> bool {
        let start_pos = self.pos;
        if self.consume(TokenCode::LPAR) {
            
            // Capture the type and pass it to array_decl 
            if let Some(mut base_type) = self.type_base() {
                self.array_decl(&mut base_type); // optional
                if self.consume(TokenCode::RPAR) {
                    if self.expr_cast() {
                        return true;
                    }
                }
            }else { self.err("Invalid type in cast expression"); }
            
        }
        self.pos = start_pos;

        if self._u() { return true; }
        
        false
    }

    // _u: (SUB | NOT) _u | expr_postfix

    fn _u(&mut self) -> bool {
        let start_pos = self.pos;
        if self.consume(TokenCode::SUB) || self.consume(TokenCode::NOT) {
            if self._u() {
                return true;
            }
            self.err("Missing expression after '-' or '!'");
        }
        self.pos = start_pos;

        if self.expr_postfix() { return true; }
        
        false
    }

    // expr_postfix: expr_primary (LBRACKET expr RBRACKET | DOT ID)*

    fn expr_postfix(&mut self) -> bool {
        let start_pos = self.pos;
        if self.expr_primary() {
            loop {
                if self.consume(TokenCode::LBRACKET) {
                    if !self.expr() { self.err("Missing expression inside '[...]'"); }
                    if !self.consume(TokenCode::RBRACKET) { self.err("Missing ']' after array index"); }
                    continue;
                }
                if self.consume(TokenCode::DOT) {
                    if !self.consume_id() { self.err("Missing ID after '.'"); }
                    continue;
                }
                break;
            }
            return true;
        }
        self.pos = start_pos;
        false
    }

    // expr_primary: ID (LPAR (expr (COMMA expr)*)? RPAR)? | CT_INT | CT_REAL | CT_CHAR | CT_STRING | LPAR expr RPAR

    fn expr_primary(&mut self) -> bool {
        let start_pos = self.pos;

        // ID (LPAR (expr (COMMA expr)*)? RPAR)?
        if self.consume_id() {
            if self.consume(TokenCode::LPAR) {
                if self.expr() {
                    while self.consume(TokenCode::COMMA) {
                        if !self.expr() { self.err("Expected expression after ',' in function call"); }
                    }
                }
                if !self.consume(TokenCode::RPAR) { self.err("Missing ')' after function call arguments"); }
            }
            return true;
        }

        // Constants
        if self.consume_ct_int() { return true; }
        if self.consume_ct_real() { return true; }
        if self.consume_ct_char() { return true; }
        if self.consume_ct_string() { return true; }

        // LPAR expr RPAR
        if self.consume(TokenCode::LPAR) {
            if self.expr() {
                if self.consume(TokenCode::RPAR) {
                    return true;
                } else { self.err("Missing ')' to close grouped expression"); }
            }
        }

        self.pos = start_pos;
        false
    }

}