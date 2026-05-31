//! Concrete Solidity AST node definitions (serde-deserializable from solc JSON).
#![allow(clippy::large_enum_variant)]

use super::NodeID;
use serde::Deserialize;
use serde_json::Value;

/// solc `typeDescriptions` block — present on most expressions / declarations.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeDescriptions {
    #[serde(default)]
    pub type_identifier: Option<String>,
    #[serde(default)]
    pub type_string: Option<String>,
}

/// A trait every concrete node implements, exposing its id + source location.
pub trait Node {
    fn id(&self) -> NodeID;
    fn src(&self) -> &str;
}

macro_rules! impl_node {
    ($($t:ty),+ $(,)?) => {
        $(impl Node for $t {
            fn id(&self) -> NodeID { self.id }
            fn src(&self) -> &str { &self.src }
        })+
    };
}

// ===========================================================================
// Source unit
// ===========================================================================

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceUnit {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub absolute_path: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub exported_symbols: Value,
    #[serde(default)]
    pub nodes: Vec<SourceUnitNode>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "nodeType")]
pub enum SourceUnitNode {
    PragmaDirective(PragmaDirective),
    ImportDirective(ImportDirective),
    ContractDefinition(ContractDefinition),
    StructDefinition(StructDefinition),
    EnumDefinition(EnumDefinition),
    ErrorDefinition(ErrorDefinition),
    FunctionDefinition(FunctionDefinition),
    VariableDeclaration(VariableDeclaration),
    UserDefinedValueTypeDefinition(UserDefinedValueTypeDefinition),
    UsingForDirective(UsingForDirective),
    #[serde(other)]
    Unsupported,
}

