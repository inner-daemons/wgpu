/*!
Replace variable initializers with statements.

After this pass, no [`LocalVariable`] or [`Private`] [`GlobalVariable`] has an
`init`. A variable that had an initializer, or that relied on implicit zero
initialization, is instead assigned by a [`Store`] at the start of its
function, or, for a private global, at the start of each entry point that uses
it.

When [`Options::workgroup`] is set, compute, task and mesh entry points also
zero the [`WorkGroup`] variables (and, in task shaders, the [`TaskPayload`])
they use, followed by a workgroup barrier. Arrays are flattened, and their
elements are spread over the invocations of the workgroup, so that no single
invocation writes a large aggregate.

[`LocalVariable`]: crate::LocalVariable
[`GlobalVariable`]: crate::GlobalVariable
[`Private`]: crate::AddressSpace::Private
[`WorkGroup`]: crate::AddressSpace::WorkGroup
[`TaskPayload`]: crate::AddressSpace::TaskPayload
[`Store`]: crate::Statement::Store
*/

use alloc::{vec, vec::Vec};

use crate::{
    arena::{Arena, Handle, Range, UniqueArena},
    proc::{Emitter, IndexableLength},
    valid::{FunctionInfo, ModuleInfo, TypeFlags},
    AddressSpace, BinaryOperator, Binding, Block, BuiltIn, Expression, FastHashMap, Function,
    FunctionArgument, GlobalVariable, Literal, LocalVariable, Scalar, ShaderStage, Span, Statement,
    Type, TypeInner,
};

/// Options for the zero initialization pass.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Options {
    /// Zero the workgroup memory used by compute, task and mesh entry points.
    pub workgroup: bool,
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum ZeroInitError {
    #[error("variable initializer is not a fully evaluated constant expression")]
    UnevaluatedInitializer,
    #[error("workgroup variable array size is not a resolved constant")]
    UnresolvedArraySize,
    #[error("workgroup variable type can't be zero initialized")]
    UnsupportedType,
    #[error("workgroup invocation count overflows u32")]
    WorkgroupSizeOverflow,
    #[error("workgroup variable element count overflows u32")]
    ElementCountOverflow,
}

/// Run the pass on `module`. Return `true` if `module` was changed.
///
/// `info` must be the validation result of `module` before the pass.
pub(crate) fn run(
    module: &mut crate::Module,
    info: &ModuleInfo,
    options: &Options,
) -> Result<bool, ZeroInitError> {
    let array_lengths = module
        .types
        .iter()
        .filter_map(|(handle, ty)| match ty.inner {
            TypeInner::Array { size, .. } => match size.resolve(module.to_ctx()) {
                Ok(IndexableLength::Known(length)) => Some((handle, length)),
                _ => None,
            },
            _ => None,
        })
        .collect::<FastHashMap<_, _>>();

    let crate::Module {
        ref mut types,
        ref mut global_variables,
        ref global_expressions,
        ref mut functions,
        ref mut entry_points,
        ..
    } = *module;

    let mut changed = false;
    for (_, function) in functions.iter_mut() {
        let mut prologue = Block::new();
        lower_locals(function, types, info, &mut prologue)?;
        changed |= prepend(function, prologue);
    }

    for (index, ep) in entry_points.iter_mut().enumerate() {
        let ep_info = info.get_entry_point(index);
        let mut prologue = Block::new();
        lower_locals(&mut ep.function, types, info, &mut prologue)?;
        lower_private_globals(
            &mut ep.function,
            types,
            info,
            ep_info,
            global_variables,
            global_expressions,
            &mut prologue,
        )?;
        if options.workgroup && ep.stage.compute_like() {
            let mut wg = WorkgroupInit {
                types,
                array_lengths: &array_lengths,
                global_variables,
                info,
                ep_info,
                stage: ep.stage,
                workgroup_size: ep.workgroup_size,
                function: &mut ep.function,
                emitter: Emitter::default(),
                block: &mut prologue,
                local_invocation_index: None,
                counter: None,
            };
            wg.run()?;
        }
        changed |= prepend(&mut ep.function, prologue);
    }

    for (_, var) in global_variables.iter_mut() {
        if var.space == AddressSpace::Private && var.init.take().is_some() {
            changed = true;
        }
    }

    Ok(changed)
}

