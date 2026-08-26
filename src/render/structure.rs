//! Control-flow structuring for recovered function bodies.
//!
//! Turns the flat per-address semantic statements and the block-level CFG
//! into a Dart-shaped statement tree: straight-line sequences, `if`/`else`
//! regions, and `while` loops. Anything the evidence cannot structure stays
//! in a linear tail, so no recovered statement is ever silently dropped.

use std::collections::{BTreeMap, BTreeSet};

use crate::model::{EvidenceConfidence, SemanticStatement};

/// A structured body node. Statement payloads are indices into the original
/// semantic-statement list; the renderer resolves and prints them.
#[derive(Debug)]
pub(crate) enum StructureNode {
    /// Ordered statements with no internal control flow.
    Linear(Vec<usize>),
    If {
        condition: String,
        confidence: EvidenceConfidence,
        then_body: Box<StructureNode>,
        else_body: Option<Box<StructureNode>>,
    },
    While {
        /// `None` renders as `while (true)` for bottom-tested loops.
        condition: Option<String>,
        confidence: EvidenceConfidence,
        body: Box<StructureNode>,
    },
    /// A recovered `return` statement.
    Return(usize),
    /// A low-confidence branch whose both arms contributed no renderable
    /// statement. Rendered as a comment, never as an `if` (the renderer
    /// demotes such branches after structuring; see `dart.rs`).
    UnresolvedPredicate(String),
    /// An exception-handler region: the VM dispatches into `body` on throw
    /// from the protected range. Rendered as a `catch` clause once try
    /// bracketing is available, otherwise as an explicit banner.
    CatchHandler(Box<StructureNode>),
    /// Ordered mixed region (the top-level sequence).
    Block(Vec<StructureNode>),
}

pub(crate) struct StructuredBody {
    pub(crate) root: StructureNode,
    /// Statements not claimed by any structured region.
    pub(crate) unstructured_count: usize,
    pub(crate) claimed: Vec<bool>,
    pub(crate) structured_branches: usize,
    pub(crate) structured_loops: usize,
}

struct Cfg {
    succs: BTreeMap<u64, Vec<u64>>,
    preds: BTreeMap<u64, Vec<u64>>,
    /// Block address -> condition (expression, confidence, true, false).
    conditions: BTreeMap<u64, (String, EvidenceConfidence, u64, u64)>,
    statements_by_block: BTreeMap<u64, Vec<usize>>,
    dominators: BTreeMap<u64, BTreeSet<u64>>,
    /// Block address -> immediate post-dominator (branch join point).
    ipdom: BTreeMap<u64, u64>,
    /// Exception-handler entry blocks decoded from the Code's exception
    /// handler table. Control never falls into them sequentially (the VM
    /// dispatches there), so they must not seed or extend source loops.
    handler_blocks: BTreeSet<u64>,
    /// Blocks reachable from the entry through decoded edges. Blocks outside
    /// this set (handler-dispatch regions whose incoming edges are not in the
    /// CFG) keep full dominator sets that poison every successor's set, so
    /// their outgoing edges must never vote in the loop-header test.
    reachable: BTreeSet<u64>,
}

