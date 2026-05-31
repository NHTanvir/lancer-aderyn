//! `WorkspaceContext` — the indexed, read-only view of a parsed Solidity
//! workspace that every detector queries.
//!
//! Building the context walks each `SourceUnit` once, cloning every node into a
//! typed index vector (giving detectors O(1) category lookup), plus an
//! `id -> node` map and `child -> parent` link map (for ancestor walks).

use std::collections::HashMap;

use crate::ast::*;

/// One source file plus its compiled content + solc source index.
#[derive(Clone, Debug, Default)]
pub struct SourceFile {
    /// Relative, report-facing path (e.g. `src/Token.sol`).
    pub path: String,
    /// Absolute path on disk, if known.
    pub absolute_path: String,
    /// Full UTF-8 source text (used for Peek / line resolution).
    pub content: String,
    /// solc source index — matches the trailing field of `src` strings.
    pub source_index: i64,
}

/// Capture key: `(file, line, snippet)`.
pub type CaptureKey = (String, usize, String);

#[derive(Default)]
pub struct WorkspaceContext {
    pub files: Vec<SourceFile>,
    source_index_to_file: HashMap<i64, usize>,

    pub source_units: Vec<SourceUnit>,
    pub nodes: HashMap<NodeID, AstNode>,
    pub parent_link: HashMap<NodeID, NodeID>,

    // Typed indexes (clones). Field name == accessor name (see `accessors!`).
    pub contract_definitions: Vec<ContractDefinition>,
    pub function_definitions: Vec<FunctionDefinition>,
    pub modifier_definitions: Vec<ModifierDefinition>,
    pub modifier_invocations: Vec<ModifierInvocation>,
    pub variable_declarations: Vec<VariableDeclaration>,
    pub event_definitions: Vec<EventDefinition>,
    pub error_definitions: Vec<ErrorDefinition>,
    pub struct_definitions: Vec<StructDefinition>,
    pub enum_definitions: Vec<EnumDefinition>,
    pub using_for_directives: Vec<UsingForDirective>,
    pub user_defined_value_type_definitions: Vec<UserDefinedValueTypeDefinition>,
    pub pragma_directives: Vec<PragmaDirective>,
    pub import_directives: Vec<ImportDirective>,
    pub inheritance_specifiers: Vec<InheritanceSpecifier>,

    pub blocks: Vec<Block>,
    pub unchecked_blocks: Vec<UncheckedBlock>,
    pub expression_statements: Vec<ExpressionStatement>,
    pub variable_declaration_statements: Vec<VariableDeclarationStatement>,
    pub if_statements: Vec<IfStatement>,
    pub for_statements: Vec<ForStatement>,
    pub while_statements: Vec<WhileStatement>,
    pub do_while_statements: Vec<DoWhileStatement>,
    pub return_statements: Vec<Return>,
    pub emit_statements: Vec<EmitStatement>,
    pub revert_statements: Vec<RevertStatement>,
    pub try_statements: Vec<TryStatement>,
    pub inline_assemblies: Vec<InlineAssembly>,
    pub placeholder_statements: Vec<PlaceholderStatement>,

    pub assignments: Vec<Assignment>,
    pub binary_operations: Vec<BinaryOperation>,
    pub unary_operations: Vec<UnaryOperation>,
    pub function_calls: Vec<FunctionCall>,
    pub function_call_options: Vec<FunctionCallOptions>,
    pub member_accesses: Vec<MemberAccess>,
    pub index_accesses: Vec<IndexAccess>,
    pub index_range_accesses: Vec<IndexRangeAccess>,
    pub identifiers: Vec<Identifier>,
    pub literals: Vec<Literal>,
    pub conditionals: Vec<Conditional>,
    pub tuple_expressions: Vec<TupleExpression>,
    pub new_expressions: Vec<NewExpression>,
    pub elementary_type_name_expressions: Vec<ElementaryTypeNameExpression>,
}

macro_rules! accessors {
    ($($name:ident : $t:ty),+ $(,)?) => {
        impl WorkspaceContext {
            $(#[doc = concat!("All `", stringify!($t), "` nodes in the workspace.")]
              pub fn $name(&self) -> &[$t] { &self.$name })+
        }
    };
}