fn prepend(function: &mut Function, mut prologue: Block) -> bool {
    if prologue.is_empty() {
        return false;
    }
    prologue.append(&mut function.body);
    function.body = prologue;
    true
}

/// Whether `ty` can be the type of a [`ZeroValue`](Expression::ZeroValue).
fn zeroable(types: &UniqueArena<Type>, info: &ModuleInfo, ty: Handle<Type>) -> bool {
    info[ty].contains(TypeFlags::CONSTRUCTIBLE)
        && !matches!(
            types[ty].inner,
            TypeInner::RayQuery { .. } | TypeInner::HitObject
        )
}

/// The zero value of `ty`, as a literal if it is a scalar.
fn zero_value(types: &UniqueArena<Type>, ty: Handle<Type>) -> Expression {
    match types[ty].inner {
        TypeInner::Scalar(scalar) => Literal::zero(scalar)
            .map(Expression::Literal)
            .unwrap_or(Expression::ZeroValue(ty)),
        _ => Expression::ZeroValue(ty),
    }
}

fn lower_locals(
    function: &mut Function,
    types: &UniqueArena<Type>,
    info: &ModuleInfo,
    prologue: &mut Block,
) -> Result<(), ZeroInitError> {
    let Function {
        ref mut local_variables,
        ref mut expressions,
        ..
    } = *function;
    for (handle, local, &span) in local_variables.iter_mut_span() {
        let init = local.init.take();
        if !zeroable(types, info, local.ty) {
            continue;
        }
        let pointer = expressions.append(Expression::LocalVariable(handle), span);
        let value = match init {
            None => expressions.append(zero_value(types, local.ty), span),
            Some(init) if expressions[init].needs_pre_emit() => init,
            Some(init) => copy_const_expression(init, None, expressions, prologue)?,
        };
        prologue.push(Statement::Store { pointer, value }, span);
    }
    Ok(())
}

fn lower_private_globals(
    function: &mut Function,
    types: &UniqueArena<Type>,
    info: &ModuleInfo,
    ep_info: &FunctionInfo,
    global_variables: &Arena<GlobalVariable>,
    global_expressions: &Arena<Expression>,
    prologue: &mut Block,
) -> Result<(), ZeroInitError> {
    let expressions = &mut function.expressions;
    for (handle, var) in global_variables.iter() {
        if var.space != AddressSpace::Private
            || ep_info[handle].is_empty()
            || !zeroable(types, info, var.ty)
        {
            continue;
        }
        let span = global_variables.get_span(handle);
        let pointer = expressions.append(Expression::GlobalVariable(handle), span);
        let value = match var.init {
            None => expressions.append(zero_value(types, var.ty), span),
            Some(init) => {
                copy_const_expression(init, Some(global_expressions), expressions, prologue)?
            }
        };
        prologue.push(Statement::Store { pointer, value }, span);
    }
    Ok(())
}

/// Copy the fully evaluated constant expression `expr` into `target`, emitting it in `block`.
///
/// `expr` is in `source`, or in `target` if `source` is `None`.
fn copy_const_expression(
    expr: Handle<Expression>,
    source: Option<&Arena<Expression>>,
    target: &mut Arena<Expression>,
    block: &mut Block,
) -> Result<Handle<Expression>, ZeroInitError> {
    let (expression, span) = {
        let arena = source.unwrap_or(target);
        (arena[expr].clone(), arena.get_span(expr))
    };
    let copy = match expression {
        Expression::Literal(_) | Expression::Constant(_) | Expression::ZeroValue(_) => {
            return Ok(target.append(expression, span));
        }
        Expression::Compose { ty, components } => {
            let components = components
                .into_iter()
                .map(|c| copy_const_expression(c, source, target, block))
                .collect::<Result<_, _>>()?;
            Expression::Compose { ty, components }
        }
        Expression::Splat { size, value } => Expression::Splat {
            size,
            value: copy_const_expression(value, source, target, block)?,
        },
        _ => return Err(ZeroInitError::UnevaluatedInitializer),
    };
    let handle = target.append(copy, span);
    block.push(
        Statement::Emit(Range::new_from_bounds(handle, handle)),
        Span::UNDEFINED,
    );
    Ok(handle)
}

