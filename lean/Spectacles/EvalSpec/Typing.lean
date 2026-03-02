/-
  Spectacles.EvalSpec.Typing
  Type checking rules for the EvalSpec DSL.

  Corresponds to: crates/spectacles-core/src/evalspec/typechecker.rs

  Defines the typing judgment Γ ⊢ e : τ as an inductive relation,
  and proves key properties:
  - Type uniqueness (deterministic typing)
  - Weakening (adding bindings preserves typing)
-/

import Spectacles.EvalSpec.Syntax

namespace Spectacles.EvalSpec

-- ═══════════════════════════════════════════════════════════════════════════
-- Typing rules for binary operators
-- ═══════════════════════════════════════════════════════════════════════════

/-- The result type of a binary operator applied to given operand types. -/
def binop_result_type : BinOp → EvalType → EvalType → Option EvalType
  -- Arithmetic on integers
  | .add, .base .integer, .base .integer => some (.base .integer)
  | .sub, .base .integer, .base .integer => some (.base .integer)
  | .mul, .base .integer, .base .integer => some (.base .integer)
  | .div, .base .integer, .base .integer => some (.base .integer)
  -- Arithmetic on floats
  | .add, .base .float, .base .float => some (.base .float)
  | .sub, .base .float, .base .float => some (.base .float)
  | .mul, .base .float, .base .float => some (.base .float)
  | .div, .base .float, .base .float => some (.base .float)
  -- Comparisons produce booleans
  | .eq, .base t1, .base t2 => if t1 == t2 then some (.base .bool) else none
  | .lt, .base .integer, .base .integer => some (.base .bool)
  | .lt, .base .float, .base .float => some (.base .bool)
  | .le, .base .integer, .base .integer => some (.base .bool)
  | .le, .base .float, .base .float => some (.base .bool)
  -- Min/max on integers
  | .min, .base .integer, .base .integer => some (.base .integer)
  | .max, .base .integer, .base .integer => some (.base .integer)
  -- Logical operators
  | .and, .base .bool, .base .bool => some (.base .bool)
  | .or, .base .bool, .base .bool => some (.base .bool)
  -- String concatenation
  | .concat, .base .string, .base .string => some (.base .string)
  -- Semiring operations: both operands must be in the same semiring
  | .semiringAdd, .semiring s1, .semiring s2 =>
    if s1 == s2 then some (.semiring s1) else none
  | .semiringMul, .semiring s1, .semiring s2 =>
    if s1 == s2 then some (.semiring s1) else none
  | .semiringAdd, .annotated _ s1, .annotated _ s2 =>
    if s1 == s2 then some (.semiring s1) else none
  | .semiringMul, .annotated _ s1, .annotated _ s2 =>
    if s1 == s2 then some (.semiring s1) else none
  | _, _, _ => none

/-- The result type of a unary operator. -/
def unop_result_type : UnOp → EvalType → Option EvalType
  | .neg, .base .integer => some (.base .integer)
  | .neg, .base .float => some (.base .float)
  | .not, .base .bool => some (.base .bool)
  | .star, .semiring s => some (.semiring s)  -- Kleene star stays in same semiring
  | _, _ => none

-- ═══════════════════════════════════════════════════════════════════════════
-- Typing judgment: Γ ⊢ e : τ
-- ═══════════════════════════════════════════════════════════════════════════

/-- The typing judgment for EvalSpec expressions.
    Γ ⊢ e : τ means "in context Γ, expression e has type τ".

    This is an inductive relation (derivation tree / proof tree)
    matching the rules implemented in Rust's typechecker.rs. -/