// ===========================================================================
// Directives & top-level declarations
// ===========================================================================

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PragmaDirective {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub literals: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportDirective {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub file: String,
    #[serde(default)]
    pub absolute_path: Option<String>,
    #[serde(default)]
    pub unit_alias: String,
    #[serde(default)]
    pub symbol_aliases: Value,
    #[serde(default)]
    pub source_unit: Option<NodeID>,
    #[serde(default)]
    pub scope: Option<NodeID>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractDefinition {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub contract_kind: Option<String>, // "contract" | "interface" | "library"
    #[serde(rename = "abstract", default)]
    pub is_abstract: bool,
    #[serde(default)]
    pub base_contracts: Vec<InheritanceSpecifier>,
    #[serde(default)]
    pub linearized_base_contracts: Vec<NodeID>,
    #[serde(default)]
    pub fully_implemented: Option<bool>,
    #[serde(default)]
    pub documentation: Option<Value>,
    #[serde(default)]
    pub nodes: Vec<ContractBodyElement>,
}

/// Children of a `ContractDefinition`.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "nodeType")]
pub enum ContractBodyElement {
    FunctionDefinition(FunctionDefinition),
    VariableDeclaration(VariableDeclaration),
    ModifierDefinition(ModifierDefinition),
    EventDefinition(EventDefinition),
    ErrorDefinition(ErrorDefinition),
    StructDefinition(StructDefinition),
    EnumDefinition(EnumDefinition),
    UsingForDirective(UsingForDirective),
    UserDefinedValueTypeDefinition(UserDefinedValueTypeDefinition),
    #[serde(other)]
    Unsupported,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InheritanceSpecifier {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub base_name: Option<IdentifierPath>,
    #[serde(default)]
    pub arguments: Option<Vec<Expression>>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionDefinition {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub kind: Option<String>, // "function" | "constructor" | "fallback" | "receive" | "freeFunction"
    #[serde(default)]
    pub visibility: Option<String>, // "public" | "external" | "internal" | "private"
    #[serde(default)]
    pub state_mutability: Option<String>, // "nonpayable" | "payable" | "view" | "pure"
    #[serde(rename = "virtual", default)]
    pub is_virtual: bool,
    #[serde(rename = "override", default)]
    pub overrides: Option<OverrideSpecifier>,
    #[serde(default)]
    pub implemented: bool,
    #[serde(default)]
    pub function_selector: Option<String>,
    #[serde(default)]
    pub modifiers: Vec<ModifierInvocation>,
    #[serde(default)]
    pub parameters: ParameterList,
    #[serde(default)]
    pub return_parameters: ParameterList,
    #[serde(default)]
    pub body: Option<Block>,
    #[serde(default)]
    pub base_functions: Vec<NodeID>,
    #[serde(default)]
    pub documentation: Option<Value>,
    #[serde(default)]
    pub scope: Option<NodeID>,
}

impl FunctionDefinition {
    pub fn kind_str(&self) -> &str {
        self.kind.as_deref().unwrap_or("function")
    }
    pub fn is_constructor(&self) -> bool {
        self.kind_str() == "constructor"
    }
    pub fn visibility_str(&self) -> &str {
        self.visibility.as_deref().unwrap_or("")
    }
    pub fn mutability_str(&self) -> &str {
        self.state_mutability.as_deref().unwrap_or("nonpayable")
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModifierDefinition {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub visibility: Option<String>,
    #[serde(rename = "virtual", default)]
    pub is_virtual: bool,
    #[serde(rename = "override", default)]
    pub overrides: Option<OverrideSpecifier>,
    #[serde(default)]
    pub parameters: ParameterList,
    #[serde(default)]
    pub body: Option<Block>,
    #[serde(default)]
    pub base_modifiers: Vec<NodeID>,
    #[serde(default)]
    pub documentation: Option<Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModifierInvocation {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub modifier_name: Option<IdentifierPath>,
    #[serde(default)]
    pub arguments: Option<Vec<Expression>>,
    #[serde(default)]
    pub kind: Option<String>, // "modifierInvocation" | "baseConstructorSpecifier"
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverrideSpecifier {
    #[serde(default)]
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub overrides: Vec<Value>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParameterList {
    #[serde(default)]
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub parameters: Vec<VariableDeclaration>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VariableDeclaration {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub type_name: Option<TypeName>,
    #[serde(default)]
    pub constant: bool,
    #[serde(default)]
    pub mutability: Option<String>, // "mutable" | "immutable" | "constant"
    #[serde(default)]
    pub state_variable: bool,
    #[serde(default)]
    pub storage_location: Option<String>, // "default" | "storage" | "memory" | "calldata"
    #[serde(default)]
    pub visibility: Option<String>,
    #[serde(default)]
    pub value: Option<Expression>,
    #[serde(default)]
    pub indexed: bool,
    #[serde(default)]
    pub function_selector: Option<String>,
    #[serde(default)]
    pub scope: Option<NodeID>,
    #[serde(default)]
    pub type_descriptions: TypeDescriptions,
}

impl VariableDeclaration {
    pub fn is_constant(&self) -> bool {
        self.constant || self.mutability.as_deref() == Some("constant")
    }
    pub fn is_immutable(&self) -> bool {
        self.mutability.as_deref() == Some("immutable")
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventDefinition {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub anonymous: bool,
    #[serde(default)]
    pub event_selector: Option<String>,
    #[serde(default)]
    pub parameters: ParameterList,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorDefinition {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub error_selector: Option<String>,
    #[serde(default)]
    pub parameters: ParameterList,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StructDefinition {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub canonical_name: Option<String>,
    #[serde(default)]
    pub visibility: Option<String>,
    #[serde(default)]
    pub members: Vec<VariableDeclaration>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnumDefinition {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub canonical_name: Option<String>,
    #[serde(default)]
    pub members: Vec<EnumValue>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnumValue {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub name: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDefinedValueTypeDefinition {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub underlying_type: Option<TypeName>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsingForDirective {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub global: bool,
    #[serde(default)]
    pub library_name: Option<IdentifierPath>,
    #[serde(default)]
    pub type_name: Option<TypeName>,
    #[serde(default)]
    pub function_list: Value,
}

// ===========================================================================
// Type names
// ===========================================================================

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "nodeType")]
pub enum TypeName {
    ElementaryTypeName(ElementaryTypeName),
    UserDefinedTypeName(UserDefinedTypeName),
    FunctionTypeName(FunctionTypeName),
    Mapping(Mapping),
    ArrayTypeName(ArrayTypeName),
    #[serde(other)]
    Unsupported,
}

impl TypeName {
    pub fn type_string(&self) -> Option<&str> {
        match self {
            TypeName::ElementaryTypeName(t) => t.type_descriptions.type_string.as_deref(),
            TypeName::UserDefinedTypeName(t) => t.type_descriptions.type_string.as_deref(),
            TypeName::FunctionTypeName(t) => t.type_descriptions.type_string.as_deref(),
            TypeName::Mapping(t) => t.type_descriptions.type_string.as_deref(),
            TypeName::ArrayTypeName(t) => t.type_descriptions.type_string.as_deref(),
            TypeName::Unsupported => None,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElementaryTypeName {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub state_mutability: Option<String>,
    #[serde(default)]
    pub type_descriptions: TypeDescriptions,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDefinedTypeName {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub path_node: Option<IdentifierPath>,
    #[serde(default)]
    pub referenced_declaration: Option<NodeID>,
    #[serde(default)]
    pub type_descriptions: TypeDescriptions,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionTypeName {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub visibility: Option<String>,
    #[serde(default)]
    pub state_mutability: Option<String>,
    #[serde(default)]
    pub type_descriptions: TypeDescriptions,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mapping {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub key_type: Option<Box<TypeName>>,
    #[serde(default)]
    pub value_type: Option<Box<TypeName>>,
    #[serde(default)]
    pub type_descriptions: TypeDescriptions,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArrayTypeName {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub base_type: Option<Box<TypeName>>,
    #[serde(default)]
    pub length: Option<Expression>,
    #[serde(default)]
    pub type_descriptions: TypeDescriptions,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentifierPath {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub referenced_declaration: Option<NodeID>,
}

// ===========================================================================
// Statements
// ===========================================================================

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "nodeType")]
pub enum Statement {
    Block(Block),
    ExpressionStatement(ExpressionStatement),
    VariableDeclarationStatement(VariableDeclarationStatement),
    IfStatement(IfStatement),
    ForStatement(ForStatement),
    WhileStatement(WhileStatement),
    DoWhileStatement(DoWhileStatement),
    Return(Return),
    EmitStatement(EmitStatement),
    RevertStatement(RevertStatement),
    TryStatement(TryStatement),
    UncheckedBlock(UncheckedBlock),
    InlineAssembly(InlineAssembly),
    PlaceholderStatement(PlaceholderStatement),
    Break(BreakStatement),
    Continue(ContinueStatement),
    #[serde(other)]
    Unsupported,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    #[serde(default)]
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub statements: Vec<Statement>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UncheckedBlock {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub statements: Vec<Statement>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpressionStatement {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    pub expression: Expression,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VariableDeclarationStatement {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub declarations: Vec<Option<VariableDeclaration>>,
    #[serde(default)]
    pub initial_value: Option<Expression>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IfStatement {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    pub condition: Expression,
    pub true_body: Box<Statement>,
    #[serde(default)]
    pub false_body: Option<Box<Statement>>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForStatement {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub initialization_expression: Option<Box<Statement>>,
    #[serde(default)]
    pub condition: Option<Expression>,
    #[serde(default)]
    pub loop_expression: Option<Box<Statement>>,
    pub body: Box<Statement>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhileStatement {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    pub condition: Expression,
    pub body: Box<Statement>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DoWhileStatement {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    pub condition: Expression,
    pub body: Box<Statement>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Return {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub expression: Option<Expression>,
    #[serde(default)]
    pub function_return_parameters: Option<NodeID>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmitStatement {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    pub event_call: FunctionCall,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RevertStatement {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    pub error_call: FunctionCall,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TryStatement {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    pub external_call: FunctionCall,
    #[serde(default)]
    pub clauses: Vec<TryCatchClause>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TryCatchClause {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub error_name: String,
    #[serde(default)]
    pub parameters: Option<ParameterList>,
    #[serde(default)]
    pub block: Block,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InlineAssembly {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    #[serde(rename = "AST", default)]
    pub ast: Option<Value>,
    #[serde(default)]
    pub evm_version: Option<String>,
    #[serde(default)]
    pub external_references: Value,
    #[serde(default)]
    pub flags: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceholderStatement {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BreakStatement {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinueStatement {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
}

// ===========================================================================
// Expressions
// ===========================================================================

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "nodeType")]
pub enum Expression {
    Assignment(Assignment),
    BinaryOperation(BinaryOperation),
    UnaryOperation(UnaryOperation),
    FunctionCall(FunctionCall),
    FunctionCallOptions(FunctionCallOptions),
    MemberAccess(MemberAccess),
    IndexAccess(IndexAccess),
    IndexRangeAccess(IndexRangeAccess),
    Identifier(Identifier),
    Literal(Literal),
    Conditional(Conditional),
    TupleExpression(TupleExpression),
    NewExpression(NewExpression),
    ElementaryTypeNameExpression(ElementaryTypeNameExpression),
    #[serde(other)]
    Unsupported,
}

impl Expression {
    pub fn type_string(&self) -> Option<&str> {
        self.type_descriptions().and_then(|t| t.type_string.as_deref())
    }
    pub fn type_descriptions(&self) -> Option<&TypeDescriptions> {
        match self {
            Expression::Assignment(e) => Some(&e.type_descriptions),
            Expression::BinaryOperation(e) => Some(&e.type_descriptions),
            Expression::UnaryOperation(e) => Some(&e.type_descriptions),
            Expression::FunctionCall(e) => Some(&e.type_descriptions),
            Expression::FunctionCallOptions(e) => Some(&e.type_descriptions),
            Expression::MemberAccess(e) => Some(&e.type_descriptions),
            Expression::IndexAccess(e) => Some(&e.type_descriptions),
            Expression::IndexRangeAccess(e) => Some(&e.type_descriptions),
            Expression::Identifier(e) => Some(&e.type_descriptions),
            Expression::Literal(e) => Some(&e.type_descriptions),
            Expression::Conditional(e) => Some(&e.type_descriptions),
            Expression::TupleExpression(e) => Some(&e.type_descriptions),
            Expression::NewExpression(e) => Some(&e.type_descriptions),
            Expression::ElementaryTypeNameExpression(e) => Some(&e.type_descriptions),
            Expression::Unsupported => None,
        }
    }
    /// The `src` location string of the expression (empty for unsupported).
    pub fn src(&self) -> &str {
        match self {
            Expression::Assignment(e) => &e.src,
            Expression::BinaryOperation(e) => &e.src,
            Expression::UnaryOperation(e) => &e.src,
            Expression::FunctionCall(e) => &e.src,
            Expression::FunctionCallOptions(e) => &e.src,
            Expression::MemberAccess(e) => &e.src,
            Expression::IndexAccess(e) => &e.src,
            Expression::IndexRangeAccess(e) => &e.src,
            Expression::Identifier(e) => &e.src,
            Expression::Literal(e) => &e.src,
            Expression::Conditional(e) => &e.src,
            Expression::TupleExpression(e) => &e.src,
            Expression::NewExpression(e) => &e.src,
            Expression::ElementaryTypeNameExpression(e) => &e.src,
            Expression::Unsupported => "",
        }
    }
    /// Best-effort id for the expression (0 for the unsupported variant).
    pub fn id(&self) -> NodeID {
        match self {
            Expression::Assignment(e) => e.id,
            Expression::BinaryOperation(e) => e.id,
            Expression::UnaryOperation(e) => e.id,
            Expression::FunctionCall(e) => e.id,
            Expression::FunctionCallOptions(e) => e.id,
            Expression::MemberAccess(e) => e.id,
            Expression::IndexAccess(e) => e.id,
            Expression::IndexRangeAccess(e) => e.id,
            Expression::Identifier(e) => e.id,
            Expression::Literal(e) => e.id,
            Expression::Conditional(e) => e.id,
            Expression::TupleExpression(e) => e.id,
            Expression::NewExpression(e) => e.id,
            Expression::ElementaryTypeNameExpression(e) => e.id,
            Expression::Unsupported => 0,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Assignment {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    pub operator: String,
    pub left_hand_side: Box<Expression>,
    pub right_hand_side: Box<Expression>,
    #[serde(default)]
    pub type_descriptions: TypeDescriptions,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BinaryOperation {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    pub operator: String,
    pub left_expression: Box<Expression>,
    pub right_expression: Box<Expression>,
    #[serde(default)]
    pub common_type: TypeDescriptions,
    #[serde(default)]
    pub type_descriptions: TypeDescriptions,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnaryOperation {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    pub operator: String,
    #[serde(default)]
    pub prefix: bool,
    pub sub_expression: Box<Expression>,
    #[serde(default)]
    pub type_descriptions: TypeDescriptions,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionCall {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub kind: Option<String>, // "functionCall" | "typeConversion" | "structConstructorCall"
    pub expression: Box<Expression>,
    #[serde(default)]
    pub arguments: Vec<Expression>,
    #[serde(default)]
    pub names: Vec<String>,
    #[serde(default)]
    pub try_call: bool,
    #[serde(default)]
    pub type_descriptions: TypeDescriptions,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionCallOptions {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    pub expression: Box<Expression>,
    #[serde(default)]
    pub options: Vec<Expression>,
    #[serde(default)]
    pub names: Vec<String>,
    #[serde(default)]
    pub type_descriptions: TypeDescriptions,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberAccess {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    pub expression: Box<Expression>,
    #[serde(default)]
    pub member_name: String,
    #[serde(default)]
    pub referenced_declaration: Option<NodeID>,
    #[serde(default)]
    pub type_descriptions: TypeDescriptions,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexAccess {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    pub base_expression: Box<Expression>,
    #[serde(default)]
    pub index_expression: Option<Box<Expression>>,
    #[serde(default)]
    pub type_descriptions: TypeDescriptions,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexRangeAccess {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    pub base_expression: Box<Expression>,
    #[serde(default)]
    pub start_expression: Option<Box<Expression>>,
    #[serde(default)]
    pub end_expression: Option<Box<Expression>>,
    #[serde(default)]
    pub type_descriptions: TypeDescriptions,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Identifier {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub referenced_declaration: Option<NodeID>,
    #[serde(default)]
    pub overloaded_declarations: Vec<NodeID>,
    #[serde(default)]
    pub type_descriptions: TypeDescriptions,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Literal {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub kind: Option<String>, // "number" | "string" | "bool" | "hexString" | "unicodeString"
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub hex_value: Option<String>,
    #[serde(default)]
    pub subdenomination: Option<String>,
    #[serde(default)]
    pub type_descriptions: TypeDescriptions,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Conditional {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    pub condition: Box<Expression>,
    pub true_expression: Box<Expression>,
    pub false_expression: Box<Expression>,
    #[serde(default)]
    pub type_descriptions: TypeDescriptions,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TupleExpression {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub components: Vec<Option<Expression>>,
    #[serde(default)]
    pub is_inline_array: bool,
    #[serde(default)]
    pub type_descriptions: TypeDescriptions,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewExpression {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub type_name: Option<TypeName>,
    #[serde(default)]
    pub type_descriptions: TypeDescriptions,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElementaryTypeNameExpression {
    pub id: NodeID,
    #[serde(default)]
    pub src: String,
    #[serde(default)]
    pub type_name: Value,
    #[serde(default)]
    pub type_descriptions: TypeDescriptions,
}

impl_node!(
    SourceUnit, PragmaDirective, ImportDirective, ContractDefinition, InheritanceSpecifier,
    FunctionDefinition, ModifierDefinition, ModifierInvocation, OverrideSpecifier, ParameterList,
    VariableDeclaration, EventDefinition, ErrorDefinition, StructDefinition, EnumDefinition,
    EnumValue, UserDefinedValueTypeDefinition, UsingForDirective, ElementaryTypeName,
    UserDefinedTypeName, FunctionTypeName, Mapping, ArrayTypeName, IdentifierPath, Block,
    UncheckedBlock, ExpressionStatement, VariableDeclarationStatement, IfStatement, ForStatement,
    WhileStatement, DoWhileStatement, Return, EmitStatement, RevertStatement, TryStatement,
    TryCatchClause, InlineAssembly, PlaceholderStatement, BreakStatement, ContinueStatement,
    Assignment, BinaryOperation, UnaryOperation, FunctionCall, FunctionCallOptions, MemberAccess,
    IndexAccess, IndexRangeAccess, Identifier, Literal, Conditional, TupleExpression, NewExpression,
    ElementaryTypeNameExpression,
);

// ===========================================================================
// Unified node handle (for the id->node map + ancestor walks)
// ===========================================================================

/// A clone of any AST node, keyed by id in `WorkspaceContext::nodes`.
/// Used for ancestor lookups and id->node resolution by detectors.
#[derive(Clone, Debug)]
pub enum AstNode {
    SourceUnit(SourceUnit),
    PragmaDirective(PragmaDirective),
    ImportDirective(ImportDirective),
    ContractDefinition(ContractDefinition),
    FunctionDefinition(FunctionDefinition),
    ModifierDefinition(ModifierDefinition),
    ModifierInvocation(ModifierInvocation),
    VariableDeclaration(VariableDeclaration),
    EventDefinition(EventDefinition),
    ErrorDefinition(ErrorDefinition),
    StructDefinition(StructDefinition),
    EnumDefinition(EnumDefinition),
    EnumValue(EnumValue),
    UserDefinedValueTypeDefinition(UserDefinedValueTypeDefinition),
    UsingForDirective(UsingForDirective),
    InheritanceSpecifier(InheritanceSpecifier),
    Block(Block),
    UncheckedBlock(UncheckedBlock),
    ExpressionStatement(ExpressionStatement),
    VariableDeclarationStatement(VariableDeclarationStatement),
    IfStatement(IfStatement),
    ForStatement(ForStatement),
    WhileStatement(WhileStatement),
    DoWhileStatement(DoWhileStatement),
    Return(Return),
    EmitStatement(EmitStatement),
    RevertStatement(RevertStatement),
    TryStatement(TryStatement),
    UncheckedBlockStmt,
    InlineAssembly(InlineAssembly),
    PlaceholderStatement(PlaceholderStatement),
    Assignment(Assignment),
    BinaryOperation(BinaryOperation),
    UnaryOperation(UnaryOperation),
    FunctionCall(FunctionCall),
    FunctionCallOptions(FunctionCallOptions),
    MemberAccess(MemberAccess),
    IndexAccess(IndexAccess),
    IndexRangeAccess(IndexRangeAccess),
    Identifier(Identifier),
    Literal(Literal),
    Conditional(Conditional),
    TupleExpression(TupleExpression),
    NewExpression(NewExpression),
    ElementaryTypeNameExpression(ElementaryTypeNameExpression),
}

impl AstNode {
    pub fn id(&self) -> NodeID {
        use AstNode::*;
        match self {
            SourceUnit(n) => n.id,
            PragmaDirective(n) => n.id,
            ImportDirective(n) => n.id,
            ContractDefinition(n) => n.id,
            FunctionDefinition(n) => n.id,
            ModifierDefinition(n) => n.id,
            ModifierInvocation(n) => n.id,
            VariableDeclaration(n) => n.id,
            EventDefinition(n) => n.id,
            ErrorDefinition(n) => n.id,
            StructDefinition(n) => n.id,
            EnumDefinition(n) => n.id,
            EnumValue(n) => n.id,
            UserDefinedValueTypeDefinition(n) => n.id,
            UsingForDirective(n) => n.id,
            InheritanceSpecifier(n) => n.id,
            Block(n) => n.id,
            UncheckedBlock(n) => n.id,
            ExpressionStatement(n) => n.id,
            VariableDeclarationStatement(n) => n.id,
            IfStatement(n) => n.id,
            ForStatement(n) => n.id,
            WhileStatement(n) => n.id,
            DoWhileStatement(n) => n.id,
            Return(n) => n.id,
            EmitStatement(n) => n.id,
            RevertStatement(n) => n.id,
            TryStatement(n) => n.id,
            UncheckedBlockStmt => 0,
            InlineAssembly(n) => n.id,
            PlaceholderStatement(n) => n.id,
            Assignment(n) => n.id,
            BinaryOperation(n) => n.id,
            UnaryOperation(n) => n.id,
            FunctionCall(n) => n.id,
            FunctionCallOptions(n) => n.id,
            MemberAccess(n) => n.id,
            IndexAccess(n) => n.id,
            IndexRangeAccess(n) => n.id,
            Identifier(n) => n.id,
            Literal(n) => n.id,
            Conditional(n) => n.id,
            TupleExpression(n) => n.id,
            NewExpression(n) => n.id,
            ElementaryTypeNameExpression(n) => n.id,
        }
    }

    pub fn src(&self) -> &str {
        use AstNode::*;
        match self {
            SourceUnit(n) => &n.src,
            PragmaDirective(n) => &n.src,
            ImportDirective(n) => &n.src,
            ContractDefinition(n) => &n.src,
            FunctionDefinition(n) => &n.src,
            ModifierDefinition(n) => &n.src,
            ModifierInvocation(n) => &n.src,
            VariableDeclaration(n) => &n.src,
            EventDefinition(n) => &n.src,
            ErrorDefinition(n) => &n.src,
            StructDefinition(n) => &n.src,
            EnumDefinition(n) => &n.src,
            EnumValue(n) => &n.src,
            UserDefinedValueTypeDefinition(n) => &n.src,
            UsingForDirective(n) => &n.src,
            InheritanceSpecifier(n) => &n.src,
            Block(n) => &n.src,
            UncheckedBlock(n) => &n.src,
            ExpressionStatement(n) => &n.src,
            VariableDeclarationStatement(n) => &n.src,
            IfStatement(n) => &n.src,
            ForStatement(n) => &n.src,
            WhileStatement(n) => &n.src,
            DoWhileStatement(n) => &n.src,
            Return(n) => &n.src,
            EmitStatement(n) => &n.src,
            RevertStatement(n) => &n.src,
            TryStatement(n) => &n.src,
            UncheckedBlockStmt => "",
            InlineAssembly(n) => &n.src,
            PlaceholderStatement(n) => &n.src,
            Assignment(n) => &n.src,
            BinaryOperation(n) => &n.src,
            UnaryOperation(n) => &n.src,
            FunctionCall(n) => &n.src,
            FunctionCallOptions(n) => &n.src,
            MemberAccess(n) => &n.src,
            IndexAccess(n) => &n.src,
            IndexRangeAccess(n) => &n.src,
            Identifier(n) => &n.src,
            Literal(n) => &n.src,
            Conditional(n) => &n.src,
            TupleExpression(n) => &n.src,
            NewExpression(n) => &n.src,
            ElementaryTypeNameExpression(n) => &n.src,
        }
    }

    /// The solc `nodeType` string for this node.
    pub fn node_type(&self) -> &'static str {
        use AstNode::*;
        match self {
            SourceUnit(_) => "SourceUnit",
            PragmaDirective(_) => "PragmaDirective",
            ImportDirective(_) => "ImportDirective",
            ContractDefinition(_) => "ContractDefinition",
            FunctionDefinition(_) => "FunctionDefinition",
            ModifierDefinition(_) => "ModifierDefinition",
            ModifierInvocation(_) => "ModifierInvocation",
            VariableDeclaration(_) => "VariableDeclaration",
            EventDefinition(_) => "EventDefinition",
            ErrorDefinition(_) => "ErrorDefinition",
            StructDefinition(_) => "StructDefinition",
            EnumDefinition(_) => "EnumDefinition",
            EnumValue(_) => "EnumValue",
            UserDefinedValueTypeDefinition(_) => "UserDefinedValueTypeDefinition",
            UsingForDirective(_) => "UsingForDirective",
            InheritanceSpecifier(_) => "InheritanceSpecifier",
            Block(_) => "Block",
            UncheckedBlock(_) => "UncheckedBlock",
            ExpressionStatement(_) => "ExpressionStatement",
            VariableDeclarationStatement(_) => "VariableDeclarationStatement",
            IfStatement(_) => "IfStatement",
            ForStatement(_) => "ForStatement",
            WhileStatement(_) => "WhileStatement",
            DoWhileStatement(_) => "DoWhileStatement",
            Return(_) => "Return",
            EmitStatement(_) => "EmitStatement",
            RevertStatement(_) => "RevertStatement",
            TryStatement(_) => "TryStatement",
            UncheckedBlockStmt => "UncheckedBlock",
            InlineAssembly(_) => "InlineAssembly",
            PlaceholderStatement(_) => "PlaceholderStatement",
            Assignment(_) => "Assignment",
            BinaryOperation(_) => "BinaryOperation",
            UnaryOperation(_) => "UnaryOperation",
            FunctionCall(_) => "FunctionCall",
            FunctionCallOptions(_) => "FunctionCallOptions",
            MemberAccess(_) => "MemberAccess",
            IndexAccess(_) => "IndexAccess",
            IndexRangeAccess(_) => "IndexRangeAccess",
            Identifier(_) => "Identifier",
            Literal(_) => "Literal",
            Conditional(_) => "Conditional",
            TupleExpression(_) => "TupleExpression",
            NewExpression(_) => "NewExpression",
            ElementaryTypeNameExpression(_) => "ElementaryTypeNameExpression",
        }
    }
}