pub(crate) fn structure_body(
    entry: u64,
    edges: &[crate::model::ControlFlowEdge],
    statements: &[SemanticStatement],
    handler_blocks: &BTreeSet<u64>,
    catch_banners: &BTreeSet<u64>,
) -> StructuredBody {
    let mut starts: BTreeSet<u64> = BTreeSet::from([entry]);
    for edge in edges {
        if let (Some(from), Some(to)) = (parse_address(&edge.from), parse_address(&edge.to)) {
            starts.insert(from);
            starts.insert(to);
        }
    }

    let mut succs: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
    let mut preds: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
    for edge in edges {
        let (Some(from), Some(to)) = (parse_address(&edge.from), parse_address(&edge.to)) else {
            continue;
        };
        succs.entry(from).or_default().push(to);
        preds.entry(to).or_default().push(from);
    }
    for successors in succs.values_mut() {
        successors.sort_unstable();
        successors.dedup();
    }
    for predecessors in preds.values_mut() {
        predecessors.sort_unstable();
        predecessors.dedup();
    }

    // Assign statements to blocks by address.
    let sorted_starts: Vec<u64> = starts.iter().copied().collect();
    let mut statements_by_block: BTreeMap<u64, Vec<usize>> = BTreeMap::new();
    for (index, statement) in statements.iter().enumerate() {
        let Some(address) = parse_address(statement.address()) else {
            continue;
        };
        let hit = sorted_starts.partition_point(|start| *start <= address);
        if hit == 0 {
            continue;
        }
        statements_by_block
            .entry(sorted_starts[hit - 1])
            .or_default()
            .push(index);
    }

    // Branch conditions keyed by their OWNING BLOCK (the greatest block
    // start <= the condition statement's address), matching how the walk
    // queries them per block.
    let mut conditions: BTreeMap<u64, (String, EvidenceConfidence, u64, u64)> = BTreeMap::new();
    for (index, statement) in statements.iter().enumerate() {
        let SemanticStatement::Condition {
            expression,
            true_target,
            false_target,
            confidence,
            ..
        } = statement
        else {
            continue;
        };
        let Some(address) = parse_address(statement_address(statements, index)) else {
            continue;
        };
        let hit = sorted_starts.partition_point(|start| *start <= address);
        if hit == 0 {
            continue;
        }
        let address = sorted_starts[hit - 1];
        let true_target = true_target.as_deref().and_then(parse_address);
        let false_target = false_target.as_deref().and_then(parse_address);
        let (t, f) = match (true_target, false_target) {
            (Some(t), Some(f)) => (t, f),
            _ => {
                // Derive from CFG successor pair when targets were lost.
                let successors = succs.get(&address).cloned().unwrap_or_default();
                if successors.len() != 2 {
                    continue;
                }
                let (a, b) = (successors[0], successors[1]);
                match (true_target, false_target) {
                    (Some(t), None) if t == a => (a, b),
                    (Some(_t), None) => (b, a),
                    (None, Some(f)) if f == b => (a, b),
                    (None, Some(_f)) => (b, a),
                    _ => continue,
                }
            }
        };
        conditions.insert(address, (expression.clone(), *confidence, t, f));
    }

    let dominators = compute_dominators(entry, &starts, &succs, &preds);
    let ipdom = compute_post_dominators(&starts, &succs);
    let mut reachable_set = BTreeSet::from([entry]);
    let mut queue: Vec<u64> = succs
        .get(&entry)
        .map(|successors| successors.to_vec())
        .unwrap_or_default();
    while let Some(node) = queue.pop() {
        if reachable_set.insert(node)
            && let Some(successors) = succs.get(&node)
        {
            queue.extend(successors.iter().copied());
        }
    }
    let reachable = reachable_set;
    let cfg = Cfg {
        succs,
        preds,
        conditions,
        statements_by_block,
        dominators,
        ipdom,
        handler_blocks: handler_blocks.iter().copied().collect(),
        reachable,
    };

    let total_blocks = starts.len();
    let mut state = WalkState {
        visited: BTreeSet::new(),
        claimed: vec![false; statements.len()],
        budget: 64usize.saturating_mul(total_blocks.max(2)).min(20_000),
        branches: 0,
        loops: 0,
    };
    let mut root_pieces = match walk(entry, &cfg, statements, &mut state) {
        StructureNode::Block(pieces) => pieces,
        node => vec![node],
    };
    // The linear walk can stall when every successor of a block was already
    // visited (diamonds that re-join). Resume from the lowest unvisited block
    // that carries evidence so its interior branching still structures.
    loop {
        if state.budget == 0 {
            break;
        }
        let Some(resume) = starts
            .iter()
            .copied()
            .filter(|start| !state.visited.contains(start))
            .find(|start| {
                cfg.statements_by_block
                    .get(start)
                    .is_some_and(|indices| !indices.is_empty())
                    || cfg.conditions.contains_key(start)
            })
        else {
            break;
        };
        let before_claims = state.claimed.iter().filter(|claim| **claim).count();
        let piece = walk(resume, &cfg, statements, &mut state);
        let after_claims = state.claimed.iter().filter(|claim| **claim).count();
        // Always consume the resume point so this cannot loop forever.
        state.visited.insert(resume);
        if after_claims > before_claims {
            root_pieces.push(piece);
        }
    }
    // Handler entry blocks the walk never reached sequentially keep their
    // recovered statements: surface them under an explicit banner instead of
    // letting a later resume-walk fold them into invented control flow.
    // Only *real* handlers (non-generated rows) get the catch banner; the
    // generated finally/dispatch-cleanup entries stay excluded from loop
    // detection above but never render as `catch` clauses.
    for handler in catch_banners {
        if !state.visited.contains(handler)
            && cfg
                .statements_by_block
                .get(handler)
                .is_some_and(|indices| !indices.is_empty())
        {
            let before_claims = state.claimed.iter().filter(|claim| **claim).count();
            let piece = walk(*handler, &cfg, statements, &mut state);
            let after_claims = state.claimed.iter().filter(|claim| **claim).count();
            state.visited.insert(*handler);
            if after_claims > before_claims {
                root_pieces.push(StructureNode::CatchHandler(Box::new(piece)));
            }
        }
    }
    let root = demote_cid_compare_towers(StructureNode::Block(root_pieces), statements);
    let _ = total_blocks;
    StructuredBody {
        root,
        claimed: state.claimed.clone(),
        unstructured_count: state.claimed.iter().filter(|claimed| !**claimed).count(),
        structured_branches: state.branches,
        structured_loops: state.loops,
    }
}

struct WalkState {
    visited: BTreeSet<u64>,
    claimed: Vec<bool>,
    budget: usize,
    branches: usize,
    loops: usize,
}

