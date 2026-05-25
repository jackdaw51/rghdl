use std::iter::Peekable;

use crate::lexer::{Lexer, Span, Token, TokenKind};

//Let's make this shi arena allocated
pub struct Parser<'a> {
    lexer: Peekable<Lexer<'a>>,
    pub arena: AstArena<'a>,
    source: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContextId(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortId(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityId(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeclId(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StmtId(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArchitectureId(pub u32);

#[derive(Debug, Clone)]
pub enum ContextItem<'a> {
    Library { name: &'a str },
    Use { path: &'a str },
}

#[derive(Debug, Clone, PartialEq)]
pub enum PortMode {
    In,
    Out,
    InOut,
    Buffer,
}

#[derive(Debug, Clone)]
pub struct Port<'a> {
    pub name: &'a str,
    pub mode: PortMode,
    pub port_type: &'a str,
}

#[derive(Debug, Clone)]
pub struct Entity<'a> {
    pub name: &'a str,
    pub ports_start: PortId,
    pub ports_end: PortId,
}

#[derive(Debug, Clone)]
pub enum Decl<'a> {
    Signal {
        name: &'a str,
        decl_type: &'a str,
    },
    Constant {
        name: &'a str,
        decl_type: &'a str,
    },
    Variable {
        name: &'a str,
        decl_type: &'a str,
    },
    Component {
        name: &'a str,
        ports_start: PortId,
        ports_end: PortId,
    },
    //TODO: user-defined types, functions and procedures
}

#[derive(Debug, Clone)]
pub enum Stmt<'a> {
    ConcurrentAssignment {
        target: &'a str,
        expression_span: Span,
    },

    // out_port <= a when control = '1' else b;
    ConditionalAssignment {
        target: &'a str,
    },

    // u_gate: and_gate port map (A => in1, B => in2, Y => out_port);
    ComponentInstantiation {
        label: &'a str,
        component_name: &'a str,
        port_map_span: Span,
    },

    // My_Process: process(clk) begin ... end process;
    Process {
        label: Option<&'a str>,
        stmts_start: StmtId,
        stmts_end: StmtId,
    },

    // sig <= '1'; it looks like concurrent, but it's inside a clocked process
    SequentialAssignment {
        target: &'a str,
        expression_span: Span,
    },

    // var := var + 1;
    VariableAssignment {
        target: &'a str,
        expression_span: Span,
    },

    // if condition then ... else ... end if;
    If {
        condition_span: Span,
        then_start: StmtId,
        then_end: StmtId,
        else_start: StmtId,
        else_end: StmtId,
    },

    // case state is when IDLE => ... when others => ... end case;
    Case {
        expression_span: Span,
        cases_span: Span,
    },

    // for i in 0 to 7 loop ... end loop;
    Loop {
        label: Option<&'a str>,
        loop_scheme_span: Span, // "for i in 0 to 7"
        stmts_start: StmtId,
        stmts_end: StmtId,
    },
}

#[derive(Debug, Clone)]
pub struct Architecture<'a> {
    pub name: &'a str,
    pub entity_name: &'a str,
    pub decls_start: DeclId,
    pub decls_end: DeclId,
    pub stmts_start: StmtId,
    pub stmts_end: StmtId,
}
#[derive(Default, Debug)]
pub struct AstArena<'a> {
    pub contexts: Vec<ContextItem<'a>>,
    pub ports: Vec<Port<'a>>,
    pub entities: Vec<Entity<'a>>,
    pub decls: Vec<Decl<'a>>,
    pub stmts: Vec<Stmt<'a>>,
    pub architectures: Vec<Architecture<'a>>,
}
impl<'a> Parser<'a> {
    pub(crate) fn new(source: &'a str) -> Self {
        Self {
            lexer: Lexer::new(source).peekable(),
            arena: AstArena::new(),
            source,
        }
    }
    fn advance(&mut self) -> Token {
        self.lexer.next().unwrap_or_else(|| Token {
            kind: TokenKind::Eof,
            span: Span { start: 0, end: 0 },
        })
    }
    fn expect(&mut self, expected: TokenKind) -> Token {
        let token = self.advance();
        if token.kind != expected {
            panic!(
                "Syntax Error: Expected {:?}, but found {:?} around: {}",
                expected,
                token.kind,
                self.get_text(token.span)
            );
        }
        token
    }
    fn get_text(&self, span: Span) -> &'a str {
        &self.source[span.start..span.end]
    }

    pub(crate) fn parse(&mut self) {
        while let Some(token) = self.lexer.peek() {
            match token.kind {
                TokenKind::KwEntity => {
                    self.parse_entity();
                }
                TokenKind::KwArchitecture => {
                    self.parse_architecture();
                }
                TokenKind::KwLibrary | TokenKind::KwUse => {
                    self.parse_lib();
                }
                TokenKind::Eof => break,

                _ => {
                    todo!()
                }
            }

            println!("{:?}", self.arena);
        }
    }

    fn parse_entity(&mut self) -> EntityId {
        self.advance();
        let name_token = self.expect(TokenKind::Identifier);
        let entity_name = self.get_text(name_token.span);

        self.expect(TokenKind::KwIs);

        let start_port_id = self.arena.ports.len() as u32;

        if let Some(tok) = self.lexer.peek() {
            if tok.kind == TokenKind::KwPort {
                self.advance();
                self.expect(TokenKind::LParen);

                loop {
                    self.parse_port();

                    let next = self.lexer.peek().unwrap();
                    if next.kind == TokenKind::Semicolon {
                        self.advance();
                    } else if next.kind == TokenKind::RParen {
                        break;
                    } else {
                        panic!("Syntax Error: Expected ';' or ')' after port declaration");
                    }
                }

                self.expect(TokenKind::RParen);
                self.expect(TokenKind::Semicolon);
            }
        }

        let end_port_id = self.arena.ports.len() as u32;
        self.expect(TokenKind::KwEnd);

        // VHDL allows end [entity] [my_entity]
        if let Some(t) = self.lexer.peek() {
            if t.kind == TokenKind::KwEntity {
                self.advance();
            }
        }

        let is_identifier = self
            .lexer
            .peek()
            .map(|t| t.kind == TokenKind::Identifier)
            .unwrap_or(false);

        if is_identifier {
            let t = self.advance();
            if self.get_text(t.span) != entity_name {
                panic!(
                    "Syntax error: End label '{}' and entity name '{}' should match",
                    self.get_text(t.span),
                    entity_name
                );
            }
        }

        self.expect(TokenKind::Semicolon);

        let entity = Entity {
            name: entity_name,
            ports_start: PortId(start_port_id),
            ports_end: PortId(end_port_id),
        };

        self.arena.alloc_entity(entity)
    }

    fn parse_architecture(&mut self) -> ArchitectureId {
        self.advance();

        let arch_name_tok = self.expect(TokenKind::Identifier);
        let arch_name = self.get_text(arch_name_tok.span);

        self.expect(TokenKind::KwOf);

        let entity_name_tok = self.expect(TokenKind::Identifier);
        let entity_name = self.get_text(entity_name_tok.span);

        self.expect(TokenKind::KwIs);

        let decls_start = self.arena.decls.len() as u32;

        while let Some(tok) = self.lexer.peek() {
            if tok.kind == TokenKind::KwBegin {
                break;
            }
            self.parse_architecture_declaration();
        }

        let decls_end = self.arena.decls.len() as u32;

        self.expect(TokenKind::KwBegin);

        let stmts_start = self.arena.stmts.len() as u32;

        while let Some(tok) = self.lexer.peek() {
            if tok.kind == TokenKind::KwEnd {
                break;
            }
            // TODO: Parse concurrent assignments, processes, component instantiations
            // self.parse_concurrent_statement();
            self.advance(); //placeholder
        }

        let stmts_end = self.arena.stmts.len() as u32;

        self.expect(TokenKind::KwEnd);

        //same with entity, possible are "end [architecture] [my_architecture]"";

        if let Some(t) = self.lexer.peek() {
            if t.kind == TokenKind::KwArchitecture {
                self.advance();
            }
        }

        let is_identifier = self
            .lexer
            .peek()
            .map(|t| t.kind == TokenKind::Identifier)
            .unwrap_or(false);

        if is_identifier {
            let t = self.advance();
            let end_name = self.get_text(t.span);

            if end_name != arch_name {
                panic!(
                    "Syntax Error: End label '{}' does not match architecture name '{}'",
                    end_name, arch_name
                );
            }
        }
        dbg!(is_identifier);
        self.expect(TokenKind::Semicolon);

        let arch = Architecture {
            name: arch_name,
            entity_name,
            decls_start: DeclId(decls_start),
            decls_end: DeclId(decls_end),
            stmts_start: StmtId(stmts_start),
            stmts_end: StmtId(stmts_end),
        };

        self.arena.alloc_architecture(arch)
    }

    fn parse_lib(&mut self) -> ContextId {
        let start_tok = self.advance();

        match start_tok.kind {
            TokenKind::KwLibrary => {
                let name_tok = self.expect(TokenKind::Identifier);
                let name = self.get_text(name_tok.span);
                self.expect(TokenKind::Semicolon);

                self.arena.alloc_context(ContextItem::Library { name })
            }
            TokenKind::KwUse => {
                let mut path_start = 0;
                let mut path_end = 0;
                let mut first = true;

                while let Some(tok) = self.lexer.peek() {
                    if tok.kind == TokenKind::Semicolon {
                        break;
                    }
                    let consumed = self.advance();
                    if first {
                        path_start = consumed.span.start;
                        first = false;
                    }
                    path_end = consumed.span.end;
                }

                let path = &self.source[path_start..path_end];
                self.expect(TokenKind::Semicolon);

                self.arena.alloc_context(ContextItem::Use { path })
            }
            _ => panic!("Expected library or use clause"),
        }
    }

    //TODO: handle comma-separated names
    fn parse_port(&mut self) -> PortId {
        let name_tok = self.expect(TokenKind::Identifier);
        let name = self.get_text(name_tok.span);

        self.expect(TokenKind::Colon);

        let mode = if let Some(tok) = self.lexer.peek() {
            match tok.kind {
                TokenKind::KwIn => {
                    self.advance();
                    PortMode::In
                }
                TokenKind::KwOut => {
                    self.advance();
                    PortMode::Out
                }
                TokenKind::KwInOut => {
                    self.advance();
                    PortMode::InOut
                }
                TokenKind::KwBuffer => {
                    self.advance();
                    PortMode::Buffer
                }
                _ => PortMode::In,
            }
        } else {
            PortMode::In
        };

        let mut paren_depth = 0;
        let type_start = self.lexer.peek().map(|t| t.span.start).unwrap_or(0);
        let mut type_end = type_start;

        while let Some(tok) = self.lexer.peek() {
            match tok.kind {
                TokenKind::LParen => paren_depth += 1,
                TokenKind::RParen if paren_depth > 0 => paren_depth -= 1,
                TokenKind::RParen if paren_depth == 0 => break,
                TokenKind::Semicolon if paren_depth == 0 => break,
                _ => {}
            }
            type_end = self.advance().span.end;
        }

        let port_type = self.source[type_start..type_end].trim();

        let port = Port {
            name,
            mode,
            port_type,
        };

        self.arena.alloc_port(port)
    }

    fn parse_architecture_declaration(&mut self) -> DeclId {
        let start_tok = self.advance();

        let is_signal = match start_tok.kind {
            TokenKind::KwSignal => true,
            TokenKind::KwConstant => false,
            // Let's start with this for now
            _ => panic!(
                "Syntax Error: Expected 'signal' or 'constant', found {:?} at byte {}",
                start_tok.kind, start_tok.span.start
            ),
        };

        let name_tok = self.expect(TokenKind::Identifier);
        let name = self.get_text(name_tok.span);

        self.expect(TokenKind::Colon);

        let mut paren_depth = 0;
        let type_start = self.lexer.peek().map(|t| t.span.start).unwrap_or(0);
        let mut type_end = type_start;

        while let Some(tok) = self.lexer.peek() {
            match tok.kind {
                TokenKind::LParen => paren_depth += 1,
                TokenKind::RParen if paren_depth > 0 => paren_depth -= 1,
                TokenKind::Semicolon if paren_depth == 0 => break,

                _ => {}
            }
            type_end = self.advance().span.end;
        }

        let decl_type = self.source[type_start..type_end].trim();

        self.expect(TokenKind::Semicolon);

        let decl = if is_signal {
            Decl::Signal { name, decl_type }
        } else {
            Decl::Constant { name, decl_type }
        };

        self.arena.alloc_decl(decl)
    }
}

impl<'a> AstArena<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn alloc_port(&mut self, port: Port<'a>) -> PortId {
        let id = self.ports.len() as u32;
        self.ports.push(port);
        PortId(id)
    }

    pub fn alloc_entity(&mut self, entity: Entity<'a>) -> EntityId {
        let id = self.entities.len() as u32;
        self.entities.push(entity);
        EntityId(id)
    }
    pub fn alloc_context(&mut self, item: ContextItem<'a>) -> ContextId {
        let id = self.contexts.len() as u32;
        self.contexts.push(item);
        ContextId(id)
    }
    pub fn alloc_decl(&mut self, decl: Decl<'a>) -> DeclId {
        let id = self.decls.len() as u32;
        self.decls.push(decl);
        DeclId(id)
    }
    pub fn alloc_stmt(&mut self, stmt: Stmt<'a>) -> StmtId {
        let id = self.stmts.len() as u32;
        self.stmts.push(stmt);
        StmtId(id)
    }

    pub fn alloc_architecture(&mut self, arch: Architecture<'a>) -> ArchitectureId {
        let id = self.architectures.len() as u32;
        self.architectures.push(arch);
        ArchitectureId(id)
    }
}
