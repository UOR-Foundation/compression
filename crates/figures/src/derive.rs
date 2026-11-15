//! The native domain pass: restate each literal of the native code that is a
//! characteristic quantity of the domain it acts in as the model's quantity.
//!
//! The native code declares, once, which of its items hold which domain of
//! the model: a field, constant, static or function (its result) whose
//! documentation names `Domain: <domain>[, <domain>…]`, and a function's
//! parameter by `Domain: <parameter>: <domain>`. A domain is a name prefix of
//! the model's quantities (`stretch` holds `stretchRange`, `stretchHalfLast`,
//! …; a quantity stated as another less one, such as `cell1Last`, brings that
//! other, its extent, into the domain).
//!
//! The pass infers the domains of every expression of a file, to a fixpoint
//! (a local, field or static takes the domains of what is stored in it), the
//! way the model's own pass (`client/gen/domains.py`) does over the coder IR,
//! and then rewrites each literal whose value is a quantity of a domain in
//! scope at it (the domains of the operands it acts with, and of where its
//! value goes) as that quantity, read from the module generated from the model
//! ([`crate::native`]). A shift's count is placed among the base-two logarithms
//! of the quantities in scope. A literal it cannot place keeps its numeral and
//! is reported with the domains in scope, so the residue names the next
//! declarations to make.
//!
//! The rewritten site names the quantity in the integer view of the type the
//! literal had; that type is read back from the compiler (`cargo check`), which
//! is the only judge of a Rust literal's type, so the rewrite never changes a
//! value or a type, and the native output stays byte-identical.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use syn::visit::{self, Visit};

use crate::native;

/// The model module a native file reads its quantities from.
#[derive(Clone, Debug)]
pub struct Model {
    /// Rust path of the generated module (`crate::model::ncm`).
    pub path: String,
    /// Quantity values by name, in the module's order.
    pub values: Vec<(String, u128)>,
    /// Constant tables read as cells: (view name, cells).
    pub tables: Vec<(String, Vec<u128>)>,
    /// Quantity bodies: the extent a `X - 1` quantity is stated from.
    pub less_one: BTreeMap<String, String>,
}

impl Model {
    /// Read a generated module's source: its model source and quantities.
    pub fn load(root: &Path, generated: &str) -> Result<Self, crate::Error> {
        let (_, src) = native::GENERATED
            .iter()
            .find(|(g, _)| *g == generated)
            .ok_or_else(|| format!("{generated} is not a generated native module"))?;
        let text = std::fs::read_to_string(root.join(src)).map_err(|e| format!("{src}: {e}"))?;
        let (nats, raw) = native::quantities(&text);
        let tables = native::table_cells(&raw)
            .into_iter()
            .map(|(n, w, vs)| (format!("{n}Cells{w}"), vs))
            .collect();
        let mut values = Vec::new();
        let mut less_one = BTreeMap::new();
        for (n, e, v) in &nats {
            values.push((n.clone(), *v));
            if let native::Expr::Op(native::Op::Sub, a, b) = e {
                if let (native::Expr::Name(x), native::Expr::Num(1)) = (&**a, &**b) {
                    less_one.insert(n.clone(), x.clone());
                }
            }
        }
        let module = generated
            .trim_start_matches("crates/uorc/src/")
            .trim_end_matches(".rs")
            .replace('/', "::");
        Ok(Model {
            path: format!("crate::{module}"),
            tables,
            values,
            less_one,
        })
    }

    /// The quantities of a domain: those its name prefixes, and the extent
    /// each `X - 1` among them is stated from.
    pub fn domain(&self, d: &str) -> Vec<(String, u128)> {
        let mut names: BTreeSet<String> = BTreeSet::new();
        for (n, _) in &self.values {
            if prefixed(n, d) {
                names.insert(n.clone());
                if let Some(x) = self.less_one.get(n) {
                    names.insert(x.clone());
                }
            }
        }
        self.values
            .iter()
            .filter(|(n, _)| names.contains(n))
            .cloned()
            .collect()
    }
}

/// The characteristic quantities a domain's name may carry, as the model's
/// declarations name them: its extent (`Range`, `Base`), its unit, origin and
/// top, a map's step, points and scale, a count's limit, and a rescale to
/// another domain (`Per<Domain>`); each also as its last value, its half and
/// the half's last value.
const CHARACTERISTIC: [&str; 9] = [
    "Range", "Base", "Unit", "Origin", "Top", "Step", "Points", "Scale", "Limit",
];
const DERIVED: [&str; 3] = ["Last", "Half", "HalfLast"];

fn prefixed(name: &str, d: &str) -> bool {
    let Some(rest) = name.strip_prefix(d) else {
        return false;
    };
    let rest = DERIVED
        .iter()
        .rev()
        .find_map(|x| rest.strip_suffix(x))
        .unwrap_or(rest);
    rest.is_empty()
        || CHARACTERISTIC.contains(&rest)
        || rest
            .strip_prefix("Per")
            .and_then(|r| r.chars().next())
            .is_some_and(|c| c.is_ascii_uppercase())
}

/// A set of domain names.
type Ds = BTreeSet<String>;

/// A placed literal: its byte range in the file, the quantity, and whether it
/// is a base-two logarithm (a shift count).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Placed {
    /// Byte range of the literal.
    pub range: (usize, usize),
    /// Line of the literal.
    pub line: usize,
    /// The literal's value.
    pub value: String,
    /// The literal's type suffix, if it has one.
    pub suffix: String,
    /// The quantity's name.
    pub name: String,
    /// A shift count: the quantity's base-two logarithm.
    pub bits: bool,
    /// Rendered as this expression instead of a model quantity (the width
    /// of the integer type the bytes are read into).
    pub raw: Option<String>,
}

/// What the pass finds in one file.
#[derive(Clone, Debug, Default)]
pub struct Outcome {
    /// Literals restated as quantities.
    pub placed: Vec<Placed>,
    /// Literals left, by (value, domains in scope).
    pub residue: BTreeMap<(String, String), u64>,
    /// Literals left: (line, value, domains in scope).
    pub left: Vec<(usize, String, String)>,
}

fn tags(attrs: &[syn::Attribute]) -> Vec<String> {
    let mut out = Vec::new();
    for a in attrs {
        if let syn::Meta::NameValue(nv) = &a.meta {
            if nv.path.is_ident("doc") {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) = &nv.value
                {
                    if let Some(rest) = s.value().trim().strip_prefix("Domain:") {
                        out.push(rest.trim().to_string());
                    }
                }
            }
        }
    }
    out
}

fn split(s: &str) -> Ds {
    s.split(',')
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty())
        .collect()
}

/// Declarations read from a file's `Domain:` tags.
#[derive(Clone, Debug, Default)]
struct Decls {
    /// Fields, constants and statics by name.
    items: BTreeMap<String, Ds>,
    /// The domains of an array item's length (`Domain: len: <domain>`), by name.
    lens: BTreeMap<String, Ds>,
    /// Function results by name.
    results: BTreeMap<String, Ds>,
    /// Function parameters by (function, parameter).
    params: BTreeMap<(String, String), Ds>,
    /// Function parameter names in order.
    order: BTreeMap<String, Vec<String>>,
    /// Every declared domain, in order of first declaration.
    domains: Vec<String>,
}

impl Decls {
    fn note(&mut self, ds: &Ds) {
        for d in ds {
            if !self.domains.contains(d) {
                self.domains.push(d.clone());
            }
        }
    }