fn walk(
    entry: u64,
    cfg: &Cfg,
    statements: &[SemanticStatement],
    state: &mut WalkState,
) -> StructureNode {
    let mut pieces: Vec<StructureNode> = Vec::new();
    let mut current = Some(entry);
    while let Some(address) = current {
        if state.budget == 0 || !state.visited.insert(address) {
            break;
        }
        state.budget -= 1;

        // Loop header? A predecessor dominated by this block is a back edge.
        // Handler entry blocks are excluded on BOTH sides: the VM dispatches
        // into them on throw, so an edge leaving one is catch dispatch, never
        // a source loop — and because handlers are unreachable from the
        // ordinary entry they carry full dominator sets, which would
        // otherwise make every edge out of one look like a back edge
        // (this fabricated `while (true)` inside catch paths, probe EC-2).
        let is_loop_header = cfg
            .preds
            .get(&address)
            .map(|predecessors| {
                predecessors.iter().any(|predecessor| {
                    !cfg.handler_blocks.contains(predecessor)
                        && cfg.reachable.contains(predecessor)
                        && cfg
                            .dominators
                            .get(predecessor)
                            .is_some_and(|doms| doms.contains(&address))
                })
            })
            .unwrap_or(false);
        if is_loop_header {
            state.loops += 1;
            pieces.push(emit_loop(address, cfg, statements, state));
            current = cfg
                .succs
                .get(&address)
                .map(|successors| {
                    successors
                        .iter()
                        .copied()
                        .find(|successor| !dominates(cfg, address, *successor))
                })
                .unwrap_or_default();
            continue;
        }

        pieces.push(emit_linear(address, cfg, statements, state));

        if let Some((expression, confidence, true_target, false_target)) =
            cfg.conditions.get(&address).cloned()
        {
            state.branches += 1;
            let merge = find_merge(true_target, false_target, cfg);
            // `walk` marks its entry visited itself; only gate on prior
            // visits here so the sub-walk actually executes.
            let then_body = if !state.visited.contains(&true_target) {
                walk(true_target, cfg, statements, state)
            } else {
                StructureNode::Linear(Vec::new())
            };
            let else_body = if false_target == merge || false_target == true_target {
                None
            } else if !state.visited.contains(&false_target) {
                Some(Box::new(walk(false_target, cfg, statements, state)))
            } else {
                None
            };
            pieces.push(demote_empty_low_confidence_branch(
                expression,
                confidence,
                then_body,
                else_body,
            ));
            current = Some(merge);
            continue;
        }

        current = cfg
            .succs
            .get(&address)
            .and_then(|successors| successors.first().copied())
            .filter(|next| !state.visited.contains(next));
    }
    StructureNode::Block(pieces)
}

/// Emits one block's non-condition statements, mapping a trailing return.
fn emit_linear(
    address: u64,
    cfg: &Cfg,
    statements: &[SemanticStatement],
    state: &mut WalkState,
) -> StructureNode {
    let indices = cfg
        .statements_by_block
        .get(&address)
        .cloned()
        .unwrap_or_default();
    let mut linear = Vec::new();
    for index in indices {
        if matches!(statements[index], SemanticStatement::Condition { .. }) {
            continue;
        }
        state.claimed[index] = true;
        linear.push(index);
        // A machine return terminates the block; nothing later is reachable.
        if matches!(statements[index], SemanticStatement::Return { .. }) {
            break;
        }
    }
    if linear.len() == 1 && matches!(statements[linear[0]], SemanticStatement::Return { .. }) {
        return StructureNode::Return(linear[0]);
    }
    StructureNode::Linear(linear)
}

fn emit_loop(
    header: u64,
    cfg: &Cfg,
    statements: &[SemanticStatement],
    state: &mut WalkState,
) -> StructureNode {
    // Natural loop: blocks dominated by the header that can reach a latch
    // without passing through the header.
    let mut loop_blocks: BTreeSet<u64> = BTreeSet::from([header]);
    let mut queue: Vec<u64> = cfg
        .preds
        .get(&header)
        .map(|predecessors| {
            predecessors
                .iter()
                .copied()
                .filter(|predecessor| dominates(cfg, header, *predecessor))
                .collect()
        })
        .unwrap_or_default();
    while let Some(node) = queue.pop() {
        if loop_blocks.insert(node) {
            if let Some(predecessors) = cfg.preds.get(&node) {
                queue.extend(
                    predecessors
                        .iter()
                        .copied()
                        .filter(|predecessor| dominates(cfg, header, *predecessor)),
                );
            }
        }
    }

    // Header test: a condition splitting loop body from exit becomes the
    // `while` predicate. `while (cond)` loops on the TRUE edge, so a body
    // reached through the false edge needs negation. When both edges stay
    // inside the loop there is no provable predicate; the test renders as an
    // inner `if` instead.
    let in_loop = |target: u64| loop_blocks.contains(&target);
    let header_condition = cfg.conditions.get(&header).cloned();
    let predicate = header_condition
        .as_ref()
        .and_then(
            |(expression, confidence, t, f)| match (in_loop(*t), in_loop(*f)) {
                (true, false) => Some((expression.clone(), *confidence, *t)),
                (false, true) => Some((negate_condition(expression, true), *confidence, *f)),
                _ => None,
            },
        );

    // Walk the loop body recursively so nested branches and loops re-nest,
    // clamping the walk at the loop boundary: exits terminate the body and
    // the latch's back edge re-enters the already-visited header.
    let mut body_nodes: Vec<StructureNode> = Vec::new();
    let body_entry = predicate.as_ref().map_or(header, |(_, _, target)| *target);
    let boundary = loop_blocks.clone();
    let entries: Vec<u64> = if predicate.is_some() {
        vec![body_entry]
    } else {
        cfg.succs
            .get(&header)
            .map(|successors| {
                successors
                    .iter()
                    .copied()
                    .filter(|successor| in_loop(*successor))
                    .collect()
            })
            .unwrap_or_default()
    };
    for entry_block in entries {
        if entry_block == header || state.visited.contains(&entry_block) {
            continue;
        }
        let walked = walk_clamped(entry_block, cfg, statements, state, &boundary);
        body_nodes.push(walked);
    }
    // Header-resident statements outside the predicate itself still belong to
    // the body (e.g. a do-while style header).
    for index in cfg
        .statements_by_block
        .get(&header)
        .cloned()
        .unwrap_or_default()
    {
        if matches!(statements[index], SemanticStatement::Condition { .. }) {
            continue;
        }
        if !state.claimed[index] {
            state.claimed[index] = true;
            match body_nodes.first_mut() {
                Some(StructureNode::Linear(existing)) => existing.insert(0, index),
                _ => body_nodes.insert(0, StructureNode::Linear(vec![index])),
            }
        }
    }
    // Both-edge-inside tests render as an inner `if` around the walk result.
    let inner_test = predicate.is_none().then(|| {
        header_condition
            .as_ref()
            .filter(|(_, _, t, f)| in_loop(*t) && in_loop(*f))
            .map(|(expression, confidence, t, f)| {
                let then_body = if !state.visited.contains(t) && *t != header {
                    walk_clamped(*t, cfg, statements, state, &boundary)
                } else {
                    StructureNode::Linear(Vec::new())
                };
                let else_body = if f != t && !state.visited.contains(f) && *f != header {
                    Some(Box::new(walk_clamped(
                        *f, cfg, statements, state, &boundary,
                    )))
                } else {
                    None
                };
                StructureNode::If {
                    condition: expression.clone(),
                    confidence: *confidence,
                    then_body: Box::new(then_body),
                    else_body,
                }
            })
            .map(|node| vec![node])
            .unwrap_or_default()
    });

    // Mark loop-internal blocks visited so outer walks do not re-enter.
    for block in &loop_blocks {
        state.visited.insert(*block);
    }

    // Dart lowers `for-in` and many `while` loops to `while (true)` with the
    // exit test as the FIRST branch inside the body: `if (!moveNext()) {}`
    // else { rest }`. One arm being empty is exactly the loop-exit edge (the
    // clamped walk renders it as nothing), so promote that test into the
    // `while` predicate and keep only the continuation in the body.
    let mut promoted_condition: Option<(String, EvidenceConfidence)> = None;
    if predicate.is_none() {
        flatten_grouping_blocks(&mut body_nodes);
        promoted_condition = promote_exit_test(&mut body_nodes);
    }
    let final_condition = predicate
        .as_ref()
        .map(|(expression, _, _)| expression.clone())
        .or_else(|| {
            promoted_condition
                .as_ref()
                .map(|(condition, _)| condition.clone())
        });
    let final_confidence = predicate
        .as_ref()
        .map(|(_, confidence, _)| *confidence)
        .unwrap_or(
            promoted_condition
                .as_ref()
                .map_or(EvidenceConfidence::Low, |(_, confidence)| *confidence),
        );

    let mut body = inner_test.unwrap_or_default();
    body.extend(body_nodes);
    StructureNode::While {
        condition: final_condition,
        confidence: final_confidence,
        body: Box::new(StructureNode::Block(body)),
    }
}