/// One step from a variable to one of its parts.
#[derive(Clone, Copy)]
enum Step {
    Member(u32),
    /// Index into an array, by the position of the array's length in the dimension list.
    Element(usize),
}

/// A part of a workgroup variable that is zeroed by a single [`Store`].
///
/// [`Store`]: Statement::Store
struct Leaf {
    steps: Vec<Step>,
    /// The lengths of the arrays that contain this part, outermost first.
    dimensions: Vec<u32>,
    value: LeafValue,
}

enum LeafValue {
    Zero(Handle<Type>),
    Atomic(Scalar),
}

struct WorkgroupInit<'a> {
    types: &'a mut UniqueArena<Type>,
    array_lengths: &'a FastHashMap<Handle<Type>, u32>,
    global_variables: &'a Arena<GlobalVariable>,
    info: &'a ModuleInfo,
    ep_info: &'a FunctionInfo,
    stage: ShaderStage,
    workgroup_size: [u32; 3],
    function: &'a mut Function,
    emitter: Emitter,
    block: &'a mut Block,
    local_invocation_index: Option<Handle<Expression>>,
    counter: Option<Handle<LocalVariable>>,
}

impl WorkgroupInit<'_> {
    fn run(&mut self) -> Result<(), ZeroInitError> {
        let invocations = self
            .workgroup_size
            .iter()
            .try_fold(1u32, |total, &size| total.checked_mul(size))
            .ok_or(ZeroInitError::WorkgroupSizeOverflow)?;

        let mut any = false;
        let mut single = Vec::new();
        for (handle, var) in self.global_variables.iter() {
            let task_payload =
                var.space == AddressSpace::TaskPayload && self.stage == ShaderStage::Task;
            if self.ep_info[handle].is_empty()
                || !(var.space == AddressSpace::WorkGroup || task_payload)
            {
                continue;
            }
            any = true;
            let mut leaves = Vec::new();
            self.collect_leaves(var.ty, &mut Vec::new(), &mut Vec::new(), &mut leaves)?;
            for leaf in leaves {
                if leaf.dimensions.is_empty() {
                    single.push((handle, leaf));
                } else {
                    self.write_distributed(handle, &leaf, invocations)?;
                }
            }
        }

        if !single.is_empty() {
            let index = self.local_invocation_index();
            let zero = self.literal(Literal::U32(0));
            let condition = self.emit(Expression::Binary {
                op: BinaryOperator::Equal,
                left: index,
                right: zero,
            });
            let mut accept = Block::new();
            for (handle, leaf) in single {
                self.write_store(&mut accept, handle, &leaf, &[]);
            }
            self.push(Statement::If {
                condition,
                accept,
                reject: Block::new(),
            });
        }

        if any {
            self.push(Statement::ControlBarrier(crate::Barrier::WORK_GROUP));
        }
        Ok(())
    }

    fn collect_leaves(
        &self,
        ty: Handle<Type>,
        steps: &mut Vec<Step>,
        dimensions: &mut Vec<u32>,
        leaves: &mut Vec<Leaf>,
    ) -> Result<(), ZeroInitError> {
        if zeroable(self.types, self.info, ty) && !self.contains_array(ty) {
            leaves.push(Leaf {
                steps: steps.clone(),
                dimensions: dimensions.clone(),
                value: LeafValue::Zero(ty),
            });
            return Ok(());
        }
        match self.types[ty].inner {
            TypeInner::Atomic(scalar) => leaves.push(Leaf {
                steps: steps.clone(),
                dimensions: dimensions.clone(),
                value: LeafValue::Atomic(scalar),
            }),
            TypeInner::Array { base, .. } => {
                let &length = self
                    .array_lengths
                    .get(&ty)
                    .ok_or(ZeroInitError::UnresolvedArraySize)?;
                steps.push(Step::Element(dimensions.len()));
                dimensions.push(length);
                self.collect_leaves(base, steps, dimensions, leaves)?;
                dimensions.pop();
                steps.pop();
            }
            TypeInner::Struct { ref members, .. } => {
                for (index, member) in members.iter().enumerate() {
                    steps.push(Step::Member(index as u32));
                    self.collect_leaves(member.ty, steps, dimensions, leaves)?;
                    steps.pop();
                }
            }
            _ => return Err(ZeroInitError::UnsupportedType),
        }
        Ok(())
    }

    fn contains_array(&self, ty: Handle<Type>) -> bool {
        match self.types[ty].inner {
            TypeInner::Array { .. } => true,
            TypeInner::Struct { ref members, .. } => {
                members.iter().any(|member| self.contains_array(member.ty))
            }
            _ => false,
        }
    }

    /// Zero `leaf` of `var`, spreading its elements over the workgroup's invocations.
    fn write_distributed(
        &mut self,
        var: Handle<GlobalVariable>,
        leaf: &Leaf,
        invocations: u32,
    ) -> Result<(), ZeroInitError> {
        let count = leaf
            .dimensions
            .iter()
            .try_fold(1u32, |total, &length| total.checked_mul(length))
            .ok_or(ZeroInitError::ElementCountOverflow)?;
        let index = self.local_invocation_index();

        if count <= invocations {
            let mut body = Block::new();
            let indices = self.element_indices(&mut body, index, &leaf.dimensions);
            self.write_store(&mut body, var, leaf, &indices);
            if count == invocations {
                self.block.append(&mut body);
            } else {
                let count = self.literal(Literal::U32(count));
                let condition = self.emit(Expression::Binary {
                    op: BinaryOperator::Less,
                    left: index,
                    right: count,
                });
                self.push(Statement::If {
                    condition,
                    accept: body,
                    reject: Block::new(),
                });
            }
            return Ok(());
        }

        // Every invocation stores at least one element, since `index < invocations < count`.
        let counter = self.counter();
        let counter = self
            .function
            .expressions
            .append(Expression::LocalVariable(counter), Span::UNDEFINED);
        self.push(Statement::Store {
            pointer: counter,
            value: index,
        });

        let mut body = Block::new();
        let count = self.literal(Literal::U32(count));
        let stride = self.literal(Literal::U32(invocations));
        self.emitter.start(&self.function.expressions);
        let current = self.append(Expression::Load { pointer: counter });
        let done = self.append(Expression::Binary {
            op: BinaryOperator::GreaterEqual,
            left: current,
            right: count,
        });
        body.extend(self.emitter.finish(&self.function.expressions));
        body.push(
            Statement::If {
                condition: done,
                accept: Block::from_vec(vec![Statement::Break]),
                reject: Block::new(),
            },
            Span::UNDEFINED,
        );
        let indices = self.element_indices(&mut body, current, &leaf.dimensions);
        self.write_store(&mut body, var, leaf, &indices);
        self.emitter.start(&self.function.expressions);
        let next = self.append(Expression::Binary {
            op: BinaryOperator::Add,
            left: current,
            right: stride,
        });
        body.extend(self.emitter.finish(&self.function.expressions));
        body.push(
            Statement::Store {
                pointer: counter,
                value: next,
            },
            Span::UNDEFINED,
        );

        self.push(Statement::Loop {
            body,
            continuing: Block::new(),
            break_if: None,
        });
        Ok(())
    }

    /// Split the flat element index `flat` into one index per dimension.
    fn element_indices(
        &mut self,
        block: &mut Block,
        flat: Handle<Expression>,
        dimensions: &[u32],
    ) -> Vec<Handle<Expression>> {
        let mut literals = Vec::with_capacity(dimensions.len() * 2);
        let mut stride = dimensions.iter().product::<u32>();
        for &length in dimensions {
            stride /= length;
            literals.push((
                (stride != 1).then(|| self.literal(Literal::U32(stride))),
                self.literal(Literal::U32(length)),
            ));
        }

        self.emitter.start(&self.function.expressions);
        let mut indices = Vec::with_capacity(dimensions.len());
        for (i, (stride, length)) in literals.into_iter().enumerate() {
            let mut value = flat;
            if let Some(stride) = stride {
                value = self.append(Expression::Binary {
                    op: BinaryOperator::Divide,
                    left: value,
                    right: stride,
                });
            }
            // The flat index is in bounds, so the outermost index is too.
            if i != 0 {
                value = self.append(Expression::Binary {
                    op: BinaryOperator::Modulo,
                    left: value,
                    right: length,
                });
            }
            indices.push(value);
        }
        block.extend(self.emitter.finish(&self.function.expressions));
        indices
    }

    fn write_store(
        &mut self,
        block: &mut Block,
        var: Handle<GlobalVariable>,
        leaf: &Leaf,
        indices: &[Handle<Expression>],
    ) {
        let value = match leaf.value {
            LeafValue::Zero(ty) => self.pre_emit(zero_value(self.types, ty)),
            LeafValue::Atomic(scalar) => {
                self.pre_emit(Expression::Literal(Literal::zero(scalar).unwrap()))
            }
        };
        let mut pointer = self.pre_emit(Expression::GlobalVariable(var));
        self.emitter.start(&self.function.expressions);
        for &step in &leaf.steps {
            pointer = self.append(match step {
                Step::Member(index) => Expression::AccessIndex {
                    base: pointer,
                    index,
                },
                Step::Element(dimension) => Expression::Access {
                    base: pointer,
                    index: indices[dimension],
                },
            });
        }
        block.extend(self.emitter.finish(&self.function.expressions));
        block.push(Statement::Store { pointer, value }, Span::UNDEFINED);
    }

    /// Return the entry point's local invocation index, adding an argument for it if needed.
    fn local_invocation_index(&mut self) -> Handle<Expression> {
        if let Some(index) = self.local_invocation_index {
            return index;
        }
        let binding = Some(Binding::BuiltIn(BuiltIn::LocalInvocationIndex));
        let mut found = None;
        for (index, argument) in self.function.arguments.iter().enumerate() {
            if argument.binding == binding {
                found = Some((index, None));
                break;
            }
            if let TypeInner::Struct { ref members, .. } = self.types[argument.ty].inner {
                if let Some(member) = members.iter().position(|m| m.binding == binding) {
                    found = Some((index, Some(member)));
                    break;
                }
            }
        }
        let (argument, member) = found.unwrap_or_else(|| {
            let ty = self.types.insert(
                Type {
                    name: None,
                    inner: TypeInner::Scalar(Scalar::U32),
                },
                Span::UNDEFINED,
            );
            self.function.arguments.push(FunctionArgument {
                name: Some("local_invocation_index".into()),
                ty,
                binding,
            });
            (self.function.arguments.len() - 1, None)
        });
        let mut index = self.pre_emit(Expression::FunctionArgument(argument as u32));
        if let Some(member) = member {
            index = self.emit(Expression::AccessIndex {
                base: index,
                index: member as u32,
            });
        }
        self.local_invocation_index = Some(index);
        index
    }

    fn counter(&mut self) -> Handle<LocalVariable> {
        *self.counter.get_or_insert_with(|| {
            let ty = self.types.insert(
                Type {
                    name: None,
                    inner: TypeInner::Scalar(Scalar::U32),
                },
                Span::UNDEFINED,
            );
            self.function.local_variables.append(
                LocalVariable {
                    name: Some("zero_init_index".into()),
                    ty,
                    init: None,
                },
                Span::UNDEFINED,
            )
        })
    }

    fn literal(&mut self, literal: Literal) -> Handle<Expression> {
        self.pre_emit(Expression::Literal(literal))
    }

    /// Append an expression that must not be covered by an [`Emit`](Statement::Emit).
    fn pre_emit(&mut self, expression: Expression) -> Handle<Expression> {
        self.function
            .expressions
            .append(expression, Span::UNDEFINED)
    }

    /// Append an expression while the emitter is running.
    fn append(&mut self, expression: Expression) -> Handle<Expression> {
        self.function
            .expressions
            .append(expression, Span::UNDEFINED)
    }

    /// Append `expression` and emit it in the prologue.
    fn emit(&mut self, expression: Expression) -> Handle<Expression> {
        self.emitter.start(&self.function.expressions);
        let handle = self.append(expression);
        self.flush();
        handle
    }

    fn flush(&mut self) {
        if self.emitter.is_running() {
            self.block
                .extend(self.emitter.finish(&self.function.expressions));
        }
    }

    fn push(&mut self, statement: Statement) {
        self.flush();
        self.block.push(statement, Span::UNDEFINED);
    }
}