    fn func(&mut self, name: &str, attrs: &[syn::Attribute], sig: &syn::Signature) {
        let names: Vec<String> = sig
            .inputs
            .iter()
            .map(|i| match i {
                syn::FnArg::Receiver(_) => "self".to_string(),
                syn::FnArg::Typed(t) => match &*t.pat {
                    syn::Pat::Ident(p) => p.ident.to_string(),
                    _ => String::new(),
                },
            })
            .collect();
        self.order.insert(name.to_string(), names);
        for t in tags(attrs) {
            if let Some(l) = t.strip_prefix("len:") {
                let ds = split(l);
                self.note(&ds);
                self.lens
                    .entry(format!("fn {name}"))
                    .or_default()
                    .extend(ds);
                continue;
            }
            if let Some(l) = t.strip_prefix("body:") {
                let ds = split(l);
                self.note(&ds);
                self.lens
                    .entry(format!("body {name}"))
                    .or_default()
                    .extend(ds);
                continue;
            }
            match t.split_once(':') {
                Some((p, d)) => {
                    let ds = split(d);
                    self.note(&ds);
                    self.params
                        .entry((name.to_string(), p.trim().to_string()))
                        .or_default()
                        .extend(ds);
                }
                None => {
                    let ds = split(&t);
                    self.note(&ds);
                    self.results.entry(name.to_string()).or_default().extend(ds);
                }
            }
        }
    }

    fn item(&mut self, name: &str, attrs: &[syn::Attribute]) {
        for t in tags(attrs) {
            if let Some(l) = t.strip_prefix("len:") {
                let ds = split(l);
                self.note(&ds);
                self.lens.entry(name.to_string()).or_default().extend(ds);
                continue;
            }
            let ds = split(&t);
            self.note(&ds);
            self.items.entry(name.to_string()).or_default().extend(ds);
        }
    }
}

struct Collect<'a>(&'a mut Decls);

impl<'ast> Visit<'ast> for Collect<'_> {
    fn visit_field(&mut self, f: &'ast syn::Field) {
        if let Some(i) = &f.ident {
            self.0.item(&i.to_string(), &f.attrs);
        }
        visit::visit_field(self, f);
    }
    fn visit_item_struct(&mut self, st: &'ast syn::ItemStruct) {
        self.0.item(&st.ident.to_string(), &st.attrs);
        visit::visit_item_struct(self, st);
    }
    fn visit_item_const(&mut self, c: &'ast syn::ItemConst) {
        self.0.item(&c.ident.to_string(), &c.attrs);
        visit::visit_item_const(self, c);
    }
    fn visit_impl_item_const(&mut self, c: &'ast syn::ImplItemConst) {
        self.0.item(&c.ident.to_string(), &c.attrs);
        visit::visit_impl_item_const(self, c);
    }
    fn visit_item_static(&mut self, c: &'ast syn::ItemStatic) {
        self.0.item(&c.ident.to_string(), &c.attrs);
        visit::visit_item_static(self, c);
    }
    fn visit_item_fn(&mut self, f: &'ast syn::ItemFn) {
        self.0.func(&f.sig.ident.to_string(), &f.attrs, &f.sig);
        visit::visit_item_fn(self, f);
    }
    fn visit_impl_item_fn(&mut self, f: &'ast syn::ImplItemFn) {
        self.0.func(&f.sig.ident.to_string(), &f.attrs, &f.sig);
        visit::visit_impl_item_fn(self, f);
    }
    fn visit_trait_item_fn(&mut self, f: &'ast syn::TraitItemFn) {
        self.0.func(&f.sig.ident.to_string(), &f.attrs, &f.sig);
        visit::visit_trait_item_fn(self, f);
    }
}

/// Methods whose result is in the receiver's domains, and whose arguments act
/// in them.
const SAME: [&str; 16] = [
    "clamp",
    "min",
    "max",
    "wrapping_add",
    "wrapping_sub",
    "wrapping_mul",
    "saturating_add",
    "saturating_sub",
    "saturating_mul",
    "abs",
    "checked_add",
    "checked_sub",
    "abs_diff",
    "div_ceil",
    "next_multiple_of",
    "rem_euclid",
];

/// Methods whose result is their receiver's value (a conversion, an unwrap).
const TRANSPARENT: [&str; 10] = [
    "into",
    "try_into",
    "unwrap",
    "expect",
    "unwrap_or",
    "clone",
    "copied",
    "cloned",
    "to_owned",
    "unwrap_or_default",
];

/// Methods whose argument is a shift count in the receiver's domains.
const SHIFTS: [&str; 6] = [
    "wrapping_shl",
    "wrapping_shr",
    "checked_shl",
    "checked_shr",
    "rotate_left",
    "rotate_right",
];

struct Pass<'a> {
    model: &'a Model,
    decls: &'a Decls,
    /// Inferred domains of fields, constants and statics.
    inferred: BTreeMap<String, Ds>,
    next: BTreeMap<String, Ds>,
    /// Locals of the function being read.
    env: Vec<BTreeMap<String, Ds>>,
    /// Domains locals are used in, by (function, local), from the last round
    /// (a literal bound to a local acts in what the local is used with).
    uses: BTreeMap<(String, String), Ds>,
    next_uses: BTreeMap<(String, String), Ds>,
    /// The function being read (for its result and parameters).
    func: Option<String>,
    out: Outcome,
    /// Skip: test code.
    tests: bool,
    /// Whether the literal being read multiplies or divides (a scale).
    scaling: bool,
    /// The array-length domains the function being read declares.
    lens_now: Ds,
    /// Literals that are the width of the type their bytes are read into.
    widths_seen: Vec<Placed>,
    /// The domains every literal of the function being read acts in
    /// (`Domain: body: <d>`: a layout function's offsets).
    body_now: Ds,
}

/// Whether a quantity is a scale: a unit, a scale or a rescale, which only
/// multiplies or divides (or shifts, by its logarithm); it never stands where
/// a value is offset, bounded or masked.
fn is_scale(name: &str) -> bool {
    let core = DERIVED
        .iter()
        .rev()
        .find_map(|x| name.strip_suffix(x))
        .unwrap_or(name);
    core.ends_with("Unit") || core.ends_with("Scale") || core.contains("Per")
}

fn is_test(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| {
        a.path().is_ident("test")
            || (a.path().is_ident("cfg")
                && a.meta
                    .require_list()
                    .is_ok_and(|l| l.tokens.to_string().split_whitespace().any(|t| t == "test")))
    })
}

