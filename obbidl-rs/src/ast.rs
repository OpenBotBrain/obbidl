#[derive(Debug, Clone, PartialEq)]
pub struct Protocol<'a> {
    pub name: &'a str,
    pub roles: Option<Vec<&'a str>>,
    pub block: Block<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block<'a> {
    pub stmts: Vec<Stmt<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt<'a> {
    Message(Message<'a>),
    Par(Vec<Block<'a>>),
    Choice(Vec<Block<'a>>),
    Fin(Block<'a>),
    Inf(Block<'a>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Message<'a> {
    pub label: &'a str,
    pub payload: Option<Vec<(&'a str, Type)>>,
    pub from: &'a str,
    pub to: &'a str,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Bool,
    Int { signed: bool, size: IntSize },
    String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IntSize {
    B32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program<'a> {
    pub protocols: Vec<Protocol<'a>>,
}
