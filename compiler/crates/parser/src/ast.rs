use std::rc::Rc;
use bitflags::bitflags;
use crate::*;

#[derive(Clone)]
pub struct QualifiedIdentifier {
    pub attribute: bool,
    pub qualifier: Option<Rc<Expression>>,
    pub name: IdentifierOrBrackets,
}

impl QualifiedIdentifier {
    pub fn to_identifier(&self) -> Option<(String, Location)> {
        if self.attribute || self.qualifier.is_some() {
            return None;
        }
        if let IdentifierOrBrackets::Id(id, location) = self.name {
            if id != "*" { Some((id, location.clone())) } else { None }
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct NonAttributeQualifiedIdentifier {
    pub qualifier: Option<Rc<Expression>>,
    pub name: IdentifierOrBrackets,
}

#[derive(Clone)]
pub enum IdentifierOrBrackets {
    Id(String, Location),
    Brackets(Rc<Expression>),
}

#[derive(Clone)]
pub struct Expression {
    pub location: Location,
    pub kind: ExpressionKind,
}

#[derive(Clone)]
pub enum ExpressionKind {
    Null,
    Boolean(bool),
    Numeric(f64),
    String(String),
    This,
    RegExp {
        body: String,
        flags: String,
    },
    Id(QualifiedIdentifier),
    XmlMarkup(String),
    XmlElement(XmlElement),
    XmlList(Vec<XmlElementContent>),
    ReservedNamespace(ReservedNamespace),
    /// `()`. Used solely internally for arrow functions.
    EmptyParen,
    Paren(Rc<Expression>),
    /// Present as part of an array initializer only.
    /// This expression is not valid in other contexts.
    Rest(Rc<Expression>),
    ArrayInitializer {
        /// Element sequence possibly containing `Rest`s and ellisions.
        elements: Vec<Option<Rc<Expression>>>,
    },
    /// `new <T> []`
    VectorInitializer {
        element_type: Rc<TypeExpression>,
        /// Element sequence possibly containing `Rest`s.
        elements: Vec<Rc<Expression>>,
    },
    ObjectInitializer {
        fields: Vec<Rc<ObjectField>>,
    },
    Function {
        name: Option<(String, Location)>,
        common: Rc<FunctionCommon>,
    },
    ArrowFunction(Rc<FunctionCommon>),
    Super(Option<Vec<Rc<Expression>>>),
    New {
        base: Rc<Expression>,
        arguments: Option<Vec<Rc<Expression>>>,
    },
    /// The `o.x` expression.
    DotMember {
        base: Rc<Expression>,
        id: QualifiedIdentifier,
    },
    /// The `o[k]` expression.
    BracketsMember {
        base: Rc<Expression>,
        key: Rc<Expression>,
    },
    /// `base.<T1, Tn>`
    WithTypeArguments {
        base: Rc<Expression>,
        arguments: Vec<Rc<Expression>>,
    },
    /// The `o.(condition)` expression.
    Filter {
        base: Rc<Expression>,
        condition: Rc<Expression>,
    },
    /// The `o..x` expression.
    Descendants {
        base: Rc<Expression>,
        id: QualifiedIdentifier,
    },
    Call {
        base: Rc<Expression>,
        arguments: Vec<Rc<Expression>>,
    },
    Unary {
        base: Rc<Expression>,
        operator: Operator,
    },
    Binary {
        left: Rc<Expression>,
        operator: Operator,
        right: Rc<Expression>,
    },
    Conditional {
        test: Rc<Expression>,
        consequent: Rc<Expression>,
        alternative: Rc<Expression>,
    },
    Assignment {
        left: Rc<Destructuring>,
        compound: Option<Operator>,
        right: Rc<Expression>,
    },
    /// The `x, y` expression.
    Sequence(Rc<Expression>, Rc<Expression>),

    /// Expression used internally only. It is used for parsing
    /// arrow functions with typed parameters and return annotation.
    WithTypeAnnotation {
        base: Rc<Expression>,
        type_annotation: Rc<TypeExpression>,
    },

    Embed {
        source: String,
        type_annotation: Option<Rc<TypeExpression>>,
    },

    /// Expression containing an optional chaining operator.
    OptionalChaining {
        base: Rc<Expression>,
        /// Postfix operators that execute if the base is not `null`
        /// and not `undefined`. The topmost node in this field is
        /// [`ExpressionKind::OptionalChainingHost`], which holds
        /// a non-null and not-undefined value.
        operations: Rc<Expression>,
    },

    /// The topmost expression from which postfix operators
    /// follow in an [`ExpressionKind::OptionalChaining`] expression
    /// inside the `operations` field.
    OptionalChainingHost,
}

#[derive(Clone)]
pub enum XmlElementContent {
    Expression(Rc<Expression>),
    Markup(String, Location),
    Text(String, Location),
    Element(XmlElement),
}

#[derive(Clone)]
pub struct XmlElement {
    pub location: Location,
    pub opening_tag_name: XmlTagName,
    pub attributes: Vec<XmlAttributeOrExpression>,
    pub content: Vec<XmlElementContent>,
    pub closing_tag_name: Option<XmlTagName>,
}

#[derive(Clone)]
pub enum XmlTagName {
    Name((String, Location)),
    Expression(Rc<Expression>),
}

#[derive(Clone)]
pub enum XmlAttributeOrExpression {
    Attribute(XmlAttribute),
    Expression(Rc<Expression>),
}

#[derive(Clone)]
pub struct XmlAttribute {
    pub name: (String, Location),
    pub value: XmlAttributeValueOrExpression,
}

#[derive(Clone)]
pub enum XmlAttributeValueOrExpression {
    Value(String),
    Expression(Rc<Expression>),
}

#[derive(Clone)]
pub enum ReservedNamespace {
    Public,
    Private,
    Protected,
    Internal,
}

#[derive(Clone)]
pub enum ObjectField {
    Field {
        key: (ObjectKey, Location),
        /// Used when parsing an object initializer as a destructuring pattern.
        /// This is the result of consuming the `!` punctuator.
        #[doc(hidden)]
        destructuring_non_null: bool,
        /// If `None`, this is a shorthand field.
        value: Option<Rc<Expression>>,
    },
    Rest(Rc<Expression>, Location),
}

impl ObjectField {
    pub fn location(&self) -> Location {
        match self {
            Self::Field { key, value, .. } => {
                if let Some(value) = value {
                    key.1.combine_with(value.location.clone())
                } else {
                    key.1.clone()
                }
            },
            Self::Rest(_, location) => location.clone(),
        }
    }
}

#[derive(Clone)]
pub enum ObjectKey {
    Id(NonAttributeQualifiedIdentifier),
    String(String, Location),
    Number(f64, Location),
    Brackets(Rc<Expression>),
}

impl ObjectKey {
    pub fn to_record_destructuring_key(&self) -> RecordDestructuringKey {
        match self {
            Self::Id(id) => RecordDestructuringKey::Id(id.clone()),
            Self::String(string, location) => RecordDestructuringKey::String(string.clone(), location.clone()),
            Self::Number(number, location) => RecordDestructuringKey::Number(*number, location.clone()),
            Self::Brackets(exp) => RecordDestructuringKey::Brackets(Rc::clone(&exp)),
        }
    }
}

#[derive(Clone)]
pub struct Destructuring {
    pub location: Location,
    pub kind: DestructuringKind,
    /// Indicates whether the pattern asserts that the
    /// destructuring base is not any of `undefined` and `null`.
    /// The patterns use the `!` punctuator to indicate this behavior.
    pub non_null: bool,
    pub type_annotation: Option<Rc<TypeExpression>>,
}

#[derive(Clone)]
pub enum DestructuringKind {
    Binding {
        name: (String, Location),
    },
    Record(Vec<Rc<RecordDestructuringField>>),
    Array(Vec<Option<ArrayDestructuringItem>>),
}

#[derive(Clone)]
pub struct RecordDestructuringField {
    pub location: Location,
    pub key: (RecordDestructuringKey, Location),
    pub non_null: bool,
    pub alias: Option<Rc<Destructuring>>,
}

#[derive(Clone)]
pub enum RecordDestructuringKey {
    Id(NonAttributeQualifiedIdentifier),
    String(String, Location),
    Number(f64, Location),
    Brackets(Rc<Expression>),
}

#[derive(Clone)]
pub enum ArrayDestructuringItem {
    Pattern(Rc<Destructuring>),
    Rest(Rc<Destructuring>, Location),
}

#[derive(Clone)]
pub struct TypeExpression {
    pub location: Location,
    pub kind: TypeExpressionKind,
}

#[derive(Clone)]
pub enum TypeExpressionKind {
    Id(QualifiedIdentifier),
    DotMember {
        base: Rc<TypeExpression>,
        member: QualifiedIdentifier,
    },
    Tuple(Vec<Rc<TypeExpression>>),
    Record(Vec<Rc<RecordTypeField>>),
    /// `*`
    Any,
    Void,
    Never,
    Undefined,
    Nullable(Rc<TypeExpression>),
    NonNullable(Rc<TypeExpression>),
    Function {
        params: Vec<FunctionTypeParam>,
        return_annotation: Rc<TypeExpression>,
    },
    StringLiteral(String),
    NumberLiteral(f64),
    /// `|`
    Union(Vec<Rc<TypeExpression>>),
    /// `&`
    Complement {
        base: Rc<TypeExpression>,
        complement: Rc<TypeExpression>,
    },
    /// `base.<T1, Tn>`
    WithTypeArguments {
        base: Rc<TypeExpression>,
        arguments: Vec<Rc<TypeExpression>>,
    },
}

#[derive(Clone)]
pub struct FunctionTypeParam {
    pub kind: FunctionParamKind,
    pub name: (String, Location),
    pub type_annotation: Option<Rc<TypeExpression>>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum FunctionParamKind {
    Required = 1,
    Optional = 2,
    Rest = 3,
}

impl FunctionParamKind {
    pub fn may_be_followed_by(&self, other: Self) -> bool {
        (*self as u32) <= (other as u32)
    }
}

#[derive(Clone)]
pub struct RecordTypeField {
    pub asdoc: Option<AsDoc>,
    pub readonly: bool,
    pub key: (RecordTypeKey, Location),
    pub key_suffix: RecordTypeKeySuffix,
    pub type_annotation: Option<Rc<TypeExpression>>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum RecordTypeKeySuffix {
    None,
    NonNullable,
    Nullable,
}

#[derive(Clone)]
pub enum RecordTypeKey {
    Id(NonAttributeQualifiedIdentifier),
    String(String, Location),
    Number(f64, Location),
    Brackets(Rc<Expression>),
}

#[derive(Clone)]
pub struct Statement {
    pub location: Location,
    pub kind: StatementKind,
}

#[derive(Clone)]
pub enum StatementKind {
    Empty,
    Super(Vec<Rc<Expression>>),
    Block(Block),
    If {
        condition: Rc<Expression>,
        consequent: Rc<Statement>,
        alternative: Option<Rc<Statement>>,
    },
    Switch {
        discriminant: Rc<Expression>,
        cases: Vec<SwitchCase>,
    },
    SwitchType {
        discriminant: Rc<Expression>,
        cases: Vec<SwitchTypeCase>,
    },
    Do {
        body: Rc<Statement>,
        test: Rc<Expression>,
    },
    While {
        test: Rc<Expression>,
        body: Rc<Statement>,
    },
    For {
        init: Option<ForInit>,
        test: Option<Rc<Expression>>,
        update: Option<Rc<Expression>>,
        body: Rc<Statement>,
    },
    ForIn {
        each: bool,
        left: ForInLeft,
        right: Rc<Expression>,
        body: Rc<Statement>,
    },
    With {
        object: Rc<Expression>,
        body: Rc<Statement>,
    },
    Continue {
        label: Option<String>,
    },
    Break {
        label: Option<String>,
    },
    Return {
        expression: Option<Rc<Expression>>,
    },
    Throw {
        expression: Rc<Expression>,
    },
    Try {
        block: Block,
        catch_clauses: Vec<CatchClause>,
        finally_clause: FinallyClause,
    },
    Expression(Rc<Expression>),
    Labeled {
        label: (String, Location),
        statement: Rc<Statement>,
    },
    DefaultXmlNamespace(Rc<Expression>),
    SimpleVariableDeclaration(SimpleVariableDeclaration),
}

#[derive(Clone)]
pub struct CatchClause {
    pub pattern: Rc<Destructuring>,
    pub block: Block,
}

#[derive(Clone)]
pub struct FinallyClause {
    pub block: Block,
}

#[derive(Clone)]
pub enum ForInit {
    Variable(SimpleVariableDeclaration),
    Expression(Rc<Expression>),
}

#[derive(Clone)]
pub enum ForInLeft {
    Variable(SimpleVariableDeclaration),
    Expression(Rc<Expression>),
}

#[derive(Clone)]
pub struct SimpleVariableDeclaration {
    pub kind: (VariableKind, Location),
    pub bindings: Vec<VariableBinding>,
}

#[derive(Clone)]
pub struct VariableBinding {
    pub pattern: Rc<Destructuring>,
    pub init: Option<Rc<Expression>>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum VariableKind {
    Var,
    Const,
}

#[derive(Clone)]
pub struct SwitchCase {
    pub test: Option<Rc<Expression>>,
    pub consequent: Vec<Rc<Directive>>,
}

#[derive(Clone)]
pub struct SwitchTypeCase {
    pub pattern: Rc<Destructuring>,
    pub block: Block,
}

#[derive(Clone)]
pub struct Block(pub Vec<Rc<Directive>>);

#[derive(Clone)]
pub struct Directive {
    pub location: Location,
    pub kind: DirectiveKind,
}

#[derive(Clone)]
pub enum DirectiveKind {
    Statement(Rc<Statement>),
    Include(Rc<IncludeDirective>),
    Import(Rc<ImportDirective>),
    UseNamespace(Rc<Expression>),
    VariableDefinition(Rc<VariableDefinition>),
    FunctionDefinition(Rc<FunctionDefinition>),
    ConstructorDefinition(Rc<ConstructorDefinition>),
    GetterDefinition(Rc<GetterDefinition>),
    SetterDefinition(Rc<SetterDefinition>),
    TypeDefinition(Rc<TypeDefinition>),
    ClassDefinition(Rc<ClassDefinition>),
    EnumDefinition(Rc<EnumDefinition>),
    InterfaceDefinition(Rc<InterfaceDefinition>),
    NamespaceDefinition(Rc<NamespaceDefinition>),
}

#[derive(Clone)]
pub struct ClassDefinition {
    pub asdoc: Option<AsDoc>,
    pub annotations: DefinitionAnnotations,
    pub name: (String, Location),
    pub generics: Generics,
    pub extends_clause: Option<Rc<TypeExpression>>,
    pub implements_clause: Option<Vec<Rc<TypeExpression>>>,
    pub block: Block,
}

#[derive(Clone)]
pub struct InterfaceDefinition {
    pub asdoc: Option<AsDoc>,
    pub annotations: DefinitionAnnotations,
    pub name: (String, Location),
    pub generics: Generics,
    pub extends_clause: Option<Vec<Rc<TypeExpression>>>,
    pub block: Block,
}

#[derive(Clone)]
pub struct EnumDefinition {
    pub asdoc: Option<AsDoc>,
    pub annotations: DefinitionAnnotations,
    pub name: (String, Location),
    pub block: Block,
}

#[derive(Clone)]
pub struct NamespaceDefinition {
    pub asdoc: Option<AsDoc>,
    pub annotations: DefinitionAnnotations,
    pub left: (String, Location),
    pub right: Option<Rc<Expression>>,
}

#[derive(Clone)]
pub struct IncludeDirective {
    pub source: String,
    pub replaced_by: Vec<Rc<Directive>>,
}

/// An import directive.
/// 
/// If it is an alias with a wildcard import item,
/// it is a package alias that opens the public namespace
/// and aliases it.
/// 
/// If it is an alias with a package recursive import item,
/// it is a package set alias that opens the public namespace of
/// all the respective packages and aliases them into a namespace set.
#[derive(Clone)]
pub struct ImportDirective {
    pub alias: Option<(String, Location)>,
    pub package_name: Vec<(String, Location)>,
    pub import_item: (ImportItem, Location),
}

#[derive(Clone)]
pub enum ImportItem {
    Wildcard,
    /// `**`
    Recursive,
    Name(String),
}

#[derive(Clone)]
pub struct VariableDefinition {
    pub asdoc: Option<AsDoc>,
    pub annotations: DefinitionAnnotations,
    pub kind: VariableKind,
    pub bindings: Vec<VariableBinding>,
}

#[derive(Clone)]
pub struct FunctionDefinition {
    pub asdoc: Option<AsDoc>,
    pub annotations: DefinitionAnnotations,
    pub name: (String, Location),
    pub generics: Generics,
    pub common: Rc<FunctionCommon>,
}

#[derive(Clone)]
pub struct ConstructorDefinition {
    pub asdoc: Option<AsDoc>,
    pub annotations: DefinitionAnnotations,
    pub name: (String, Location),
    pub common: Rc<FunctionCommon>,
}

#[derive(Clone)]
pub struct GetterDefinition {
    pub asdoc: Option<AsDoc>,
    pub annotations: DefinitionAnnotations,
    pub name: (String, Location),
    pub common: Rc<FunctionCommon>,
}

#[derive(Clone)]
pub struct SetterDefinition {
    pub asdoc: Option<AsDoc>,
    pub annotations: DefinitionAnnotations,
    pub name: (String, Location),
    pub common: Rc<FunctionCommon>,
}

#[derive(Clone)]
pub struct TypeDefinition {
    pub asdoc: Option<AsDoc>,
    pub annotations: DefinitionAnnotations,
    pub left: (String, Location),
    pub generics: Generics,
    pub right: Rc<TypeExpression>,
}

#[derive(Clone)]
pub struct DefinitionAnnotations {
    pub metadata: Vec<Rc<Metadata>>,
    pub flag_modifiers: DefinitionModifiersFlags,
    pub access_modifier: Option<Rc<Expression>>,
}

bitflags! {
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub struct DefinitionModifiersFlags: u32 {
        const OVERRIDE  = 0b00000001;
        const FINAL     = 0b00000010;
        const DYNAMIC   = 0b00000100;
        const NATIVE    = 0b00001000;
        const STATIC    = 0b00010000;
    }
}

#[derive(Clone)]
pub struct Metadata {
    pub asdoc: Option<AsDoc>,
    pub location: Location,
    /// The metadata name. The metadata name may contain a single `::` delimiter.
    pub name: (String, Location),
    pub entries: Vec<MetadataEntry>,
}

#[derive(Clone)]
pub struct MetadataEntry {
    pub key: Option<(String, Location)>,
    pub value: (String, Location),
}

#[derive(Clone)]
pub struct Generics {
    pub params: Option<Vec<Rc<GenericParam>>>,
    pub where_clause: Option<GenericsWhere>,
}

#[derive(Clone)]
pub struct GenericParam {
    pub location: Location,
    pub name: (String, Location),
    pub constraints: Vec<Rc<TypeExpression>>,
    pub default_type: Option<Rc<TypeExpression>>,
}

#[derive(Clone)]
pub struct GenericsWhere {
    pub constraints: Vec<GenericsWhereConstraint>,
}

#[derive(Clone)]
pub struct GenericsWhereConstraint {
    pub name: (String, Location),
    pub constraint: Rc<TypeExpression>,
}

#[derive(Clone)]
pub struct FunctionCommon {
    pub flags: FunctionFlags,
    pub params: Vec<FunctionParam>,
    pub return_annotation: Option<Rc<TypeExpression>>,
    pub body: Option<FunctionBody>,
}

#[derive(Clone)]
pub struct FunctionParam {
    pub location: Location,
    pub kind: FunctionParamKind,
    pub binding: VariableBinding,
}

bitflags! {
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub struct FunctionFlags: u32 {
        const AWAIT     = 0b00000001;
        const YIELD     = 0b00000010;
    }
}

#[derive(Clone)]
pub enum FunctionBody {
    Block(Block),
    /// The function body is allowed to be an expression
    /// in arrow functions.
    Expression(Rc<Expression>),
}

#[derive(Clone)]
pub struct AsDoc {
    pub main_body: String,
    pub tags: Vec<AsDocTag>,
}

#[derive(Clone)]
pub enum AsDocTag {
    Copy(String),
    Default(String),
    EventType(Rc<TypeExpression>),
    Example(String),
    ExampleText(String),
    InheritDoc,
    Internal(String),
    Param {
        name: String,
        description: String,
    },
    Private,
    Return(String),
    See {
        reference: String,
        display_text: Option<String>,
    },
    Throws {
        class_name: Rc<TypeExpression>,
        description: Option<String>,
    },
}

#[derive(Clone)]
pub struct PackageDefinition {
    pub asdoc: Option<AsDoc>,
    pub location: Location,
    pub id: Vec<(String, Location)>,
    pub block: Block,
}

#[derive(Clone)]
pub struct Program {
    pub packages: Vec<Rc<PackageDefinition>>,
    pub directives: Vec<Rc<Directive>>,
}