/// Dart lowers `for-in` and many `while` loops to `while (true)` with the
/// exit test as the FIRST branch inside the body: `if (!moveNext()) {} else
/// { rest }`. One arm being empty is exactly the loop-exit edge (the clamped
/// walk renders it as nothing), so promote that test into the `while`
/// predicate and keep only the continuation in the body. Straight-line
/// statements before the test run on every iteration before it, which is
/// exactly where a `while` predicate sits, so they stay ahead of the
/// continuation.
fn promote_exit_test(nodes: &mut [StructureNode]) -> Option<(String, EvidenceConfidence)> {
    for node in nodes.iter_mut() {
        match node {
            // Straight-line statements ahead of the test run on every
            // iteration before it — exactly where a `while` predicate sits.
            StructureNode::Linear(_) => continue,
            StructureNode::Block(children) => {
                let promoted = promote_exit_test(children)?;
                return Some(promoted);
            }
            StructureNode::If {
                condition,
                confidence,
                then_body,
                else_body,
            } => {
                let then_exits =
                    matches!(then_body.as_ref(), StructureNode::Linear(v) if v.is_empty());
                let else_continues = else_body.as_ref().is_some_and(
                    |body| !matches!(body.as_ref(), StructureNode::Linear(v) if v.is_empty()),
                );
                if !then_exits || !else_continues {
                    return None;
                }
                let promoted = (negate_condition(condition, true), *confidence);
                let continuation = else_body.take().expect("checked above");
                *node = *continuation;
                return Some(promoted);
            }
            _ => return None,
        }
    }
    None
}

/// Splices nested pure-grouping `Block`s into their parent so pattern scans
/// see the statement sequence directly. Blocks that carry control structure
/// (an `If`/`While`/`Return` anywhere inside) are preserved as boundaries.
fn flatten_grouping_blocks(nodes: &mut Vec<StructureNode>) {
    let mut flat = Vec::with_capacity(nodes.len());
    for node in nodes.drain(..) {
        match node {
            StructureNode::Block(children) => {
                let mut inner = children;
                flatten_grouping_blocks(&mut inner);
                let has_control = inner.iter().any(|node| {
                    matches!(
                        node,
                        StructureNode::If { .. }
                            | StructureNode::While { .. }
                            | StructureNode::Return(_)
                    )
                });
                if has_control {
                    flat.push(StructureNode::Block(inner));
                } else {
                    flat.extend(inner);
                }
            }
            other => flat.push(other),
        }
    }
    *nodes = flat;
}

