use std::borrow::Cow;
use tokenizer::Span;
use interner::RcStr;

#[derive(Clone, Debug, PartialEq)]
pub enum ParsedItem {
    Text(RcStr),
    CodeBlock(Vec<Expr>),
}

pub type UseAlias = Option<RcStr>;

#[derive(Clone, Debug, PartialEq)]
pub enum UseClause {
    QualifiedName(Path, UseAlias),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Path {
    Identifier(RcStr),
    /// An identifier which is prefixed by a namespace (e.g. a FQDN-class-path)
    /// fragment.1 = The namespace
    /// fragment.2 = The class
    NsIdentifier(RcStr, RcStr),
}

/// binary operators
#[derive(Clone, Debug, PartialEq)]
pub enum Op {
    Concat,
    // arith
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Mod,
    // logical
    Or,
    And,
    // equality
    Identical,
    NotIdentical,
    Eq,
    Neq,
    // relational
    Lt,
    Gt,
    Le,
    Ge,
    // bitwise
    BitwiseAnd,
    BitwiseInclOr,
    /// XOR
    BitwiseExclOr,
    /// spaceship operator, <=>
    Spaceship,
    Instanceof,
    Sl,
    Sr,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UnaryOp {
    Positive,
    Negative,
    Not,
    PreInc,
    PreDec,
    PostInc,
    PostDec,
    BitwiseNot,
    /// "@"" http://php.net/manual/en/language.operators.errorcontrol.php
    /// any error messages that might be generated by that expression will be ignored.
    SilenceErrors,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Visibility {
    None,
    Public,
    Private,
    Protected
}

#[derive(Clone, Debug, PartialEq)]
pub enum ClassModifier {
    None,
    Abstract,
    Final,
}

/// the boolean indicates whether the underlying item is static or not
#[derive(Clone, Debug, PartialEq)]
pub struct Modifiers(pub bool, pub Visibility, pub ClassModifier);

#[derive(Clone, Debug, PartialEq)]
pub struct Expr(pub Expr_, pub Span);

#[derive(Clone, Debug, PartialEq)]
pub enum Expr_ {
    None,
    /// indicates the path to e.g. a namespace or is a simple identifier
    Path(Path),
    String(RcStr),
    Int(i64),
    Double(f64),
    Array(Vec<(Expr, Expr)>),
    Variable(RcStr),
    Reference(Box<Expr>),
    Use(Vec<UseClause>),
    Clone(Box<Expr>),
    Exit(Box<Expr>),
    Echo(Vec<Expr>),
    Isset(Vec<Expr>),
    Empty(Box<Expr>),
    Unset(Vec<Expr>),
    Return(Box<Expr>),
    Throw(Box<Expr>),
    Break(usize),
    Continue(usize),
    Block(Vec<Expr>),

    Include(IncludeTy, Box<Expr>),
    ArrayIdx(Box<Expr>, Vec<Expr>),
    ObjMember(Box<Expr>, Vec<Expr>),
    StaticMember(Box<Expr>, Vec<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    New(Box<Expr>, Vec<Expr>),
    UnaryOp(UnaryOp, Box<Expr>),
    BinaryOp(Op, Box<Expr>, Box<Expr>),
    Cast(Ty, Box<Expr>),
    Function(FunctionDecl),

    // statements
    Assign(Box<Expr>, Box<Expr>),
    /// compound (binary) assign e.g. $test += 3; which is equal to $test = $test + 3; (Assign, BinaryOp)
    CompoundAssign(Box<Expr>, Op, Box<Expr>),
    AssignRef(Box<Expr>, Box<Expr>),
    List(Vec<(Expr, Expr)>),
    /// If (condition=.0) { Block=.1 } else Else_Expr=.2
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    While(Box<Expr>, Box<Expr>),
    DoWhile(Box<Expr>, Box<Expr>),
    /// For(initializer=.0; cond=.1; end_of_loop=.2) statement=.3
    For(Box<Expr>, Box<Expr>, Box<Expr>, Box<Expr>),
    ForEach(Box<Expr>, Box<Expr>, Box<Expr>, Box<Expr>),
    /// Try(TryBlock, CatchClauses, FinallyClause)
    Try(Box<Expr>, Vec<CatchClause>, Box<Expr>),

    /// switch (stmt=.0) [case item: body]+=.1
    /// All item-cases for a body will be included in the first-member Vec
    /// so basically we have a mapping from all-cases -> body in .1
    /// TODO: should be desugared into an if-statement
    Switch(Box<Expr>, Vec<(Vec<Expr>, Expr)>),

    /// same as if, just will pass the return-value of either expression to the parent
    /// if .1 (then) is None, the value of .0 (condition) will be used
    /// TODO: this should be desugared into an `If` during post-processing
    TernaryIf(Box<Expr>, Option<Box<Expr>>, Box<Expr>),

    // These are not actual expressions, but will be stored as such, before any filtering happens
    Decl(Decl),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Ty {
    Array,
    Callable,
    Bool,
    Float,
    Int,
    Double,
    String,
    Object,
}

#[derive(Clone, Debug, PartialEq)]
pub enum IncludeTy {
    Include,
    IncludeOnce,
    Require,
    RequireOnce,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TraitUse {
    InsteadOf((Path, Path), Vec<Path>),
    As((Path, Path), Visibility, Option<RcStr>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParamDefinition {
    pub name: RcStr,
    pub as_ref: bool,
    /// The type of the parameter
    pub ty: Option<Ty>,
    /// The default value for the parameter
    pub default: Expr,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionDecl {
    pub params: Vec<ParamDefinition>,
    pub body: Vec<Expr>,
    /// A list of variables to pass from the parent scope to the scope of this function
    /// So variables which are basically available shared into this function's scope
    pub usev: Vec<RcStr>,
    pub ret_ref: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClassDecl {
    pub cmod: ClassModifier,
    pub name: RcStr,
    pub base_class: Option<Path>,
    /// The implemented interfaces of this class
    pub implements: Vec<Path>,
    pub members: Vec<ClassMember>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ClassMember {
    Constant(RcStr, Expr),
    Property(Modifiers, RcStr, Expr),
    Method(Modifiers, RcStr, FunctionDecl),
    TraitUse(Vec<Path>, Vec<TraitUse>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Decl {
    Namespace(Vec<RcStr>),
    GlobalFunction(RcStr, FunctionDecl),
    Class(ClassDecl),
    Interface(RcStr, Vec<Path>, Vec<ClassMember>),
    Trait(RcStr, Vec<ClassMember>),
    StaticVars(Vec<(RcStr, Expr)>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CatchClause {
    pub ty: Path,
    pub var: RcStr,
    pub block: Expr,
}
