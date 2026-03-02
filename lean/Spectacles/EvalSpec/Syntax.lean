/-
  Spectacles.EvalSpec.Syntax
  Abstract syntax of the EvalSpec DSL for metric specification.

  Corresponds to: crates/spectacles-core/src/evalspec/types.rs

  This provides the Lean formalization of the EvalSpec DSL that was
  previously only defined in Rust. The formalization covers:
  - Base types (String, Int, Float, Bool, List, Token, TokenSequence)
  - Semiring type annotations (Counting, Boolean, Tropical)
  - Expression AST (literals, variables, binary/unary ops, let bindings)
  - The EvalType combining base types with semiring annotations
-/

import Spectacles.Semiring.Basic
import Spectacles.Semiring.Instances
import Spectacles.Semiring.Goldilocks

namespace Spectacles.EvalSpec

-- ═══════════════════════════════════════════════════════════════════════════
-- Base types
-- ═══════════════════════════════════════════════════════════════════════════

/-- Base types in the EvalSpec type system.
    Corresponds to Rust's BaseType enum. -/
inductive BaseType where
  | string : BaseType
  | integer : BaseType
  | float : BaseType
  | bool : BaseType
  | list : BaseType → BaseType
  | token : BaseType
  | tokenSequence : BaseType
  | ngram : Nat → BaseType
  deriving Repr, DecidableEq

/-- A base type is a scalar (non-compound) type. -/
def BaseType.isScalar : BaseType → Bool
  | .string => true
  | .integer => true
  | .float => true
  | .bool => true
  | _ => false

-- ═══════════════════════════════════════════════════════════════════════════
-- Semiring type classification
-- ═══════════════════════════════════════════════════════════════════════════

/-- The semiring types supported by EvalSpec.
    Each corresponds to a concrete semiring instance.
    Matches Rust's SemiringType enum. -/
inductive SemiringTag where
  | counting : SemiringTag       -- (ℕ, +, ×)
  | boolean : SemiringTag        -- ({0,1}, ∨, ∧)
  | tropical : SemiringTag       -- (ℕ∪{∞}, min, +)
  | goldilocks : SemiringTag     -- (F_p, +, ×)
  deriving Repr, DecidableEq

/-- The carrier type associated with a semiring tag. -/
def SemiringTag.CarrierType : SemiringTag → Type
  | .counting => Nat
  | .boolean => Bool
  | .tropical => ExtNat
  | .goldilocks => GoldilocksField

-- ═══════════════════════════════════════════════════════════════════════════
-- EvalSpec type system
-- ═══════════════════════════════════════════════════════════════════════════

/-- The type system for EvalSpec expressions.
    Combines base types with optional semiring annotations.
    Corresponds to Rust's EvalType enum. -/
inductive EvalType where
  | base : BaseType → EvalType
  | semiring : SemiringTag → EvalType
  | function : List EvalType → EvalType → EvalType
  | metric : EvalType → EvalType → EvalType
  | unit : EvalType
  | annotated : BaseType → SemiringTag → EvalType
  deriving Repr

/-- Extract the base type component. -/
def EvalType.getBase : EvalType → BaseType
  | .base b => b
  | .annotated b _ => b
  | .unit => .bool  -- unit approximated as empty tuple
  | .semiring _ => .float
  | _ => .string

/-- Extract the semiring annotation, if any. -/
def EvalType.getSemiring : EvalType → Option SemiringTag
  | .semiring s => some s
  | .annotated _ s => some s
  | _ => none

-- ═══════════════════════════════════════════════════════════════════════════
-- Literals
-- ═══════════════════════════════════════════════════════════════════════════

/-- Literal values in the EvalSpec language. -/
inductive Literal where
  | string : String → Literal
  | integer : Int → Literal
  | float : Int → Literal   -- Represented as fixed-point for decidability
  | bool : Bool → Literal
  deriving Repr, DecidableEq

/-- The type of a literal. -/
def Literal.typeOf : Literal → BaseType
  | .string _ => .string
  | .integer _ => .integer
  | .float _ => .float
  | .bool _ => .bool

-- ═══════════════════════════════════════════════════════════════════════════
-- Binary and unary operators
-- ═══════════════════════════════════════════════════════════════════════════

/-- Binary operators in EvalSpec.
    Matches Rust's BinaryOp enum. -/
inductive BinOp where
  | add : BinOp
  | sub : BinOp
  | mul : BinOp
  | div : BinOp
  | min : BinOp
  | max : BinOp
  | and : BinOp
  | or : BinOp
  | eq : BinOp
  | lt : BinOp
  | le : BinOp
  | concat : BinOp
  | semiringAdd : BinOp    -- ⊕ (semiring addition)
  | semiringMul : BinOp    -- ⊗ (semiring multiplication)
  deriving Repr, DecidableEq

/-- Unary operators. -/
inductive UnOp where
  | neg : UnOp
  | not : UnOp
  | star : UnOp    -- Kleene star
  deriving Repr, DecidableEq

-- ═══════════════════════════════════════════════════════════════════════════
-- Expression AST
-- ═══════════════════════════════════════════════════════════════════════════

/-- The central expression AST for EvalSpec.
    Matches the core of Rust's Expr enum, restricted to the
    subset that is WFA-compilable. -/
inductive Expr where
  | literal : Literal → Expr
  | var : String → Expr
  | binOp : BinOp → Expr → Expr → Expr
  | unOp : UnOp → Expr → Expr
  | letIn : String → EvalType → Expr → Expr → Expr
  | app : String → List Expr → Expr
  | ifThenElse : Expr → Expr → Expr → Expr
  | fold : Expr → Expr → Expr → Expr  -- fold f init collection
  deriving Repr

-- ═══════════════════════════════════════════════════════════════════════════
-- Typing context
-- ═══════════════════════════════════════════════════════════════════════════

/-- A typing context maps variable names to their types. -/
def TypCtx := List (String × EvalType)

/-- Look up a variable in the context. -/
def TypCtx.lookup (ctx : TypCtx) (x : String) : Option EvalType :=
  ctx.find? (fun p => p.1 == x) |>.map (·.2)

/-- Extend the context with a new binding. -/
def TypCtx.extend (ctx : TypCtx) (x : String) (τ : EvalType) : TypCtx :=
  (x, τ) :: ctx

end Spectacles.EvalSpec