/// `walk` restricted to a region of the CFG: blocks outside `boundary` end
/// the linear chain (they are the loop exit path and remain for the outer
/// walk to structure).
fn walk_clamped(
    entry: u64,
    cfg: &Cfg,
    statements: &[SemanticStatement],
    state: &mut WalkState,
    boundary: &BTreeSet<u64>,
) -> StructureNode {
    if !boundary.contains(&entry) || state.visited.contains(&entry) {
        return StructureNode::Linear(Vec::new());
    }
    // Temporarily install the boundary as a stop set by wrapping the walk:
    // any successor leaving the region is treated as terminal.
    let mut pieces = Vec::new();
    let mut current = Some(entry);
    while let Some(address) = current {
        if state.budget == 0 || !state.visited.insert(address) || !boundary.contains(&address) {
            break;
        }
        state.budget -= 1;

        // Nested natural loop inside this region: recurse through emit_loop,
        // which itself walks its body clamped to the nested boundary.
        // Handler blocks stay excluded on both sides of the back-edge test
        // here as well (see `walk`'s loop-header comment).
        let is_nested_loop_header = cfg
            .preds
            .get(&address)
            .map(|predecessors| {
                predecessors.iter().any(|predecessor| {
                    !cfg.handler_blocks.contains(predecessor)
                        && cfg.reachable.contains(predecessor)
                        && cfg
                            .dominators
                            .get(predecessor)
                            .is_some_and(|doms| doms.contains(&address))
                })
            })
            .unwrap_or(false);
        if is_nested_loop_header {
            state.loops += 1;
            pieces.push(emit_loop(address, cfg, statements, state));
            current = cfg
                .succs
                .get(&address)
                .map(|successors| {
                    successors.iter().copied().find(|successor| {
                        !dominates(cfg, address, *successor)
                            && !state.visited.contains(successor)
                            && boundary.contains(successor)
                    })
                })
                .unwrap_or_default();
            continue;
        }

        pieces.push(emit_linear(address, cfg, statements, state));
        if let Some((expression, confidence, true_target, false_target)) =
            cfg.conditions.get(&address).cloned()
        {
            state.branches += 1;
            let merge = find_merge(true_target, false_target, cfg);
            let merge_in_region = boundary.contains(&merge);
            let then_body = if !state.visited.contains(&true_target) {
                walk_clamped(true_target, cfg, statements, state, boundary)
            } else {
                StructureNode::Linear(Vec::new())
            };
            let else_body = if false_target == merge || false_target == true_target {
                None
            } else if !state.visited.contains(&false_target) {
                Some(Box::new(walk_clamped(
                    false_target,
                    cfg,
                    statements,
                    state,
                    boundary,
                )))
            } else {
                None
            };
            pieces.push(demote_empty_low_confidence_branch(
                expression,
                confidence,
                then_body,
                else_body,
            ));
            current = merge_in_region.then_some(merge);
            continue;
        }
        current = cfg
            .succs
            .get(&address)
            .and_then(|successors| successors.first().copied())
            .filter(|next| !state.visited.contains(next) && boundary.contains(next));
    }
    StructureNode::Block(pieces)
}

fn negate_condition(condition: &str, negated: bool) -> String {
    if !negated {
        return condition.to_owned();
    }
    // A whole-wrapped negation cancels: `!(x)` -> `x`.
    if let Some(inner) = whole_wrapped_negation(condition) {
        return inner.to_owned();
    }
    // Negate a simple comparison; anything else gets an explicit `!`.
    for (from, to) in [
        (" == ", " != "),
        (" != ", " == "),
        (" < ", " >= "),
        (" >= ", " < "),
        (" > ", " <= "),
        (" <= ", " > "),
    ] {
        if let Some(position) = condition.find(from) {
            let mut output = String::with_capacity(condition.len() + 2);
            output.push_str(&condition[..position]);
            output.push_str(to);
            output.push_str(&condition[position + from.len()..]);
            return output;
        }
    }
    format!("!({condition})")
}

/// `!(x)` -> `x` when the paren opened after `!` closes at the very end.
fn whole_wrapped_negation(condition: &str) -> Option<&str> {
    if !condition.starts_with("!(") || !condition.ends_with(')') {
        return None;
    }
    let mut depth = 0i32;
    for (index, byte) in condition.bytes().enumerate() {
        match byte {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return if index == condition.len() - 1 {
                        Some(&condition[2..condition.len() - 1])
                    } else {
                        None
                    };
                }
            }
            _ => {}
        }
    }
    None
}

/// Low-confidence branches whose both arms carry no renderable statement are
/// lifter noise over machine registers, not source control flow (probe EC-4:
/// six nested empty `if (x2 != lookupResult)` shells). Demote them to an
/// explicit comment node; any branch with a non-empty arm stays an `if`.
fn demote_empty_low_confidence_branch(
    condition: String,
    confidence: EvidenceConfidence,
    then_body: StructureNode,
    else_body: Option<Box<StructureNode>>,
) -> StructureNode {
    fn has_content(node: &StructureNode) -> bool {
        match node {
            StructureNode::Linear(indices) => !indices.is_empty(),
            StructureNode::Return(_) => true,
            StructureNode::If { .. } | StructureNode::While { .. } | StructureNode::UnresolvedPredicate(_) => {
                true
            }
            StructureNode::CatchHandler(body) => has_content(body),
            StructureNode::Block(children) => children.iter().any(has_content),
        }
    }
    if confidence == EvidenceConfidence::High
        || has_content(&then_body)
        || else_body.as_deref().is_some_and(has_content)
    {
        return StructureNode::If {
            condition,
            confidence,
            then_body: Box::new(then_body),
            else_body,
        };
    }
    // The arms may still have claimed statements that rendered nothing
    // render-worthy (nested demotions); the comment keeps the predicate's
    // evidence visible without inventing executable Dart.
    StructureNode::UnresolvedPredicate(condition)
}