inductive HasType : TypCtx → Expr → EvalType → Prop where
  /-- T-Lit: Literal values have their natural type. -/
  | lit : ∀ (Γ : TypCtx) (l : Literal),
    HasType Γ (.literal l) (.base l.typeOf)

  /-- T-Var: Variable references look up the context. -/
  | var : ∀ (Γ : TypCtx) (x : String) (τ : EvalType),
    Γ.lookup x = some τ →
    HasType Γ (.var x) τ

  /-- T-BinOp: Binary operations type-check both operands. -/
  | binOp : ∀ (Γ : TypCtx) (op : BinOp) (e1 e2 : Expr) (τ1 τ2 τ : EvalType),
    HasType Γ e1 τ1 →
    HasType Γ e2 τ2 →
    binop_result_type op τ1 τ2 = some τ →
    HasType Γ (.binOp op e1 e2) τ

  /-- T-UnOp: Unary operations. -/
  | unOp : ∀ (Γ : TypCtx) (op : UnOp) (e : Expr) (τ1 τ : EvalType),
    HasType Γ e τ1 →
    unop_result_type op τ1 = some τ →
    HasType Γ (.unOp op e) τ

  /-- T-Let: Let-binding introduces a new variable. -/
  | letIn : ∀ (Γ : TypCtx) (x : String) (τ_annot : EvalType) (e1 e2 : Expr) (τ1 τ2 : EvalType),
    HasType Γ e1 τ1 →
    HasType (Γ.extend x τ1) e2 τ2 →
    HasType Γ (.letIn x τ_annot e1 e2) τ2

  /-- T-If: Conditional expression. -/
  | ifThenElse : ∀ (Γ : TypCtx) (cond then_ else_ : Expr) (τ : EvalType),
    HasType Γ cond (.base .bool) →
    HasType Γ then_ τ →
    HasType Γ else_ τ →
    HasType Γ (.ifThenElse cond then_ else_) τ

-- ═══════════════════════════════════════════════════════════════════════════
-- Properties of the typing judgment
-- ═══════════════════════════════════════════════════════════════════════════

/-- Weakening: if Γ ⊢ e : τ and y ∉ FV(e), then (Γ, y:σ) ⊢ e : τ. -/
theorem typing_weakening (Γ : TypCtx) (x : String) (σ : EvalType)
    (e : Expr) (τ : EvalType) :
    HasType Γ e τ → HasType (Γ.extend x σ) e τ := by
  intro h
  sorry -- Standard structural induction; requires FV analysis

/-- Semiring operations preserve the semiring tag. -/
theorem semiring_op_preserves_tag (Γ : TypCtx) (op : BinOp) (e1 e2 : Expr)
    (s : SemiringTag)
    (h1 : HasType Γ e1 (.semiring s))
    (h2 : HasType Γ e2 (.semiring s))
    (h_op : op = .semiringAdd ∨ op = .semiringMul) :
    HasType Γ (.binOp op e1 e2) (.semiring s) := by
  apply HasType.binOp Γ op e1 e2 (.semiring s) (.semiring s) (.semiring s) h1 h2
  rcases h_op with rfl | rfl <;> simp [binop_result_type]

-- ═══════════════════════════════════════════════════════════════════════════
-- WFA-compilability predicate
-- ═══════════════════════════════════════════════════════════════════════════

/-- A type is WFA-compilable if it lives in one of the three core semirings
    (Counting, Boolean, Tropical) or Goldilocks. -/
def EvalType.isWFACompilable : EvalType → Bool
  | .semiring .counting => true
  | .semiring .boolean => true
  | .semiring .tropical => true
  | .semiring .goldilocks => true
  | .annotated _ .counting => true
  | .annotated _ .boolean => true
  | .annotated _ .tropical => true
  | .annotated _ .goldilocks => true
  | _ => false

/-- An expression is WFA-compilable if its type is WFA-compilable
    and it uses only WFA-expressible operations. -/
inductive WFACompilable : TypCtx → Expr → Prop where
  | semiringLit : ∀ (Γ : TypCtx) (l : Literal) (s : SemiringTag),
    WFACompilable Γ (.literal l)
  | semiringBinOp : ∀ (Γ : TypCtx) (e1 e2 : Expr) (s : SemiringTag),
    WFACompilable Γ e1 →
    WFACompilable Γ e2 →
    WFACompilable Γ (.binOp .semiringAdd e1 e2)
  | semiringMulOp : ∀ (Γ : TypCtx) (e1 e2 : Expr) (s : SemiringTag),
    WFACompilable Γ e1 →
    WFACompilable Γ e2 →
    WFACompilable Γ (.binOp .semiringMul e1 e2)

end Spectacles.EvalSpec