accessors!(
    contract_definitions: ContractDefinition,
    function_definitions: FunctionDefinition,
    modifier_definitions: ModifierDefinition,
    modifier_invocations: ModifierInvocation,
    variable_declarations: VariableDeclaration,
    event_definitions: EventDefinition,
    error_definitions: ErrorDefinition,
    struct_definitions: StructDefinition,
    enum_definitions: EnumDefinition,
    using_for_directives: UsingForDirective,
    user_defined_value_type_definitions: UserDefinedValueTypeDefinition,
    pragma_directives: PragmaDirective,
    import_directives: ImportDirective,
    inheritance_specifiers: InheritanceSpecifier,
    blocks: Block,
    unchecked_blocks: UncheckedBlock,
    expression_statements: ExpressionStatement,
    variable_declaration_statements: VariableDeclarationStatement,
    if_statements: IfStatement,
    for_statements: ForStatement,
    while_statements: WhileStatement,
    do_while_statements: DoWhileStatement,
    return_statements: Return,
    emit_statements: EmitStatement,
    revert_statements: RevertStatement,
    try_statements: TryStatement,
    inline_assemblies: InlineAssembly,
    placeholder_statements: PlaceholderStatement,
    assignments: Assignment,
    binary_operations: BinaryOperation,
    unary_operations: UnaryOperation,
    function_calls: FunctionCall,
    function_call_options: FunctionCallOptions,
    member_accesses: MemberAccess,
    index_accesses: IndexAccess,
    index_range_accesses: IndexRangeAccess,
    identifiers: Identifier,
    literals: Literal,
    conditionals: Conditional,
    tuple_expressions: TupleExpression,
    new_expressions: NewExpression,
    elementary_type_name_expressions: ElementaryTypeNameExpression,
);