/// Subtype-test cache dispatch compiles to towers of low-confidence register
/// compares against class-id integers (`x4 <= 55`, `x4 == 2046`, …) whose only
/// descendants are more such compares (probe EC-4 / E17). No source control
/// flow exists there; a tower whose every leaf is empty collapses into a
/// single explicit comment naming the compared constants.
fn demote_cid_compare_towers(
    node: StructureNode,
    statements: &[SemanticStatement],
) -> StructureNode {
    fn cid_constants(condition: &str, confidence: EvidenceConfidence) -> Option<String> {
        if confidence != EvidenceConfidence::Low {
            return None;
        }
        let compact = condition.replace(' ', "");
        // Shape gate: `<register-token><compare><integer>` — the register
        // token leads, an operator follows, and the trailing digit run is the
        // compared constant. Anything else (masks, field offsets) stays.
        let digits_start = compact.rfind(|c: char| !c.is_ascii_digit())? + 1;
        let digits = &compact[digits_start..];
        if digits.is_empty() {
            return None;
        }
        let head = &compact[..compact.len() - digits.len()];
        (head.contains('=') || head.contains('<') || head.contains('>'))
            .then(|| {
                let register = head.trim_end_matches(|c: char| !c.is_ascii_alphabetic());
                format!("{} {}", register, digits)
            })
            .filter(|label| label.split(' ').next().is_some_and(|token| !token.is_empty()))
    }

    fn tower_collapsed(
        node: &StructureNode,
        collected: &mut Vec<String>,
        statements: &[SemanticStatement],
    ) -> bool {
        match node {
            StructureNode::UnresolvedPredicate(text) => {
                if let Some(inner) = text
                    .strip_prefix("subtype-test cache dispatch (compared cid constants: ")
                    .and_then(|rest| rest.strip_suffix(')'))
                {
                    collected.push(inner.to_owned());
                    true
                } else {
                    false
                }
            }
            // A linear run whose every statement is an unresolved synthetic
            // call is the runtime throw/unreachable tail of a failed cache
            // probe — machine semantics, not source content.
            StructureNode::Linear(indices) => indices.iter().all(|index| {
                match statements.get(*index) {
                    None => true,
                    Some(SemanticStatement::ResolvedCall { target, .. }) => {
                        target.starts_with("sub_")
                    }
                    _ => false,
                }
            }),
            StructureNode::Block(children) => children
                .iter()
                .all(|child| tower_collapsed(child, collected, statements)),
            _ => false,
        }
    }

    match node {
        StructureNode::If {
            condition,
            confidence,
            then_body,
            else_body,
        } => {
            let demoted_then =
                demote_cid_compare_towers(*then_body, statements);
            let demoted_else = else_body
                .map(|body| Box::new(demote_cid_compare_towers(*body, statements)));
            if let Some(constants) = cid_constants(&condition, confidence) {
                let mut collected = vec![constants];
                let then_collapsed =
                    tower_collapsed(&demoted_then, &mut collected, statements);
                let else_collapsed = demoted_else
                    .as_deref()
                    .map(|body| tower_collapsed(body, &mut collected, statements))
                    .unwrap_or(true);
                if then_collapsed && else_collapsed {
                    return StructureNode::UnresolvedPredicate(format!(
                        "subtype-test cache dispatch (compared cid constants: {})",
                        collected.join(", ")
                    ));
                }
            }
            StructureNode::If {
                condition,
                confidence,
                then_body: Box::new(demoted_then),
                else_body: demoted_else,
            }
        }
        StructureNode::Block(children) => StructureNode::Block(
            children.into_iter().map(|child| demote_cid_compare_towers(child, statements)).collect(),
        ),
        StructureNode::CatchHandler(body) => {
            StructureNode::CatchHandler(Box::new(demote_cid_compare_towers(*body, statements)))
        }
        other => other,
    }
}

fn dominates(cfg: &Cfg, dominator: u64, candidate: u64) -> bool {
    cfg.dominators
        .get(&candidate)
        .is_some_and(|doms| doms.contains(&dominator))
}

fn find_merge(true_target: u64, false_target: u64, cfg: &Cfg) -> u64 {
    // The branch's immediate post-dominator is the exact join point when the
    // CFG reached it; this keeps diamonds that re-enter an arm from folding
    // into each other (a plain nearest-common-successor search picks the
    // true edge itself there).
    if let Some(join) = cfg
        .ipdom
        .get(&true_target)
        .or_else(|| cfg.ipdom.get(&false_target))
    {
        return *join;
    }
    // Bounded forward search for the nearest common successor.
    let mut reachable_from_true = BTreeSet::new();
    let mut queue = std::collections::VecDeque::from([true_target]);
    let mut limit = 256usize;
    while let Some(node) = queue.pop_front() {
        if limit == 0 || !reachable_from_true.insert(node) {
            continue;
        }
        limit -= 1;
        if let Some(successors) = cfg.succs.get(&node) {
            queue.extend(successors.iter().copied());
        }
    }
    let mut visited = BTreeSet::new();
    queue = std::collections::VecDeque::from([false_target]);
    limit = 256usize;
    let mut best: Option<u64> = None;
    while let Some(node) = queue.pop_front() {
        if limit == 0 || !visited.insert(node) {
            continue;
        }
        limit -= 1;
        if reachable_from_true.contains(&node) {
            best = Some(match best {
                Some(current) if current <= node => current,
                _ => node,
            });
        }
        if let Some(successors) = cfg.succs.get(&node) {
            queue.extend(successors.iter().copied());
        }
    }
    best.unwrap_or(false_target)
}

