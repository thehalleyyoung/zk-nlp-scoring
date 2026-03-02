/-
  Spectacles.Semiring.Instances
  Concrete semiring instances: Boolean, Counting, Tropical.

  Corresponds to: crates/spectacles-core/src/wfa/semiring.rs
    (BooleanSemiring, CountingSemiring, TropicalSemiring)

  Sorry audit:
    R3 (identity simplification): 3 sorrys → DISCHARGED below
    R4 (decidability): 2 sorrys → DISCHARGED below
-/

import Spectacles.Semiring.Basic

namespace Spectacles

-- ═══════════════════════════════════════════════════════════════════════════
-- Boolean semiring  (𝔹, ∨, ∧, false, true)
-- ═══════════════════════════════════════════════════════════════════════════

/-- The Boolean semiring: addition is logical OR, multiplication is logical AND. -/
instance : Semiring Bool where
  zero := false
  one := true
  add := or
  mul := and
  add_comm := by
    intro a b
    cases a <;> cases b <;> rfl
  add_assoc := by
    intro a b c
    cases a <;> cases b <;> cases c <;> rfl
  add_zero := by                          -- R3: was sorry, now discharged
    intro a
    cases a <;> rfl
  zero_add := by                          -- R3: was sorry, now discharged
    intro a
    cases a <;> rfl
  mul_assoc := by
    intro a b c
    cases a <;> cases b <;> cases c <;> rfl
  mul_one := by                           -- R3: was sorry, now discharged
    intro a
    cases a <;> rfl
  one_mul := by
    intro a
    cases a <;> rfl
  mul_zero := by
    intro a
    cases a <;> rfl
  zero_mul := by
    intro a
    cases a <;> rfl
  left_distrib := by
    intro a b c
    cases a <;> cases b <;> cases c <;> rfl
  right_distrib := by
    intro a b c
    cases a <;> cases b <;> cases c <;> rfl

/-- Boolean decidable equality — R4 sorry discharged. -/
instance : DecidableEq Bool := Bool.decEq

-- ═══════════════════════════════════════════════════════════════════════════
-- Counting semiring  (ℕ, +, ×, 0, 1)
-- ═══════════════════════════════════════════════════════════════════════════

/-- The counting (natural number) semiring: standard addition and multiplication. -/
instance : Semiring Nat where
  zero := 0
  one := 1
  add := Nat.add
  mul := Nat.mul
  add_comm := Nat.add_comm
  add_assoc := Nat.add_assoc
  add_zero := Nat.add_zero
  zero_add := Nat.zero_add
  mul_assoc := Nat.mul_assoc
  mul_one := Nat.mul_one
  one_mul := Nat.one_mul
  mul_zero := Nat.mul_zero
  zero_mul := Nat.zero_mul
  left_distrib := Nat.left_distrib
  right_distrib := Nat.right_distrib

/-- Natural number decidable equality — R4 sorry discharged. -/
instance : DecidableEq Nat := Nat.decEq

-- ═══════════════════════════════════════════════════════════════════════════
-- Tropical semiring  (ℕ ∪ {∞}, min, +, ∞, 0)
-- ═══════════════════════════════════════════════════════════════════════════

/-- Extended natural numbers for the tropical semiring. -/
inductive ExtNat where
  | finite : Nat → ExtNat
  | infinity : ExtNat
  deriving DecidableEq, Repr

namespace ExtNat

def add : ExtNat → ExtNat → ExtNat
  | finite m, finite n => finite (m + n)
  | _, infinity => infinity
  | infinity, _ => infinity

def min : ExtNat → ExtNat → ExtNat
  | finite m, finite n => finite (Nat.min m n)
  | finite m, infinity => finite m
  | infinity, finite n => finite n
  | infinity, infinity => infinity

theorem min_comm (a b : ExtNat) : min a b = min b a := by
  cases a <;> cases b <;> simp [min, Nat.min_comm]

theorem min_assoc (a b c : ExtNat) : min (min a b) c = min a (min b c) := by
  cases a <;> cases b <;> cases c <;> simp [min, Nat.min_assoc]

theorem min_infinity (a : ExtNat) : min a infinity = a := by
  cases a <;> simp [min]

theorem infinity_min (a : ExtNat) : min infinity a = a := by
  cases a <;> simp [min]

theorem add_comm_ext (a b : ExtNat) : add a b = add b a := by
  cases a <;> cases b <;> simp [add, Nat.add_comm]

theorem add_assoc_ext (a b c : ExtNat) : add (add a b) c = add a (add b c) := by
  cases a <;> cases b <;> cases c <;> simp [add, Nat.add_assoc]

theorem add_zero_ext (a : ExtNat) : add a (finite 0) = a := by
  cases a <;> simp [add]

theorem zero_add_ext (a : ExtNat) : add (finite 0) a = a := by
  cases a <;> simp [add]

theorem add_infinity (a : ExtNat) : add a infinity = infinity := by
  cases a <;> simp [add]

theorem infinity_add (a : ExtNat) : add infinity a = infinity := by
  cases a <;> simp [add]

theorem left_distrib_ext (a b c : ExtNat) :
    add a (min b c) = min (add a b) (add a c) := by
  cases a <;> cases b <;> cases c <;> simp [add, min]
  · simp [Nat.min_def]; split <;> split <;> omega

theorem right_distrib_ext (a b c : ExtNat) :
    add (min a b) c = min (add a c) (add b c) := by
  cases a <;> cases b <;> cases c <;> simp [add, min]
  · simp [Nat.min_def]; split <;> split <;> omega

end ExtNat

/-- The tropical semiring: (ℕ∪{∞}, min, +, ∞, 0).
    Addition is min, multiplication is +, zero is ∞, one is 0. -/
instance : Semiring ExtNat where
  zero := ExtNat.infinity
  one := ExtNat.finite 0
  add := ExtNat.min
  mul := ExtNat.add
  add_comm := ExtNat.min_comm
  add_assoc := ExtNat.min_assoc
  add_zero := ExtNat.min_infinity
  zero_add := ExtNat.infinity_min
  mul_assoc := ExtNat.add_assoc_ext
  mul_one := ExtNat.add_zero_ext
  one_mul := ExtNat.zero_add_ext
  mul_zero := ExtNat.add_infinity
  zero_mul := ExtNat.infinity_add
  left_distrib := ExtNat.left_distrib_ext
  right_distrib := ExtNat.right_distrib_ext

end Spectacles
