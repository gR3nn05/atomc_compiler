use crate::token::{Token, TokenCode};

pub struct Parser{
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser{
    pub fn new(tokens: Vec<Token>) -> Self {
        Self{tokens, pos: 0 }
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

    //struct_def: STRUCT ID LACC var_def* RACC SEMICOLON

    fn struct_def(&mut self) -> bool{
        let start_pos = self.pos;
        if self.consume(TokenCode::STRUCT){
            if self.consume_id(){
                if self.consume(TokenCode::LACC){
                    while self.var_def(){} //var_def*
                    if self.consume(TokenCode::RACC){
                        if self.consume(TokenCode::SEMICOLON){
                            return true;
                        }
                        else {self.err("Expected ';' after struct definition"); }
                    }
                    else {self.err("Expected '}' in struct definition"); }
                }
            }
            else {self.err("Missing ID after 'struct'"); }
        }
        self.pos = start_pos;
        false
    }

    //var_def: type_base ID array_decl? (COMMA ID array_decl?)* SEMICOLON

    fn var_def(&mut self) -> bool{
        let start_pos = self.pos;
        if self.type_base(){
            if self.consume_id(){
                self.array_decl(); //array_decl?
                while self.consume(TokenCode::COMMA){
                    if self.consume_id(){
                        self.array_decl();
                    }else {
                        self.err("Missing variable name after ','");
                    }
                }
                if self.consume(TokenCode::SEMICOLON){
                    return true;
                }
                else {self.err("Expected ';' after var definition"); }
            }
        }
        self.pos = start_pos;
        false
    }

    //type_base: INT | DOUBLE | CHAR | STRUCT ID

    fn type_base(&mut self) -> bool{
        let start_pos = self.pos; //save initial pos

        if self.consume(TokenCode::INT) {return true;}
        if self.consume(TokenCode::DOUBLE) {return true;}
        if self.consume(TokenCode::CHAR) {return true;}
        if self.consume(TokenCode::STRUCT) {
            if self.consume_id(){
                return true;
            }
            self.err("Missing ID after 'struct'");
        }

        self.pos = start_pos; //restore pos if no match
        false
    }

    //array_decl: LBRACKET expr? RBRACKET

    fn array_decl(&mut self) -> bool{
        let start_pos = self.pos;
        if self.consume(TokenCode::LBRACKET){
            self.expr(); //changed to satisfy test 9
            if self.consume(TokenCode::RBRACKET){
                return true
            }
            self.err("Missing ']' after '['");
        }
        self.pos = start_pos;
        false
    }

    // fn_def: (type_base | VOID) ID
    //               LPAR (fn_param (COMMA fn_param)*)? RPAR
    //               stm_compound 

    fn fn_def(&mut self) -> bool{
        let start_pos = self.pos;
        let mut has_type = false;

        if self.type_base() || self.consume(TokenCode::VOID){
            has_type = true;
        }

        if has_type{
            if self.consume_id(){
                if self.consume(TokenCode::LPAR){
                    if self.fn_param(){
                        while self.consume(TokenCode::COMMA){
                            if !self.fn_param() {self.err("Expected function parameters after ','"); }
                        }
                    }
                    if self.consume(TokenCode::RPAR){
                        if self.stm_compound(){
                            return true;
                        } else {self.err("Expexted compond statement '{...}' from function body"); }
                    } else {self.err("Missing ')' in function definition"); }
                }
            }
        }
        self.pos = start_pos;
        false
    }

    //fn_param: type_base ID array_decl?

    fn fn_param(&mut self) -> bool {
        let start_pos = self.pos;
        if self.type_base() {
            if self.consume_id() {
                self.array_decl(); // optional
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
        if self.stm_compound() {return true; }

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

    //stm_compound: LACC (var_def | stm)* RACC

    fn stm_compound(&mut self) -> bool{
        let start_pos = self.pos;
        if self.consume(TokenCode::LACC){
            loop{
                // loop as long as we match either a var_def OR a stm
                if self.var_def() { continue; }
                if self.stm() { continue; }
                break;
            }

            if self.consume(TokenCode::RACC){
                return true;
            }
            self.err("Missing '}' at the end of compound statement");
        }
        self.pos = start_pos;
        false
    }

    //Expressions

    //expr: expr_assign

    fn expr(&mut self) -> bool{
        self.expr_assign()
    }

    //expr_assign: expr_unary ASSIGN expr_assign | expr_or

    fn expr_assign(&mut self) -> bool{
        let start_pos = self.pos;

        if self.expr_unary() {
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
            if self.type_base() {
                self.array_decl(); // optional
                if self.consume(TokenCode::RPAR) {
                    if self.expr_cast() {
                        return true;
                    }
                }
            }
        }
        self.pos = start_pos;

        if self.expr_unary() { return true; }
        
        false
    }

    // expr_unary: (SUB | NOT) expr_unary | expr_postfix

    fn expr_unary(&mut self) -> bool {
        let start_pos = self.pos;
        if self.consume(TokenCode::SUB) || self.consume(TokenCode::NOT) {
            if self.expr_unary() {
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