/// Immediate post-dominators via the same iterative dominator sweep run on
/// the reversed CFG. Every return block feeds one virtual exit node, so
/// blocks that fall off the end of a region still join somewhere sensible.
fn compute_post_dominators(
    starts: &BTreeSet<u64>,
    succs: &BTreeMap<u64, Vec<u64>>,
) -> BTreeMap<u64, u64> {
    const EXIT: u64 = u64::MAX;
    let mut reverse_succs: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
    let mut reverse_preds: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
    for (&from, successors) in succs {
        for to in successors {
            reverse_succs.entry(*to).or_default().push(from);
            reverse_preds.entry(from).or_default().push(*to);
        }
    }
    for start in starts {
        let exits = succs
            .get(start)
            .is_none_or(|successors| successors.is_empty());
        if exits {
            reverse_succs.entry(EXIT).or_default().push(*start);
            reverse_preds.entry(*start).or_default().push(EXIT);
        }
    }
    if !reverse_preds.contains_key(&EXIT) && !starts.contains(&EXIT) {
        reverse_preds.entry(EXIT).or_default();
    }
    let all_blocks: BTreeSet<u64> = starts
        .iter()
        .copied()
        .chain(std::iter::once(EXIT))
        .collect();
    let dominators = compute_dominators(EXIT, &all_blocks, &reverse_succs, &reverse_preds);
    let mut ipdom = BTreeMap::new();
    for (&block, dominator_set) in &dominators {
        if block == EXIT {
            continue;
        }
        // The immediate post-dominator is the strict post-dominator with the
        // largest own post-dominator set (closest to the block).
        let mut candidates: Vec<u64> = dominator_set
            .iter()
            .filter(|candidate| **candidate != block)
            .copied()
            .collect();
        candidates.sort_by_key(|candidate| dominators.get(candidate).map_or(0, |set| set.len()));
        if let Some(nearest) = candidates.pop() {
            ipdom.insert(block, nearest);
        }
    }
    ipdom
}

fn statement_address(statements: &[SemanticStatement], index: usize) -> &str {
    statements[index].address()
}

fn parse_address(value: &str) -> Option<u64> {
    u64::from_str_radix(value.trim_start_matches("0x"), 16).ok()
}

/// Iterative dominator computation (Cooper-Harvey-Kennedy style RPO sweep).
fn compute_dominators(
    entry: u64,
    starts: &BTreeSet<u64>,
    succs: &BTreeMap<u64, Vec<u64>>,
    preds: &BTreeMap<u64, Vec<u64>>,
) -> BTreeMap<u64, BTreeSet<u64>> {
    // Reverse postorder via iterative DFS.
    let mut postorder: Vec<u64> = Vec::new();
    let mut visited: BTreeSet<u64> = BTreeSet::new();
    let mut stack: Vec<(u64, usize)> = vec![(entry, 0)];
    visited.insert(entry);
    while let Some((node, index)) = stack.pop() {
        let successors = succs.get(&node).cloned().unwrap_or_default();
        if index < successors.len() {
            stack.push((node, index + 1));
            let successor = successors[index];
            if visited.insert(successor) {
                stack.push((successor, 0));
            }
        } else {
            postorder.push(node);
        }
    }
    let mut rpo = postorder;
    rpo.reverse();
    // Include unreachable blocks at the end so their sets stay full.
    let mut order: BTreeMap<u64, usize> = BTreeMap::new();
    for (index, node) in rpo.iter().enumerate() {
        order.insert(*node, index);
    }
    for node in starts {
        order.entry(*node).or_insert(usize::MAX);
    }

    let full: BTreeSet<u64> = starts.iter().copied().collect();
    let mut dominators: BTreeMap<u64, BTreeSet<u64>> = BTreeMap::new();
    for node in starts {
        dominators.insert(
            *node,
            if *node == entry {
                BTreeSet::from([entry])
            } else {
                full.clone()
            },
        );
    }
    let mut changed = true;
    while changed {
        changed = false;
        for node in rpo.iter().copied().chain(starts.iter().copied()) {
            if node == entry {
                continue;
            }
            let Some(predecessors) = preds.get(&node) else {
                continue;
            };
            let mut new_set: Option<BTreeSet<u64>> = None;
            for predecessor in predecessors {
                if !order.contains_key(predecessor) {
                    continue;
                }
                let Some(predecessor_doms) = dominators.get(predecessor) else {
                    continue;
                };
                new_set = Some(match new_set {
                    None => predecessor_doms.clone(),
                    Some(current) => current.intersection(predecessor_doms).copied().collect(),
                });
            }
            let mut new_set = new_set.unwrap_or_else(|| full.clone());
            new_set.insert(node);
            if let Some(existing) = dominators.get_mut(&node)
                && *existing != new_set
            {
                *existing = new_set;
                changed = true;
            }
        }
    }
    dominators
}

#[cfg(test)]
mod tests {
    use super::{StructureNode, negate_condition, promote_exit_test, structure_body, whole_wrapped_negation};
    use crate::model::{ControlFlowEdge, ControlFlowEdgeKind, EvidenceConfidence, SemanticStatement};
    use std::collections::BTreeSet;

    #[test]
    fn unwraps_whole_wrapped_negations_only() {
        assert_eq!(negate_condition("!(x)", true), "x");
        // One cancellation per level: the inner negation stays.
        assert_eq!(negate_condition("!(!((a)))", true), "!((a))");
        // A partial wrap still negates as a comparison when one exists.
        assert_eq!(negate_condition("!(a) == (b)", true), "!(a) != (b)");
        assert_eq!(negate_condition("a == b", true), "a != b");
        assert_eq!(
            negate_condition("a != b", false),
            "a != b",
            "positive request must not touch the condition"
        );
    }

    #[test]
    fn detects_fully_balanced_negation_wraps() {
        assert_eq!(whole_wrapped_negation("!(x)"), Some("x"));
        assert_eq!(whole_wrapped_negation("!(f(a))"), Some("f(a)"));
        assert_eq!(whole_wrapped_negation("!(a) == (b)"), None);
        assert_eq!(whole_wrapped_negation("a"), None);
    }