impl Pass<'_> {
    fn item_ds(&self, name: &str) -> Ds {
        let mut d = self.decls.items.get(name).cloned().unwrap_or_default();
        if let Some(i) = self.inferred.get(name) {
            d.extend(i.iter().cloned());
        }
        d
    }

    fn local(&self, name: &str) -> Option<Ds> {
        self.env.iter().rev().find_map(|s| s.get(name).cloned())
    }

    fn bind(&mut self, name: String, ds: Ds) {
        if let Some(s) = self.env.last_mut() {
            s.insert(name, ds);
        }
    }

    fn store(&mut self, name: &str, ds: &Ds) {
        if !ds.is_empty() {
            self.next
                .entry(name.to_string())
                .or_default()
                .extend(ds.iter().cloned());
        }
    }

    /// The domains declared for where a function's argument `i` goes.
    fn param(&self, f: &str, i: usize, method: bool) -> Ds {
        let Some(names) = self.decls.order.get(f) else {
            return Ds::new();
        };
        let at = if method && names.first().is_some_and(|n| n == "self") {
            i + 1
        } else {
            i
        };
        names
            .get(at)
            .and_then(|p| self.decls.params.get(&(f.to_string(), p.clone())))
            .cloned()
            .unwrap_or_default()
    }

    fn place(&mut self, lit: &syn::LitInt, scope: &Ds, shift: bool) {
        let Ok(v) = lit.base10_parse::<u128>() else {
            return;
        };
        let mut scope = scope.clone();
        scope.extend(self.body_now.iter().cloned());
        let scope = &scope;
        let r = lit.span().byte_range();
        let at = (r.start, r.end, lit.span().start().line);
        if !self.place_value(v, at, lit.suffix(), scope, shift) {
            let ordered = self.ordered(scope);
            let key = (
                format!("{}{v}", if shift { "<<" } else { "" }),
                ordered.join("/"),
            );
            self.out.left.push((at.2, key.0.clone(), key.1.clone()));
            *self.out.residue.entry(key).or_default() += 1;
        }
    }

    fn ordered(&self, scope: &Ds) -> Vec<String> {
        self.decls
            .domains
            .iter()
            .filter(|d| scope.contains(*d))
            .cloned()
            .collect()
    }

    /// Place value `v` at `at` (byte range, line): true when it is a quantity
    /// of a domain in scope (or, as a shift count, a quantity's logarithm; a
    /// `log2 <domain>` in scope holds the logarithms of that domain's).
    fn place_value(
        &mut self,
        v: u128,
        at: (usize, usize, usize),
        suffix: &str,
        scope: &Ds,
        shift: bool,
    ) -> bool {
        if v <= 1 {
            return true;
        }
        for d in self.ordered(scope) {
            let (name, log) = match d.strip_prefix("log2 ") {
                Some(x) => (x.to_string(), !shift),
                None => (d.clone(), shift),
            };
            for (n, q) in self.model.domain(&name) {
                if is_scale(&n) && !(log || self.scaling) {
                    continue;
                }
                let hit = if log {
                    q.is_power_of_two() && u128::from(q.trailing_zeros()) == v
                } else {
                    q == v
                };
                if hit {
                    self.out.placed.push(Placed {
                        range: (at.0, at.1),
                        line: at.2,
                        value: v.to_string(),
                        suffix: suffix.to_string(),
                        name: n,
                        bits: log,
                        raw: None,
                    });
                    return true;
                }
            }
        }
        false
    }

    /// The domains of `e`; `want` is the domains of where its value goes.
    fn ex(&mut self, e: &syn::Expr, want: &Ds) -> Ds {
        use syn::Expr as E;
        if !matches!(e, E::Lit(_) | E::Unary(_)) && self.scaling {
            self.scaling = false;
            let d = self.ex(e, want);
            self.scaling = true;
            return d;
        }
        match e {
            E::Lit(l) => {
                if let syn::Lit::Int(i) = &l.lit {
                    self.place(i, want, false);
                }
                Ds::new()
            }
            E::Path(p) => {
                if let Some(i) = p.path.get_ident() {
                    let n = i.to_string();
                    if let Some(d) = self.local(&n) {
                        return d;
                    }
                    return self.item_ds(&n);
                }
                p.path
                    .segments
                    .last()
                    .map(|s| self.item_ds(&s.ident.to_string()))
                    .unwrap_or_default()
            }
            E::Field(f) => {
                self.ex(&f.base, &Ds::new());
                match &f.member {
                    syn::Member::Named(i) => self.item_ds(&i.to_string()),
                    syn::Member::Unnamed(_) => Ds::new(),
                }
            }
            E::Index(i) => {
                let d = self.ex(&i.expr, &Ds::new());
                self.ex(&i.index, &Ds::new());
                d
            }
            E::Paren(p) => self.ex(&p.expr, want),
            E::Group(g) => self.ex(&g.expr, want),
            E::Cast(c) => self.ex(&c.expr, want),
            E::Reference(r) => self.ex(&r.expr, want),
            E::Unary(u) => match u.op {
                syn::UnOp::Neg(_) => self.ex(&u.expr, want),
                syn::UnOp::Deref(_) => self.ex(&u.expr, want),
                _ => {
                    self.ex(&u.expr, &Ds::new());
                    Ds::new()
                }
            },
            E::Binary(b) => self.binary(b, want),
            E::MethodCall(m) => {
                let name = m.method.to_string();
                if SAME.contains(&name.as_str()) {
                    let mut scope = self.peek(&m.receiver);
                    for a in &m.args {
                        scope.extend(self.peek(a));
                    }
                    scope.extend(want.iter().cloned());
                    let r = self.ex(&m.receiver, &scope);
                    for a in &m.args {
                        self.ex(a, &scope);
                    }
                    let mut out = r;
                    for a in &m.args {
                        out.extend(self.peek(a));
                    }
                    return out;
                }
                if SHIFTS.contains(&name.as_str()) {
                    let mut scope = self.peek(&m.receiver);
                    scope.extend(want.iter().cloned());
                    let r = self.ex(&m.receiver, want);
                    for a in &m.args {
                        self.shift_count(a, &scope);
                    }
                    return r;
                }
                if TRANSPARENT.contains(&name.as_str()) {
                    let r = self.ex(&m.receiver, want);
                    for a in &m.args {
                        self.ex(a, &r);
                    }
                    return r;
                }
                let r = self.ex(&m.receiver, &Ds::new());
                for (i, a) in m.args.iter().enumerate() {
                    let p = self.param(&name, i, true);
                    self.used(a, &p);
                    self.ex(a, &p);
                }
                let _ = r;
                self.decls.results.get(&name).cloned().unwrap_or_default()
            }
            E::Call(c) => {
                let name = match &*c.func {
                    E::Path(p) => p.path.segments.last().map(|s| s.ident.to_string()),
                    _ => None,
                };
                if let (Some(n), E::Path(p)) = (name.as_deref(), &*c.func) {
                    if (n == "from_le_bytes" || n == "from_be_bytes") && p.path.segments.len() == 2
                    {
                        let ty = p.path.segments[0].ident.to_string();
                        for a in &c.args {
                            self.widths(a, &ty);
                        }
                    }
                }
                // a conversion carries its argument's domains
                if name
                    .as_deref()
                    .is_some_and(|n| n == "from" || n == "try_from")
                    && c.args.len() == 1
                {
                    return self.ex(&c.args[0], want);
                }
                // a local closure: the domains of its body
                if let E::Path(p) = &*c.func {
                    if let Some(local) = p.path.get_ident().and_then(|i| self.local(&i.to_string()))
                    {
                        for a in &c.args {
                            self.ex(a, &Ds::new());
                        }
                        return local;
                    }
                }
                if name.is_none() {
                    self.ex(&c.func, &Ds::new());
                }
                for (i, a) in c.args.iter().enumerate() {
                    let p = name
                        .as_deref()
                        .map(|n| self.param(n, i, false))
                        .unwrap_or_default();
                    self.used(a, &p);
                    self.ex(a, &p);
                }
                name.and_then(|n| self.decls.results.get(&n).cloned())
                    .unwrap_or_default()
            }
            E::Assign(a) => {
                let target = self.ex(&a.left, &Ds::new());
                let d = self.ex(&a.right, &target);
                self.assign(&a.left, &d);
                Ds::new()
            }
            E::Block(b) => self.block(&b.block, want),
            E::Unsafe(b) => self.block(&b.block, want),
            E::If(i) => {
                self.ex(&i.cond, &Ds::new());
                let mut d = self.block(&i.then_branch, want);
                if let Some((_, e)) = &i.else_branch {
                    d.extend(self.ex(e, want));
                }
                d
            }
            E::Match(m) => {
                let s = self.ex(&m.expr, &Ds::new());
                let mut d = Ds::new();
                for arm in &m.arms {
                    self.env.push(BTreeMap::new());
                    self.pat(&arm.pat, &s);
                    d.extend(self.ex(&arm.body, want));
                    self.env.pop();
                }
                d
            }
            E::Array(a) if self.table(a) => Ds::new(),
            E::Array(a) => {
                let mut scope = want.clone();
                for x in &a.elems {
                    scope.extend(self.peek(x));
                }
                let mut d = Ds::new();
                for x in &a.elems {
                    d.extend(self.ex(x, &scope));
                }
                d
            }
            E::Repeat(r) => {
                let d = self.ex(&r.expr, want);
                self.ex(&r.len, want);
                d
            }
            E::Tuple(t) => {
                // a tuple's parts bound together (`let (a, b) = (x, y)`) share its domains
                let mut d = Ds::new();
                for x in &t.elems {
                    d.extend(self.ex(x, &Ds::new()));
                }
                d
            }
            E::Range(r) => {
                let mut scope = want.clone();
                for x in r.start.iter().chain(r.end.iter()) {
                    scope.extend(self.peek(x));
                }
                let mut d = Ds::new();
                for x in r.start.iter().chain(r.end.iter()) {
                    d.extend(self.ex(x, &scope));
                }
                d
            }
            E::Return(r) => {
                let w = self
                    .func
                    .as_ref()
                    .and_then(|f| self.decls.results.get(f).cloned())
                    .unwrap_or_default();
                if let Some(x) = &r.expr {
                    self.ex(x, &w);
                }
                Ds::new()
            }
            E::Closure(c) => {
                self.env.push(BTreeMap::new());
                let d = self.ex(&c.body, &Ds::new());
                self.env.pop();
                d
            }
            E::Loop(l) => {
                self.block(&l.body, &Ds::new());
                Ds::new()
            }
            E::While(w) => {
                self.ex(&w.cond, &Ds::new());
                self.block(&w.body, &Ds::new());
                Ds::new()
            }
            E::ForLoop(f) => {
                let d = self.ex(&f.expr, &Ds::new());
                self.env.push(BTreeMap::new());
                self.pat(&f.pat, &d);
                self.block(&f.body, &Ds::new());
                self.env.pop();
                Ds::new()
            }
            E::Let(l) => {
                let d = self.ex(&l.expr, &Ds::new());
                self.pat(&l.pat, &d);
                Ds::new()
            }
            E::Struct(s) => {
                for f in &s.fields {
                    let w = match &f.member {
                        syn::Member::Named(i) => self.item_ds(&i.to_string()),
                        syn::Member::Unnamed(_) => Ds::new(),
                    };
                    self.used(&f.expr, &w);
                    // a field's array value: its length acts in the field's length domains
                    let d = match (&f.expr, &f.member) {
                        (E::Repeat(r), syn::Member::Named(i)) => {
                            let l = self
                                .decls
                                .lens
                                .get(&i.to_string())
                                .cloned()
                                .unwrap_or_else(|| w.clone());
                            self.ex(&r.len, &l);
                            self.ex(&r.expr, &w)
                        }
                        _ => self.ex(&f.expr, &w),
                    };
                    if let syn::Member::Named(i) = &f.member {
                        self.store(&i.to_string(), &d);
                    }
                }
                if let Some(r) = &s.rest {
                    self.ex(r, &Ds::new());
                }
                Ds::new()
            }
            E::Try(t) => self.ex(&t.expr, want),
            E::Break(b) => {
                if let Some(x) = &b.expr {
                    self.ex(x, &Ds::new());
                }
                Ds::new()
            }
            E::Const(c) => self.block(&c.block, want),
            _ => {
                visit::visit_expr(&mut Inner(self), e);
                Ds::new()
            }
        }
    }

    /// An array of numerals that is a constant table of the model, read as
    /// cells: placed whole.
    fn table(&mut self, a: &syn::ExprArray) -> bool {
        let mut vs = Vec::new();
        for e in &a.elems {
            match e {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Int(i),
                    ..
                }) => match i.base10_parse::<u128>() {
                    Ok(v) => vs.push(v),
                    Err(_) => return false,
                },
                _ => return false,
            }
        }
        if vs.len() < 2 {
            return false;
        }
        let Some((name, _)) = self.model.tables.iter().find(|(_, c)| *c == vs) else {
            return false;
        };
        let r = a.bracket_token.span.join().byte_range();
        self.out.placed.push(Placed {
            range: (r.start, r.end),
            line: a.bracket_token.span.join().start().line,
            value: format!("[{} cells]", vs.len()),
            suffix: String::new(),
            name: name.clone(),
            bits: false,
            raw: None,
        });
        true
    }

    /// The literals of `e` that equal the width of `ty` (the integer the bytes
    /// are read into by `ty::from_le_bytes` / `from_be_bytes`): they are that
    /// width, `size_of::<ty>()`.
    fn widths(&mut self, e: &syn::Expr, ty: &str) {
        let w: u128 = match ty {
            "u16" | "i16" => 2,
            "u32" | "i32" => 4,
            "u64" | "i64" => 8,
            "u128" | "i128" => 16,
            _ => return,
        };
        type Lit = (Option<u128>, bool, std::ops::Range<usize>, usize);
        struct Lits<'v>(&'v mut Vec<Lit>);
        impl<'ast> Visit<'ast> for Lits<'_> {
            fn visit_lit_int(&mut self, i: &'ast syn::LitInt) {
                self.0.push((
                    i.base10_parse::<u128>().ok(),
                    i.suffix().is_empty(),
                    i.span().byte_range(),
                    i.span().start().line,
                ));
            }
        }
        let mut lits = Vec::new();
        Lits(&mut lits).visit_expr(e);
        for (v, plain, r, line) in lits {
            if v == Some(w) && plain {
                self.widths_seen.push(Placed {
                    range: (r.start, r.end),
                    line,
                    value: w.to_string(),
                    suffix: String::new(),
                    name: format!("size_of::<{ty}>"),
                    bits: false,
                    raw: Some(format!("std::mem::size_of::<{ty}>()")),
                });
            }
        }
    }

    /// The domains of `e` without placing anything.
    fn peek(&mut self, e: &syn::Expr) -> Ds {
        let widths = self.widths_seen.len();
        let saved = (
            self.out.placed.len(),
            self.out.residue.clone(),
            self.next.clone(),
            self.out.left.len(),
        );
        self.env.push(BTreeMap::new());
        let d = self.ex(e, &Ds::new());
        self.env.pop();
        self.out.placed.truncate(saved.0);
        self.out.residue = saved.1;
        self.next = saved.2;
        self.out.left.truncate(saved.3);
        self.widths_seen.truncate(widths);
        d
    }

    fn shift_count(&mut self, e: &syn::Expr, scope: &Ds) {
        match e {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(i),
                ..
            }) => self.place(i, scope, true),
            syn::Expr::Paren(p) => self.shift_count(&p.expr, scope),
            syn::Expr::Cast(c) => self.shift_count(&c.expr, scope),
            syn::Expr::Binary(b) if matches!(b.op, syn::BinOp::Add(_) | syn::BinOp::Sub(_)) => {
                // a count less another (`bits - 12`): both are counts
                self.shift_count(&b.left, scope);
                self.shift_count(&b.right, scope);
            }
            _ => {
                self.ex(e, &Ds::new());
            }
        }
    }

    /// Read an operand: the scaling flag holds for a literal operand only; a
    /// compound operand is read in its own operators' roles.
    fn ex_operand(&mut self, e: &syn::Expr, scope: &Ds, outer: bool) -> Ds {
        let lit = matches!(e, syn::Expr::Lit(_))
            || matches!(e, syn::Expr::Unary(u) if matches!(&*u.expr, syn::Expr::Lit(_)));
        if lit {
            return self.ex(e, scope);
        }
        let role = self.scaling;
        self.scaling = false;
        let d = self.ex(e, scope);
        self.scaling = role;
        let _ = outer;
        d
    }

    /// Note that a local operand acts in `scope`.
    fn used(&mut self, e: &syn::Expr, scope: &Ds) {
        let e = match e {
            syn::Expr::Paren(p) => &*p.expr,
            syn::Expr::Cast(c) => &*c.expr,
            syn::Expr::Unary(u) => &*u.expr,
            x => x,
        };
        if let syn::Expr::Path(p) = e {
            if let (Some(i), Some(f)) = (p.path.get_ident(), self.func.clone()) {
                let n = i.to_string();
                if self.local(&n).is_some() && !scope.is_empty() {
                    self.next_uses
                        .entry((f, n))
                        .or_default()
                        .extend(scope.iter().cloned());
                }
            }
        }
    }

    fn binary(&mut self, b: &syn::ExprBinary, want: &Ds) -> Ds {
        use syn::BinOp as B;
        match b.op {
            B::Shl(_) => {
                // `1 << k`: the power of two itself, when it is a quantity in scope
                if let (
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Int(one),
                        ..
                    }),
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Int(kl),
                        ..
                    }),
                ) = (&*b.left, &*b.right)
                {
                    if let (Ok(1), Ok(k)) = (one.base10_parse::<u128>(), kl.base10_parse::<u32>()) {
                        let span = one.span().join(kl.span()).unwrap_or_else(|| one.span());
                        let r = span.byte_range();
                        if k < 127
                            && self.place_value(
                                1u128 << k,
                                (r.start, r.end, span.start().line),
                                one.suffix(),
                                want,
                                false,
                            )
                        {
                            return Ds::new();
                        }
                    }
                }
                self.shift(b, want)
            }
            B::Shr(_) | B::ShlAssign(_) | B::ShrAssign(_) => self.shift(b, want),
            _ => self.binary_rest(b, want),
        }
    }

    fn shift(&mut self, b: &syn::ExprBinary, want: &Ds) -> Ds {
        let mut scope = self.peek(&b.left);
        scope.extend(want.iter().cloned());
        let d = self.ex(&b.left, want);
        self.shift_count(&b.right, &scope);
        d
    }

    fn binary_rest(&mut self, b: &syn::ExprBinary, want: &Ds) -> Ds {
        use syn::BinOp as B;
        match b.op {
            B::And(_) | B::Or(_) => {
                self.ex(&b.left, &Ds::new());
                self.ex(&b.right, &Ds::new());
                Ds::new()
            }
            B::Eq(_) | B::Lt(_) | B::Le(_) | B::Ne(_) | B::Ge(_) | B::Gt(_) => {
                let mut scope = self.peek(&b.left);
                scope.extend(self.peek(&b.right));
                self.used(&b.left, &scope);
                self.used(&b.right, &scope);
                self.ex(&b.left, &scope);
                self.ex(&b.right, &scope);
                Ds::new()
            }
            B::AddAssign(_)
            | B::SubAssign(_)
            | B::MulAssign(_)
            | B::DivAssign(_)
            | B::RemAssign(_)
            | B::BitAndAssign(_)
            | B::BitOrAssign(_)
            | B::BitXorAssign(_) => {
                let mut scope = self.peek(&b.left);
                scope.extend(self.peek(&b.right));
                self.ex(&b.left, &scope);
                let d = self.ex(&b.right, &scope);
                self.assign(&b.left, &d);
                Ds::new()
            }
            _ => {
                let mut scope = self.peek(&b.left);
                scope.extend(self.peek(&b.right));
                scope.extend(want.iter().cloned());
                self.used(&b.left, &scope);
                self.used(&b.right, &scope);
                let saved = self.scaling;
                self.scaling = matches!(b.op, B::Mul(_) | B::Div(_) | B::Rem(_));
                let mut d = self.ex_operand(&b.left, &scope, saved);
                d.extend(self.ex_operand(&b.right, &scope, saved));
                self.scaling = saved;
                d
            }
        }
    }

    fn assign(&mut self, target: &syn::Expr, d: &Ds) {
        match target {
            syn::Expr::Path(p) => {
                if let Some(i) = p.path.get_ident() {
                    let n = i.to_string();
                    if self.local(&n).is_some() {
                        let mut ds = self.local(&n).unwrap_or_default();
                        ds.extend(d.iter().cloned());
                        for s in self.env.iter_mut().rev() {
                            if let Some(x) = s.get_mut(&n) {
                                *x = ds;
                                break;
                            }
                        }
                    } else {
                        self.store(&n, d);
                    }
                }
            }
            syn::Expr::Field(f) => {
                if let syn::Member::Named(i) = &f.member {
                    self.store(&i.to_string(), d);
                }
            }
            syn::Expr::Index(i) => self.assign(&i.expr, d),
            syn::Expr::Unary(u) => self.assign(&u.expr, d),
            syn::Expr::Paren(p) => self.assign(&p.expr, d),
            _ => {}
        }
    }

    fn pat(&mut self, p: &syn::Pat, d: &Ds) {
        match p {
            syn::Pat::Ident(i) => {
                self.bind(i.ident.to_string(), d.clone());
                if let Some((_, s)) = &i.subpat {
                    self.pat(s, d);
                }
            }
            syn::Pat::Type(t) => self.pat(&t.pat, d),
            syn::Pat::Reference(r) => self.pat(&r.pat, d),
            syn::Pat::Tuple(t) => {
                for x in &t.elems {
                    self.pat(x, d);
                }
            }
            syn::Pat::Slice(s) => {
                for x in &s.elems {
                    self.pat(x, d);
                }
            }
            syn::Pat::TupleStruct(t) => {
                for x in &t.elems {
                    self.pat(x, d);
                }
            }
            syn::Pat::Struct(s) => {
                for f in &s.fields {
                    let w = match &f.member {
                        syn::Member::Named(i) => self.item_ds(&i.to_string()),
                        syn::Member::Unnamed(_) => Ds::new(),
                    };
                    self.pat(&f.pat, &w);
                }
            }
            syn::Pat::Or(o) => {
                for x in &o.cases {
                    self.pat(x, d);
                }
            }
            syn::Pat::Lit(l) => {
                if let syn::Lit::Int(i) = &l.lit {
                    self.place(i, d, false);
                }
            }
            syn::Pat::Range(r) => {
                for x in r.start.iter().chain(r.end.iter()) {
                    self.ex(x, d);
                }
            }
            _ => {}
        }
    }

    /// A type's array lengths: each acts in `ds` (a table's extent is its
    /// domain's).
    fn ty(&mut self, t: &syn::Type, ds: &Ds) {
        match t {
            syn::Type::Array(a) => {
                self.ty(&a.elem, ds);
                self.ex(&a.len, ds);
            }
            syn::Type::Reference(r) => self.ty(&r.elem, ds),
            syn::Type::Paren(p) => self.ty(&p.elem, ds),
            syn::Type::Slice(s) => self.ty(&s.elem, ds),
            syn::Type::Tuple(t) => {
                for e in &t.elems {
                    self.ty(e, ds);
                }
            }
            syn::Type::Path(p) => {
                for seg in &p.path.segments {
                    if let syn::PathArguments::AngleBracketed(a) = &seg.arguments {
                        for g in &a.args {
                            match g {
                                syn::GenericArgument::Type(t) => self.ty(t, ds),
                                syn::GenericArgument::Const(e) => {
                                    self.ex(e, &Ds::new());
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn block(&mut self, b: &syn::Block, want: &Ds) -> Ds {
        self.env.push(BTreeMap::new());
        let mut d = Ds::new();
        let n = b.stmts.len();
        for (i, s) in b.stmts.iter().enumerate() {
            match s {
                syn::Stmt::Local(l) => {
                    if let syn::Pat::Type(t) = &l.pat {
                        let lens = self.lens_now.clone();
                        self.ty(&t.ty, &lens);
                    }
                    let used = match &l.pat {
                        syn::Pat::Ident(p) => self
                            .func
                            .as_ref()
                            .and_then(|f| self.uses.get(&(f.clone(), p.ident.to_string())))
                            .cloned()
                            .unwrap_or_default(),
                        syn::Pat::Type(t) => match &*t.pat {
                            syn::Pat::Ident(p) => self
                                .func
                                .as_ref()
                                .and_then(|f| self.uses.get(&(f.clone(), p.ident.to_string())))
                                .cloned()
                                .unwrap_or_default(),
                            _ => Ds::new(),
                        },
                        _ => Ds::new(),
                    };
                    let v = l
                        .init
                        .as_ref()
                        .map(|i| {
                            // a local's uses reach its value only when the value is a
                            // numeral or a choice of numerals, not through arithmetic
                            let direct = matches!(
                                &*i.expr,
                                syn::Expr::Lit(_)
                                    | syn::Expr::Repeat(_)
                                    | syn::Expr::Array(_)
                                    | syn::Expr::If(_)
                                    | syn::Expr::Match(_)
                                    | syn::Expr::Unary(_)
                                    | syn::Expr::Block(_)
                            );
                            let w = if direct { used.clone() } else { Ds::new() };
                            let v = self.ex(&i.expr, &w);
                            if let Some((_, e)) = &i.diverge {
                                self.ex(e, &Ds::new());
                            }
                            v
                        })
                        .unwrap_or_default();
                    self.pat(&l.pat, &v);
                }
                syn::Stmt::Expr(e, semi) => {
                    let w = if i + 1 == n && semi.is_none() {
                        want.clone()
                    } else {
                        Ds::new()
                    };
                    let x = self.ex(e, &w);
                    if i + 1 == n && semi.is_none() {
                        d = x;
                    }
                }
                syn::Stmt::Item(it) => self.item(it),
                syn::Stmt::Macro(_) => {}
            }
        }
        self.env.pop();
        d
    }

    fn func(&mut self, name: &str, sig: &syn::Signature, body: &syn::Block) {
        let saved = self.func.replace(name.to_string());
        self.env.push(BTreeMap::new());
        let result = self.decls.results.get(name).cloned().unwrap_or_default();
        // the signature's array lengths: its `len:` declaration, else the result's domains
        let lens = self.decls.lens.get(&format!("fn {name}")).cloned();
        let saved_lens = std::mem::replace(&mut self.lens_now, lens.clone().unwrap_or_default());
        let body_ds = self
            .decls
            .lens
            .get(&format!("body {name}"))
            .cloned()
            .unwrap_or_default();
        let saved_body = std::mem::replace(&mut self.body_now, body_ds);
        if let syn::ReturnType::Type(_, t) = &sig.output {
            self.ty(t, lens.as_ref().unwrap_or(&result));
        }
        for i in &sig.inputs {
            if let syn::FnArg::Typed(t) = i {
                let pd = match &*t.pat {
                    syn::Pat::Ident(p) => self
                        .decls
                        .params
                        .get(&(name.to_string(), p.ident.to_string()))
                        .cloned()
                        .unwrap_or_default(),
                    _ => Ds::new(),
                };
                let mut l = lens.clone().unwrap_or_default();
                l.extend(pd.iter().cloned());
                self.ty(&t.ty, &l);
                if let syn::Pat::Ident(p) = &*t.pat {
                    let n = p.ident.to_string();
                    let d = self
                        .decls
                        .params
                        .get(&(name.to_string(), n.clone()))
                        .cloned()
                        .unwrap_or_default();
                    self.bind(n, d);
                }
            }
        }
        let want = self.decls.results.get(name).cloned().unwrap_or_default();
        self.block(body, &want);
        self.env.pop();
        self.func = saved;
        self.lens_now = saved_lens;
        self.body_now = saved_body;
    }

    fn item(&mut self, it: &syn::Item) {
        match it {
            syn::Item::Fn(f) if !is_test(&f.attrs) => {
                self.func(&f.sig.ident.to_string(), &f.sig, &f.block)
            }
            syn::Item::Impl(i) if !is_test(&i.attrs) => {
                for x in &i.items {
                    match x {
                        syn::ImplItem::Fn(f) if !is_test(&f.attrs) => {
                            self.func(&f.sig.ident.to_string(), &f.sig, &f.block)
                        }
                        syn::ImplItem::Const(c) => {
                            let w = self.item_ds(&c.ident.to_string());
                            let d = self.ex(&c.expr, &w);
                            self.store(&c.ident.to_string(), &d);
                        }
                        _ => {}
                    }
                }
            }
            syn::Item::Trait(t) => {
                for x in &t.items {
                    if let syn::TraitItem::Fn(f) = x {
                        match &f.default {
                            Some(b) => self.func(&f.sig.ident.to_string(), &f.sig, b),
                            None => self.func(
                                &f.sig.ident.to_string(),
                                &f.sig,
                                &syn::Block {
                                    brace_token: Default::default(),
                                    stmts: Vec::new(),
                                },
                            ),
                        }
                    }
                }
            }
            syn::Item::Mod(m) if !is_test(&m.attrs) => {
                if let Some((_, items)) = &m.content {
                    for x in items {
                        self.item(x);
                    }
                }
            }
            syn::Item::Struct(st) if !is_test(&st.attrs) => {
                let own = st.ident.to_string();
                for f in &st.fields {
                    // a named field's own declaration, an unnamed one's its struct's
                    let key = f.ident.as_ref().map_or(own.clone(), |i| i.to_string());
                    let w = self
                        .decls
                        .lens
                        .get(&key)
                        .cloned()
                        .unwrap_or_else(|| self.item_ds(&key));
                    self.ty(&f.ty, &w);
                }
            }
            syn::Item::Const(c) => {
                let w = self.item_ds(&c.ident.to_string());
                let l = self
                    .decls
                    .lens
                    .get(&c.ident.to_string())
                    .cloned()
                    .unwrap_or_else(|| w.clone());
                self.ty(&c.ty, &l);
                let d = self.ex(&c.expr, &w);
                self.store(&c.ident.to_string(), &d);
            }
            syn::Item::Static(c) => {
                let w = self.item_ds(&c.ident.to_string());
                self.ty(&c.ty, &w);
                let d = self.ex(&c.expr, &w);
                self.store(&c.ident.to_string(), &d);
            }
            _ => {}
        }
        let _ = self.tests;
    }
}

/// Visits the sub-expressions of an expression kind the pass does not type.
struct Inner<'p, 'a>(&'p mut Pass<'a>);

impl<'ast> Visit<'ast> for Inner<'_, '_> {
    fn visit_expr(&mut self, e: &'ast syn::Expr) {
        self.0.ex(e, &Ds::new());
    }
}

/// Run the pass over one file's source.
pub fn pass(model: &Model, text: &str, others: &[String]) -> Result<Outcome, crate::Error> {
    let ast = syn::parse_file(text).map_err(|e| e.to_string())?;
    let mut decls = Decls::default();
    Collect(&mut decls).visit_file(&ast);
    // the declarations of the other files read together (a module's parent and children)
    for o in others {
        if let Ok(f) = syn::parse_file(o) {
            Collect(&mut decls).visit_file(&f);
        }
    }
    let mut p = Pass {
        model,
        decls: &decls,
        inferred: BTreeMap::new(),
        next: BTreeMap::new(),
        uses: BTreeMap::new(),
        next_uses: BTreeMap::new(),
        env: Vec::new(),
        func: None,
        out: Outcome::default(),
        tests: false,
        scaling: false,
        lens_now: Ds::new(),
        body_now: Ds::new(),
        widths_seen: Vec::new(),
    };
    for _ in 0..32 {
        p.next = BTreeMap::new();
        p.next_uses = BTreeMap::new();
        p.out = Outcome::default();
        p.widths_seen.clear();
        for it in &ast.items {
            p.item(it);
        }
        let mut merged = p.inferred.clone();
        for (k, v) in &p.next {
            merged
                .entry(k.clone())
                .or_default()
                .extend(v.iter().cloned());
        }
        let mut uses = p.uses.clone();
        for (k, v) in &p.next_uses {
            uses.entry(k.clone()).or_default().extend(v.iter().cloned());
        }
        let settled = merged == p.inferred && uses == p.uses;
        p.uses = uses;
        if settled {
            let mut ws = std::mem::take(&mut p.widths_seen);
            ws.retain(|w| !p.out.placed.iter().any(|x| x.range == w.range));
            p.out.placed.extend(ws);
            // a width's site was also counted as residue: drop it there
            p.out.placed.sort_by_key(|x| x.range);
            p.out.placed.dedup_by_key(|x| x.range);
            return Ok(p.out);
        }
        p.inferred = merged;
    }
    Err("domains: no fixpoint".to_string())
}

/// The integer views a literal of unknown type is tried in, in the order the
/// compiler is asked.
pub const VIEWS: [&str; 9] = [
    "i32", "u32", "usize", "u64", "i64", "u16", "i16", "u8", "i128",
];

/// Render a placed literal in the view `ty`.
pub fn site(model: &Model, p: &Placed, ty: &str) -> String {
    if let Some(r) = &p.raw {
        return r.clone();
    }
    if p.bits {
        format!("{}::{ty}::bits::{}", model.path, p.name)
    } else {
        format!("{}::{ty}::{}", model.path, p.name)
    }
}

/// One rewritten site under the compiler's judgement.
#[derive(Clone, Debug)]
struct Site {
    file: usize,
    placed: Placed,
    /// The views tried so far, the current last.
    tried: Vec<String>,
    /// Left as a numeral: no view the compiler admits.
    dropped: bool,
}

fn render(model: &Model, text: &str, sites: &[&Site]) -> (String, Vec<(usize, usize)>) {
    let mut out = String::with_capacity(text.len());
    let mut at = 0;
    let mut ranges = Vec::new();
    for s in sites {
        let (a, b) = s.placed.range;
        out.push_str(&text[at..a]);
        let start = out.len();
        if s.dropped {
            out.push_str(&text[a..b]);
        } else {
            out.push_str(&site(
                model,
                &s.placed,
                s.tried.last().map_or("i32", String::as_str),
            ));
        }
        ranges.push((start, out.len()));
        at = b;
    }
    out.push_str(&text[at..]);
    (out, ranges)
}

/// A compiler error: file, byte range, and every type named in it.
#[derive(Debug)]
struct Diagnostic {
    file: String,
    range: (usize, usize),
    line: usize,
    types: Vec<String>,
}

fn types_in(s: &str) -> Vec<String> {
    s.split('`')
        .skip(1)
        .step_by(2)
        .filter(|t| VIEWS.contains(t) || *t == "{integer}")
        .map(str::to_string)
        .collect()
}

fn diagnostics(root: &Path, packages: &[&str]) -> Result<Vec<Diagnostic>, crate::Error> {
    let mut cmd = std::process::Command::new("cargo");
    cmd.current_dir(root).args([
        "check",
        "--locked",
        "--quiet",
        "--message-format=json",
        "--all-targets",
    ]);
    for p in packages {
        cmd.args(["-p", p]);
    }
    let out = cmd.output().map_err(|e| format!("cargo check: {e}"))?;
    let mut ds = Vec::new();
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        if v["reason"] != "compiler-message" || v["message"]["level"] != "error" {
            continue;
        }
        let m = &v["message"];
        let mut text = m["message"].as_str().unwrap_or_default().to_string();
        for sp in m["spans"].as_array().into_iter().flatten() {
            text.push(' ');
            text.push_str(sp["label"].as_str().unwrap_or_default());
        }
        for c in m["children"].as_array().into_iter().flatten() {
            text.push(' ');
            text.push_str(c["message"].as_str().unwrap_or_default());
        }
        for sp in m["spans"].as_array().into_iter().flatten() {
            if sp["is_primary"] != true {
                continue;
            }
            ds.push(Diagnostic {
                file: sp["file_name"].as_str().unwrap_or_default().to_string(),
                range: (
                    sp["byte_start"].as_u64().unwrap_or(0) as usize,
                    sp["byte_end"].as_u64().unwrap_or(0) as usize,
                ),
                line: sp["line_start"].as_u64().unwrap_or(0) as usize,
                types: types_in(&text),
            });
        }
        if ds.is_empty() || m["spans"].as_array().is_none_or(Vec::is_empty) {
            return Err(format!("cargo check: {text}"));
        }
    }
    if !out.status.success() && ds.is_empty() {
        return Err(format!(
            "cargo check failed without a diagnostic:\n{}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(ds)
}

/// The report of a rewrite.
#[derive(Debug, Default)]
pub struct Report {
    /// Sites restated, by file.
    pub restated: BTreeMap<String, usize>,
    /// Sites placed but left as numerals (no view the compiler admits).
    pub dropped: usize,
    /// Residue, by file, (value, scope).
    pub residue: BTreeMap<String, BTreeMap<(String, String), u64>>,
    /// Placed sites, by file: (line, value, quantity).
    pub placed: BTreeMap<String, Vec<(usize, String, String)>>,
    /// Residue sites, by file: (line, value, scope).
    pub left: BTreeMap<String, Vec<(usize, String, String)>>,
}

/// Rewrite `files` (repository-relative), each reading the generated module
/// `generated`, under the compiler's judgement of `packages`.
pub fn rewrite(
    root: &Path,
    generated: &str,
    files: &[String],
    packages: &[&str],
    dry: bool,
) -> Result<Report, crate::Error> {
    let model = Model::load(root, generated)?;
    let mut texts = Vec::new();
    let mut sites = Vec::new();
    let mut report = Report::default();
    let all: Vec<String> = files
        .iter()
        .map(|f| std::fs::read_to_string(root.join(f)).map_err(|e| format!("{f}: {e}")))
        .collect::<Result<_, _>>()?;
    for (i, f) in files.iter().enumerate() {
        let text = all[i].clone();
        let others: Vec<String> = all
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, t)| t.clone())
            .collect();
        let o = pass(&model, &text, &others).map_err(|e| format!("{f}: {e}"))?;
        report.residue.insert(f.clone(), o.residue);
        let mut left = o.left;
        left.sort();
        left.dedup();
        report.left.insert(f.clone(), left);
        report.placed.insert(
            f.clone(),
            o.placed
                .iter()
                .map(|p| {
                    (
                        p.line,
                        p.value.clone(),
                        format!("{}{}", if p.bits { "bits::" } else { "" }, p.name),
                    )
                })
                .collect(),
        );
        for p in o.placed {
            let first = if p.suffix.is_empty() {
                "i32".to_string()
            } else {
                p.suffix.clone()
            };
            sites.push(Site {
                file: i,
                placed: p,
                tried: vec![first],
                dropped: false,
            });
        }
        texts.push(text);
    }
    if dry {
        for s in &sites {
            *report.restated.entry(files[s.file].clone()).or_default() += 1;
        }
        return Ok(report);
    }
    let originals = texts.clone();
    let write = |sites: &[Site]| -> Result<Vec<Vec<(usize, usize)>>, crate::Error> {
        let mut all = Vec::new();
        for (i, f) in files.iter().enumerate() {
            let mine: Vec<&Site> = sites.iter().filter(|s| s.file == i).collect();
            let (text, ranges) = render(&model, &texts[i], &mine);
            std::fs::write(root.join(f), text).map_err(|e| format!("{f}: {e}"))?;
            all.push(ranges);
        }
        Ok(all)
    };
    let restore = || {
        for (i, f) in files.iter().enumerate() {
            let _ = std::fs::write(root.join(f), &originals[i]);
        }
    };
    for _round in 0..(VIEWS.len() + 2) {
        let ranges = write(&sites)?;
        let ds = match diagnostics(root, packages) {
            Ok(d) => d,
            Err(e) => {
                restore();
                return Err(e);
            }
        };
        if ds.is_empty() {
            for f in files {
                let _ = std::process::Command::new("rustfmt")
                    .current_dir(root)
                    .args(["--edition", "2021", f])
                    .status();
            }
            for s in sites.iter().filter(|s| !s.dropped) {
                *report.restated.entry(files[s.file].clone()).or_default() += 1;
            }
            report.dropped = sites.iter().filter(|s| s.dropped).count();
            return Ok(report);
        }
        // index sites by (file, rendered range)
        let mut hit = vec![false; sites.len()];
        let mut want: Vec<Option<String>> = vec![None; sites.len()];
        let mut stray = Vec::new();
        for d in &ds {
            let Some(fi) = files.iter().position(|f| d.file.ends_with(f.as_str())) else {
                stray.push(d);
                continue;
            };
            let mut any = false;
            let text_lines: Vec<usize> = Vec::new();
            let _ = text_lines;
            for (k, s) in sites.iter().enumerate() {
                if s.file != fi || s.dropped {
                    continue;
                }
                let idx = sites[..k].iter().filter(|x| x.file == fi).count();
                let r = ranges[fi][idx];
                let overlaps = r.0 < d.range.1 && d.range.0 < r.1;
                if overlaps {
                    any = true;
                    hit[k] = true;
                    let cur = s.tried.last().cloned().unwrap_or_default();
                    want[k] = d
                        .types
                        .iter()
                        .find(|t| **t != cur && *t != "{integer}")
                        .cloned();
                }
            }
            if !any {
                // an operator's error (`cannot add`): the sites on its line
                let line_of =
                    |text: &str, at: usize| text[..at.min(text.len())].matches('\n').count() + 1;
                let rendered = std::fs::read_to_string(root.join(&files[fi])).unwrap_or_default();
                for (k, s) in sites.iter().enumerate() {
                    if s.file != fi || s.dropped {
                        continue;
                    }
                    let idx = sites[..k].iter().filter(|x| x.file == fi).count();
                    if line_of(&rendered, ranges[fi][idx].0) == d.line {
                        any = true;
                        hit[k] = true;
                        let cur = s.tried.last().cloned().unwrap_or_default();
                        if want[k].is_none() {
                            want[k] = d
                                .types
                                .iter()
                                .find(|t| **t != cur && *t != "{integer}")
                                .cloned();
                        }
                    }
                }
            }
            if !any {
                stray.push(d);
            }
        }
        let mut retyped_now: Vec<usize> = Vec::new();
        // errors a site explains come first: the rest may follow from them
        if !stray.is_empty() && !hit.iter().any(|h| *h) {
            // an error no site explains at its span: a site bound earlier (a local the
            // error reads) whose view the error names takes the other view it names;
            // else every site on the error's lines stays a numeral
            let mut fixed = false;
            for d in &stray {
                if let Some(fi) = files.iter().position(|f| d.file.ends_with(f.as_str())) {
                    let rendered =
                        std::fs::read_to_string(root.join(&files[fi])).unwrap_or_default();
                    let line_of =
                        |at: usize| rendered[..at.min(rendered.len())].matches('\n').count() + 1;
                    let mut retyped = false;
                    for k in 0..sites.len() {
                        if sites[k].file != fi || sites[k].dropped || hit[k] {
                            continue;
                        }
                        let idx = sites[..k].iter().filter(|x| x.file == fi).count();
                        let l = line_of(ranges[fi][idx].0);
                        let cur = sites[k].tried.last().cloned().unwrap_or_default();
                        if l <= d.line + 3 && l + 60 >= d.line && d.types.contains(&cur) {
                            if let Some(t) = d.types.iter().find(|t| {
                                **t != cur && *t != "{integer}" && !sites[k].tried.contains(t)
                            }) {
                                sites[k].tried.push(t.clone());
                                hit[k] = true;
                                retyped_now.push(k);
                                want[k] = None;
                                retyped = true;
                                fixed = true;
                            }
                        }
                    }
                    if retyped {
                        continue;
                    }
                    for k in 0..sites.len() {
                        if sites[k].file != fi || sites[k].dropped {
                            continue;
                        }
                        let idx = sites[..k].iter().filter(|x| x.file == fi).count();
                        let l = line_of(ranges[fi][idx].0);
                        if l + 3 >= d.line && l <= d.line + 3 {
                            sites[k].dropped = true;
                            fixed = true;
                        }
                    }
                }
            }
            if !fixed {
                if std::env::var_os("FIGURES_DERIVE_KEEP").is_none() {
                    restore();
                }
                return Err(format!(
                    "cargo check: {} errors no rewritten site explains, first: {:?}",
                    stray.len(),
                    stray.iter().take(3).collect::<Vec<_>>()
                ));
            }
        }
        for k in 0..sites.len() {
            if !hit[k] || retyped_now.contains(&k) {
                continue;
            }
            let next = want[k]
                .clone()
                .filter(|t| !sites[k].tried.contains(t))
                .or_else(|| {
                    VIEWS
                        .iter()
                        .map(|v| v.to_string())
                        .find(|v| !sites[k].tried.contains(v))
                });
            match next {
                Some(t) => sites[k].tried.push(t),
                None => sites[k].dropped = true,
            }
        }
    }
    for s in sites.iter_mut() {
        s.dropped = true;
    }
    restore();
    Err("the compiler admitted no view for some sites within the rounds".to_string())
}
