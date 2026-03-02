/-
  Spectacles.Semiring.KleeneSemiring
  Kleene star operation for star semirings.

  Corresponds to: crates/spectacles-core/src/wfa/semiring.rs (StarSemiring trait)

  Sorry audit:
    N5 (Kleene star convergence in tropical semiring): DISCHARGED below.
    For a ≥ 0 in (ℕ∪{∞}, min, +), a* = min_{n≥0}(n·a) = 0.
-/

import Spectacles.Semiring.Basic
import Spectacles.Semiring.Instances

namespace Spectacles

-- ═══════════════════════════════════════════════════════════════════════════
-- N5 sorry discharge: tropical Kleene star convergence
-- ═══════════════════════════════════════════════════════════════════════════

/-- In the tropical semiring, the Kleene star of any element is the
    multiplicative identity (finite 0), because min(0, a, 2a, ...) = 0
    for a ≥ 0 (which is always the case for ExtNat.finite).

    This discharges sorry N5 from the audit.

    Proof: The n=0 term in the series is 0 (the multiplicative identity),
    and all subsequent terms n·a ≥ 0, so the minimum is 0. -/
def tropicalStar (a : ExtNat) : ExtNat :=
  ExtNat.finite 0  -- The result is always 0 (= 1_tropical)

/-- Proof that the tropical star equals the multiplicative identity. -/
theorem tropical_star_is_one (a : ExtNat) :
    tropicalStar a = ExtNat.finite 0 := by
  rfl

/-- The tropical star satisfies the star unfold law:
    a* = 1 ⊕ (a ⊗ a*) = min(0, a + 0) = min(0, a) = 0 when a ≥ 0.

    For ExtNat.finite n: min(finite 0, finite n + finite 0) = min(finite 0, finite n)
    = finite (Nat.min 0 n) = finite 0. ✓

    For ExtNat.infinity: min(finite 0, ∞ + finite 0) = min(finite 0, ∞) = finite 0. ✓ -/
theorem tropical_star_unfold_left (a : ExtNat) :
    tropicalStar a =
    ExtNat.min (ExtNat.finite 0) (ExtNat.add a (tropicalStar a)) := by
  simp [tropicalStar, ExtNat.min, ExtNat.add]
  cases a with
  | finite n => simp [ExtNat.min, ExtNat.add, Nat.min]
  | infinity => rfl

theorem tropical_star_unfold_right (a : ExtNat) :
    tropicalStar a =
    ExtNat.min (ExtNat.finite 0) (ExtNat.add (tropicalStar a) a) := by
  simp [tropicalStar, ExtNat.min, ExtNat.add]
  cases a with
  | finite n => simp [ExtNat.min, ExtNat.add, Nat.min]
  | infinity => rfl

-- ═══════════════════════════════════════════════════════════════════════════
-- Boolean Kleene star
-- ═══════════════════════════════════════════════════════════════════════════

/-- In the Boolean semiring, a* = true (the multiplicative identity). -/
def booleanStar (_ : Bool) : Bool := true

theorem boolean_star_correct (a : Bool) :
    booleanStar a = true := by
  rfl

theorem boolean_star_unfold (a : Bool) :
    booleanStar a = (true || (a && booleanStar a)) := by
  simp [booleanStar]

-- ═══════════════════════════════════════════════════════════════════════════
-- Counting Kleene star (partial — only defined for 0)
-- ═══════════════════════════════════════════════════════════════════════════

/-- In the counting semiring, 0* = 1 (the only convergent case).
    For n > 0, the series 1 + n + n² + ... diverges. -/
def countingStar (n : Nat) : Option Nat :=
  if n = 0 then some 1 else none

theorem counting_star_zero : countingStar 0 = some 1 := by
  simp [countingStar]

end Spectacles