    fn linear(indices: &[usize]) -> StructureNode {
        StructureNode::Linear(indices.to_vec())
    }

    #[test]
    fn promotes_leading_exit_test_into_loop_predicate() {
        let mut nodes = vec![
            linear(&[]),
            StructureNode::Block(vec![
                linear(&[2]),
                StructureNode::If {
                    condition: "!(moveNext())".to_owned(),
                    confidence: EvidenceConfidence::Medium,
                    then_body: Box::new(linear(&[])),
                    else_body: Some(Box::new(StructureNode::Block(vec![linear(&[3, 4])]))),
                },
            ]),
        ];
        let promoted = promote_exit_test(&mut nodes).expect("exit test should promote");
        assert_eq!(promoted.0, "moveNext()");
        match &nodes[1] {
            StructureNode::Block(children) => match &children[1] {
                StructureNode::Block(inner) => match inner.first() {
                    Some(StructureNode::Linear(indices)) => {
                        assert_eq!(indices, &[3, 4]);
                    }
                    other => panic!("continuation not linear: {other:?}"),
                },
                other => panic!("continuation not spliced: {other:?}"),
            },
            other => panic!("if not replaced: {other:?}"),
        }
    }

    #[test]
    fn does_not_promote_when_no_arm_exits() {
        let mut nodes = vec![StructureNode::If {
            condition: "flag".to_owned(),
            confidence: EvidenceConfidence::High,
            then_body: Box::new(linear(&[0])),
            else_body: Some(Box::new(linear(&[1]))),
        }];
        assert!(promote_exit_test(&mut nodes).is_none());
        assert!(
            matches!(&nodes[0], StructureNode::If { .. }),
            "the branch must stay intact"
        );
    }

    /// Regression (probe EC-2 / generated finally handler): a block reachable
    /// only through an exception-dispatch region carries a full dominator set
    /// (its unreachable predecessor keeps one), which used to make its
    /// outgoing edge look like a back edge and fabricate `while (true)` in
    /// catch paths. The structurer must ignore predecessors that are not
    /// reachable from the entry.
    #[test]
    fn poisoned_dominator_edge_does_not_fabricate_loop() {
        // entry -> a; a -> exit (conditional). The dispatch block `d` has no
        // incoming decoded edge (VM throws into it); d -> b, b -> a. With the
        // poisoned full set on d, b's dominators swallowed everything and the
        // b -> a edge looked like a loop latch at `a`.
        let edges = [
            ("0x100", "0x110"),
            ("0x110", "0x140"),
            ("0x120", "0x130"),
            ("0x130", "0x110"),
        ];
        let control_flow = edges
            .iter()
            .map(|(from, to)| ControlFlowEdge {
                from: from.to_string(),
                to: to.to_string(),
                kind: ControlFlowEdgeKind::Fallthrough,
            })
            .collect::<Vec<_>>();
        // One statement per block so every block carries evidence.
        let statements = ["0x102", "0x112", "0x132", "0x122"]
            .iter()
            .map(|address| SemanticStatement::ResolvedCall {
                target: format!("sub_{address}"),
                arguments: Vec::new(),
                confidence: EvidenceConfidence::Medium,
                address: address.to_string(),
            })
            .collect::<Vec<_>>();
        // 0x120 is the exception-dispatch region: a handler block.
        let handlers = BTreeSet::from([0x120]);
        let structured = structure_body(
            0x100,
            &control_flow,
            &statements,
            &handlers,
            &BTreeSet::new(),
        );
        fn count_loops(node: &StructureNode) -> usize {
            match node {
                StructureNode::While { .. } => 1,
                StructureNode::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    count_loops(then_body)
                        + else_body.as_ref().map(|body| count_loops(body)).unwrap_or(0)
                }
                StructureNode::Block(children) => children.iter().map(count_loops).sum(),
                _ => 0,
            }
        }
        assert_eq!(count_loops(&structured.root), 0);
    }

    #[test]
    fn demotes_cid_compare_towers_to_single_comment() {
        use super::demote_cid_compare_towers;
        let shell = |condition: &str, then_body: StructureNode, else_body: Option<StructureNode>| {
            StructureNode::If {
                condition: condition.to_owned(),
                confidence: EvidenceConfidence::Low,
                then_body: Box::new(then_body),
                else_body: else_body.map(Box::new),
            }
        };
        let tower = shell(
            "x4 <= 55",
            StructureNode::Block(Vec::new()),
            Some(shell(
                "x4 == 2046",
                StructureNode::Block(Vec::new()),
                Some(shell(
                    "x4 == 2105",
                    StructureNode::Block(Vec::new()),
                    Some(StructureNode::Block(Vec::new())),
                )),
            )),
        );
        let statements: Vec<SemanticStatement> = Vec::new();
        let demoted = demote_cid_compare_towers(tower, &statements);
        match &demoted {
            StructureNode::UnresolvedPredicate(text) => {
                assert!(text.contains("2046"), "missing 2046: {text}");
                assert!(text.contains("2105"), "missing 2105: {text}");
            }
            other => panic!("tower not collapsed: {other:?}"),
        }
        // A tower whose arm carries real statements stays an `if`.
        let with_content = shell(
            "x4 <= 55",
            StructureNode::Block(vec![StructureNode::Return(0)]),
            None,
        );
        assert!(matches!(
            demote_cid_compare_towers(with_content, &statements),
            StructureNode::If { .. }
        ));
    }
}
