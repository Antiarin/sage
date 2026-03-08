// AST node types for the Sage parser

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Simple(String),
    Nullable(Box<Type>),
    Generic {
        name: String,
        params: Vec<Type>,
    },
    Reference {
        mutable: bool,
        inner: Box<Type>,
    },
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub type_ann: Option<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub type_ann: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Expr,
    pub body: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Decorator {
    pub name: String,
    pub args: Vec<(String, Expr)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnSignature {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    Literal(String),
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    BoolLiteral(bool),
    Identifier(String),
    BinaryOp {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
    UnaryOp {
        op: UnOp,
        expr: Box<Expr>,
    },
    FnCall {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    MethodCall {
        object: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    FieldAccess {
        object: Box<Expr>,
        field: String,
    },
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },
    Match {
        expr: Box<Expr>,
        arms: Vec<MatchArm>,
    },
    IfElse {
        condition: Box<Expr>,
        then_block: Vec<Stmt>,
        else_block: Option<Vec<Stmt>>,
    },
    Closure {
        params: Vec<Param>,
        body: Vec<Stmt>,
    },
    ListLiteral(Vec<Expr>),
    ListComprehension {
        expr: Box<Expr>,
        var: String,
        iter: Box<Expr>,
        filter: Option<Box<Expr>>,
    },
    Spawn(Box<Expr>),
    Parallel {
        collection: Box<Expr>,
        param: String,
        body: Box<Expr>,
    },
    Scope {
        body: Vec<Stmt>,
    },
    NullCoalesce {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    SafeAccess {
        object: Box<Expr>,
        field: String,
    },
    StringInterpolation {
        parts: Vec<StringPart>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let {
        name: String,
        mutable: bool,
        type_ann: Option<Type>,
        value: Expr,
    },
    FnDef {
        name: String,
        params: Vec<Param>,
        return_type: Option<Type>,
        body: Vec<Stmt>,
        decorators: Vec<Decorator>,
    },
    StructDef {
        name: String,
        fields: Vec<Field>,
    },
    TraitDef {
        name: String,
        methods: Vec<FnSignature>,
    },
    ImplBlock {
        trait_name: Option<String>,
        target: String,
        methods: Vec<Stmt>,
    },
    Return(Option<Expr>),
    Expression(Expr),
    ForLoop {
        var: String,
        iter: Expr,
        body: Vec<Stmt>,
    },
    WhileLoop {
        condition: Expr,
        body: Vec<Stmt>,
    },
    Import {
        path: Vec<String>,
    },
    TryCatch {
        try_block: Vec<Stmt>,
        catch_var: String,
        catch_block: Vec<Stmt>,
    },
    TestFn {
        name: String,
        body: Vec<Stmt>,
    },
    AgentDef {
        name: String,
        config: Vec<(String, Expr)>,
    },
    ModuleDef {
        name: String,
        decorators: Vec<Decorator>,
        body: Vec<Stmt>,
    },
}