impl WorkspaceContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// All source unit trees.
    pub fn source_units(&self) -> &[SourceUnit] {
        &self.source_units
    }

    /// Register one parsed source file + its AST. Call `build()` afterwards.
    pub fn add_source_unit(&mut self, file: SourceFile, unit: SourceUnit) {
        let idx = self.files.len();
        self.source_index_to_file.insert(file.source_index, idx);
        self.files.push(file);
        self.source_units.push(unit);
    }

    /// Walk all source units, populating every index, the node map, and parent
    /// links. Idempotent-ish: call once after all `add_source_unit` calls.
    pub fn build(&mut self) {
        let units = std::mem::take(&mut self.source_units);
        for su in &units {
            self.register_source_unit(su);
        }
        self.source_units = units;
    }

    // ---- lookups -----------------------------------------------------------

    pub fn get_node(&self, id: NodeID) -> Option<&AstNode> {
        self.nodes.get(&id)
    }

    pub fn get_parent(&self, id: NodeID) -> Option<&AstNode> {
        let pid = *self.parent_link.get(&id)?;
        self.nodes.get(&pid)
    }

    /// Walk parent links until a node of `node_type` is found.
    pub fn closest_ancestor_of_type(&self, id: NodeID, node_type: &str) -> Option<&AstNode> {
        let mut current = *self.parent_link.get(&id)?;
        loop {
            let node = self.nodes.get(&current)?;
            if node.node_type() == node_type {
                return Some(node);
            }
            current = *self.parent_link.get(&current)?;
        }
    }

    /// The `ContractDefinition` enclosing a node, if any.
    pub fn enclosing_contract(&self, id: NodeID) -> Option<&ContractDefinition> {
        match self.closest_ancestor_of_type(id, "ContractDefinition")? {
            AstNode::ContractDefinition(c) => Some(c),
            _ => None,
        }
    }

    /// The `FunctionDefinition` enclosing a node, if any.
    pub fn enclosing_function(&self, id: NodeID) -> Option<&FunctionDefinition> {
        match self.closest_ancestor_of_type(id, "FunctionDefinition")? {
            AstNode::FunctionDefinition(f) => Some(f),
            _ => None,
        }
    }

    // ---- source / location -------------------------------------------------

    pub fn source_file_for_src(&self, src: &str) -> Option<&SourceFile> {
        let loc = SrcLocation::parse(src)?;
        let idx = *self.source_index_to_file.get(&loc.source_index)?;
        self.files.get(idx)
    }

    /// Extract the source text referenced by `src` (the "Peek" operation).
    pub fn peek(&self, src: &str) -> Option<String> {
        let file = self.source_file_for_src(src)?;
        slice_for_src(&file.content, src).map(|s| s.to_string())
    }

    /// Compute the `(file, line, snippet)` capture key for a node's `src`.
    pub fn capture_key(&self, src: &str) -> Option<CaptureKey> {
        let file = self.source_file_for_src(src)?;
        let loc = SrcLocation::parse(src)?;
        let line = line_for_offset(&file.content, loc.byte_offset);
        let snippet = slice_for_src(&file.content, src)
            .map(|s| s.lines().next().unwrap_or(s).trim().to_string())
            .unwrap_or_default();
        Some((file.path.clone(), line, snippet))
    }

    // ---- registration walk -------------------------------------------------

    fn record(&mut self, node: AstNode, parent: Option<NodeID>) {
        let id = node.id();
        if let Some(p) = parent {
            self.parent_link.insert(id, p);
        }
        self.nodes.insert(id, node);
    }

    fn register_source_unit(&mut self, su: &SourceUnit) {
        let parent = Some(su.id);
        self.record(AstNode::SourceUnit(su.clone()), None);
        for node in &su.nodes {
            match node {
                SourceUnitNode::PragmaDirective(n) => {
                    self.pragma_directives.push(n.clone());
                    self.record(AstNode::PragmaDirective(n.clone()), parent);
                }
                SourceUnitNode::ImportDirective(n) => {
                    self.import_directives.push(n.clone());
                    self.record(AstNode::ImportDirective(n.clone()), parent);
                }
                SourceUnitNode::ContractDefinition(n) => self.register_contract(n, parent),
                SourceUnitNode::StructDefinition(n) => self.register_struct(n, parent),
                SourceUnitNode::EnumDefinition(n) => self.register_enum(n, parent),
                SourceUnitNode::ErrorDefinition(n) => self.register_error(n, parent),
                SourceUnitNode::FunctionDefinition(n) => self.register_function(n, parent),
                SourceUnitNode::VariableDeclaration(n) => self.register_variable(n, parent),
                SourceUnitNode::UserDefinedValueTypeDefinition(n) => self.register_udvt(n, parent),
                SourceUnitNode::UsingForDirective(n) => self.register_using_for(n, parent),
                SourceUnitNode::Unsupported => {}
            }
        }
    }

    fn register_contract(&mut self, c: &ContractDefinition, parent: Option<NodeID>) {
        self.contract_definitions.push(c.clone());
        self.record(AstNode::ContractDefinition(c.clone()), parent);
        let p = Some(c.id);
        for spec in &c.base_contracts {
            self.inheritance_specifiers.push(spec.clone());
            self.record(AstNode::InheritanceSpecifier(spec.clone()), p);
            if let Some(args) = &spec.arguments {
                for a in args {
                    self.register_expression(a, Some(spec.id));
                }
            }
        }
        for node in &c.nodes {
            match node {
                ContractBodyElement::FunctionDefinition(n) => self.register_function(n, p),
                ContractBodyElement::VariableDeclaration(n) => self.register_variable(n, p),
                ContractBodyElement::ModifierDefinition(n) => self.register_modifier(n, p),
                ContractBodyElement::EventDefinition(n) => self.register_event(n, p),
                ContractBodyElement::ErrorDefinition(n) => self.register_error(n, p),
                ContractBodyElement::StructDefinition(n) => self.register_struct(n, p),
                ContractBodyElement::EnumDefinition(n) => self.register_enum(n, p),
                ContractBodyElement::UsingForDirective(n) => self.register_using_for(n, p),
                ContractBodyElement::UserDefinedValueTypeDefinition(n) => self.register_udvt(n, p),
                ContractBodyElement::Unsupported => {}
            }
        }
    }

    fn register_function(&mut self, f: &FunctionDefinition, parent: Option<NodeID>) {
        self.function_definitions.push(f.clone());
        self.record(AstNode::FunctionDefinition(f.clone()), parent);
        let p = Some(f.id);
        self.register_parameter_list(&f.parameters, p);
        self.register_parameter_list(&f.return_parameters, p);
        for m in &f.modifiers {
            self.modifier_invocations.push(m.clone());
            self.record(AstNode::ModifierInvocation(m.clone()), p);
            if let Some(args) = &m.arguments {
                for a in args {
                    self.register_expression(a, Some(m.id));
                }
            }
        }
        if let Some(body) = &f.body {
            self.register_block(body, p);
        }
    }

    fn register_modifier(&mut self, m: &ModifierDefinition, parent: Option<NodeID>) {
        self.modifier_definitions.push(m.clone());
        self.record(AstNode::ModifierDefinition(m.clone()), parent);
        let p = Some(m.id);
        self.register_parameter_list(&m.parameters, p);
        if let Some(body) = &m.body {
            self.register_block(body, p);
        }
    }

    fn register_parameter_list(&mut self, pl: &ParameterList, parent: Option<NodeID>) {
        for v in &pl.parameters {
            self.register_variable(v, parent);
        }
    }

    fn register_variable(&mut self, v: &VariableDeclaration, parent: Option<NodeID>) {
        self.variable_declarations.push(v.clone());
        self.record(AstNode::VariableDeclaration(v.clone()), parent);
        let p = Some(v.id);
        if let Some(tn) = &v.type_name {
            self.register_type_name(tn, p);
        }
        if let Some(val) = &v.value {
            self.register_expression(val, p);
        }
    }

    fn register_event(&mut self, e: &EventDefinition, parent: Option<NodeID>) {
        self.event_definitions.push(e.clone());
        self.record(AstNode::EventDefinition(e.clone()), parent);
        self.register_parameter_list(&e.parameters, Some(e.id));
    }

    fn register_error(&mut self, e: &ErrorDefinition, parent: Option<NodeID>) {
        self.error_definitions.push(e.clone());
        self.record(AstNode::ErrorDefinition(e.clone()), parent);
        self.register_parameter_list(&e.parameters, Some(e.id));
    }

    fn register_struct(&mut self, s: &StructDefinition, parent: Option<NodeID>) {
        self.struct_definitions.push(s.clone());
        self.record(AstNode::StructDefinition(s.clone()), parent);
        for m in &s.members {
            self.register_variable(m, Some(s.id));
        }
    }

    fn register_enum(&mut self, e: &EnumDefinition, parent: Option<NodeID>) {
        self.enum_definitions.push(e.clone());
        self.record(AstNode::EnumDefinition(e.clone()), parent);
        for v in &e.members {
            self.record(AstNode::EnumValue(v.clone()), Some(e.id));
        }
    }

    fn register_udvt(&mut self, u: &UserDefinedValueTypeDefinition, parent: Option<NodeID>) {
        self.user_defined_value_type_definitions.push(u.clone());
        self.record(AstNode::UserDefinedValueTypeDefinition(u.clone()), parent);
    }

    fn register_using_for(&mut self, u: &UsingForDirective, parent: Option<NodeID>) {
        self.using_for_directives.push(u.clone());
        self.record(AstNode::UsingForDirective(u.clone()), parent);
    }

    fn register_type_name(&mut self, tn: &TypeName, parent: Option<NodeID>) {
        match tn {
            TypeName::ArrayTypeName(a) => {
                if let Some(bt) = &a.base_type {
                    self.register_type_name(bt, parent);
                }
                if let Some(len) = &a.length {
                    self.register_expression(len, parent);
                }
            }
            TypeName::Mapping(m) => {
                if let Some(k) = &m.key_type {
                    self.register_type_name(k, parent);
                }
                if let Some(v) = &m.value_type {
                    self.register_type_name(v, parent);
                }
            }
            _ => {}
        }
    }

    fn register_block(&mut self, b: &Block, parent: Option<NodeID>) {
        self.blocks.push(b.clone());
        self.record(AstNode::Block(b.clone()), parent);
        for s in &b.statements {
            self.register_statement(s, Some(b.id));
        }
    }

    fn register_statement(&mut self, s: &Statement, parent: Option<NodeID>) {
        match s {
            Statement::Block(b) => self.register_block(b, parent),
            Statement::UncheckedBlock(u) => {
                self.unchecked_blocks.push(u.clone());
                self.record(AstNode::UncheckedBlock(u.clone()), parent);
                for st in &u.statements {
                    self.register_statement(st, Some(u.id));
                }
            }
            Statement::ExpressionStatement(e) => {
                self.expression_statements.push(e.clone());
                self.record(AstNode::ExpressionStatement(e.clone()), parent);
                self.register_expression(&e.expression, Some(e.id));
            }
            Statement::VariableDeclarationStatement(v) => {
                self.variable_declaration_statements.push(v.clone());
                self.record(AstNode::VariableDeclarationStatement(v.clone()), parent);
                for d in v.declarations.iter().flatten() {
                    self.register_variable(d, Some(v.id));
                }
                if let Some(init) = &v.initial_value {
                    self.register_expression(init, Some(v.id));
                }
            }
            Statement::IfStatement(i) => {
                self.if_statements.push(i.clone());
                self.record(AstNode::IfStatement(i.clone()), parent);
                self.register_expression(&i.condition, Some(i.id));
                self.register_statement(&i.true_body, Some(i.id));
                if let Some(fb) = &i.false_body {
                    self.register_statement(fb, Some(i.id));
                }
            }
            Statement::ForStatement(f) => {
                self.for_statements.push(f.clone());
                self.record(AstNode::ForStatement(f.clone()), parent);
                if let Some(init) = &f.initialization_expression {
                    self.register_statement(init, Some(f.id));
                }
                if let Some(cond) = &f.condition {
                    self.register_expression(cond, Some(f.id));
                }
                if let Some(le) = &f.loop_expression {
                    self.register_statement(le, Some(f.id));
                }
                self.register_statement(&f.body, Some(f.id));
            }
            Statement::WhileStatement(w) => {
                self.while_statements.push(w.clone());
                self.record(AstNode::WhileStatement(w.clone()), parent);
                self.register_expression(&w.condition, Some(w.id));
                self.register_statement(&w.body, Some(w.id));
            }
            Statement::DoWhileStatement(d) => {
                self.do_while_statements.push(d.clone());
                self.record(AstNode::DoWhileStatement(d.clone()), parent);
                self.register_expression(&d.condition, Some(d.id));
                self.register_statement(&d.body, Some(d.id));
            }
            Statement::Return(r) => {
                self.return_statements.push(r.clone());
                self.record(AstNode::Return(r.clone()), parent);
                if let Some(e) = &r.expression {
                    self.register_expression(e, Some(r.id));
                }
            }
            Statement::EmitStatement(e) => {
                self.emit_statements.push(e.clone());
                self.record(AstNode::EmitStatement(e.clone()), parent);
                self.register_expression(
                    &Expression::FunctionCall(e.event_call.clone()),
                    Some(e.id),
                );
            }
            Statement::RevertStatement(r) => {
                self.revert_statements.push(r.clone());
                self.record(AstNode::RevertStatement(r.clone()), parent);
                self.register_expression(
                    &Expression::FunctionCall(r.error_call.clone()),
                    Some(r.id),
                );
            }
            Statement::TryStatement(t) => {
                self.try_statements.push(t.clone());
                self.record(AstNode::TryStatement(t.clone()), parent);
                self.register_expression(
                    &Expression::FunctionCall(t.external_call.clone()),
                    Some(t.id),
                );
                for clause in &t.clauses {
                    if let Some(pl) = &clause.parameters {
                        self.register_parameter_list(pl, Some(t.id));
                    }
                    self.register_block(&clause.block, Some(t.id));
                }
            }
            Statement::InlineAssembly(a) => {
                self.inline_assemblies.push(a.clone());
                self.record(AstNode::InlineAssembly(a.clone()), parent);
            }
            Statement::PlaceholderStatement(p) => {
                self.placeholder_statements.push(p.clone());
                self.record(AstNode::PlaceholderStatement(p.clone()), parent);
            }
            Statement::Break(_) | Statement::Continue(_) | Statement::Unsupported => {}
        }
    }

    fn register_expression(&mut self, e: &Expression, parent: Option<NodeID>) {
        match e {
            Expression::Assignment(a) => {
                self.assignments.push(a.clone());
                self.record(AstNode::Assignment(a.clone()), parent);
                self.register_expression(&a.left_hand_side, Some(a.id));
                self.register_expression(&a.right_hand_side, Some(a.id));
            }
            Expression::BinaryOperation(b) => {
                self.binary_operations.push(b.clone());
                self.record(AstNode::BinaryOperation(b.clone()), parent);
                self.register_expression(&b.left_expression, Some(b.id));
                self.register_expression(&b.right_expression, Some(b.id));
            }
            Expression::UnaryOperation(u) => {
                self.unary_operations.push(u.clone());
                self.record(AstNode::UnaryOperation(u.clone()), parent);
                self.register_expression(&u.sub_expression, Some(u.id));
            }
            Expression::FunctionCall(f) => {
                self.function_calls.push(f.clone());
                self.record(AstNode::FunctionCall(f.clone()), parent);
                self.register_expression(&f.expression, Some(f.id));
                for a in &f.arguments {
                    self.register_expression(a, Some(f.id));
                }
            }
            Expression::FunctionCallOptions(f) => {
                self.function_call_options.push(f.clone());
                self.record(AstNode::FunctionCallOptions(f.clone()), parent);
                self.register_expression(&f.expression, Some(f.id));
                for o in &f.options {
                    self.register_expression(o, Some(f.id));
                }
            }
            Expression::MemberAccess(m) => {
                self.member_accesses.push(m.clone());
                self.record(AstNode::MemberAccess(m.clone()), parent);
                self.register_expression(&m.expression, Some(m.id));
            }
            Expression::IndexAccess(i) => {
                self.index_accesses.push(i.clone());
                self.record(AstNode::IndexAccess(i.clone()), parent);
                self.register_expression(&i.base_expression, Some(i.id));
                if let Some(ie) = &i.index_expression {
                    self.register_expression(ie, Some(i.id));
                }
            }
            Expression::IndexRangeAccess(i) => {
                self.index_range_accesses.push(i.clone());
                self.record(AstNode::IndexRangeAccess(i.clone()), parent);
                self.register_expression(&i.base_expression, Some(i.id));
                if let Some(s) = &i.start_expression {
                    self.register_expression(s, Some(i.id));
                }
                if let Some(en) = &i.end_expression {
                    self.register_expression(en, Some(i.id));
                }
            }
            Expression::Identifier(id) => {
                self.identifiers.push(id.clone());
                self.record(AstNode::Identifier(id.clone()), parent);
            }
            Expression::Literal(l) => {
                self.literals.push(l.clone());
                self.record(AstNode::Literal(l.clone()), parent);
            }
            Expression::Conditional(c) => {
                self.conditionals.push(c.clone());
                self.record(AstNode::Conditional(c.clone()), parent);
                self.register_expression(&c.condition, Some(c.id));
                self.register_expression(&c.true_expression, Some(c.id));
                self.register_expression(&c.false_expression, Some(c.id));
            }
            Expression::TupleExpression(t) => {
                self.tuple_expressions.push(t.clone());
                self.record(AstNode::TupleExpression(t.clone()), parent);
                for c in t.components.iter().flatten() {
                    self.register_expression(c, Some(t.id));
                }
            }
            Expression::NewExpression(n) => {
                self.new_expressions.push(n.clone());
                self.record(AstNode::NewExpression(n.clone()), parent);
                if let Some(tn) = &n.type_name {
                    self.register_type_name(tn, Some(n.id));
                }
            }
            Expression::ElementaryTypeNameExpression(et) => {
                self.elementary_type_name_expressions.push(et.clone());
                self.record(AstNode::ElementaryTypeNameExpression(et.clone()), parent);
            }
            Expression::Unsupported => {}
        }
    }
}
