/-
  Spectacles.Semiring.Embedding
  Injective semiring homomorphisms from Boolean/Counting into Goldilocks field.

  Corresponds to: crates/spectacles-core/src/wfa/field_embedding.rs

  Sorry audit:
    R2 (trivial inequality): 4 sorrys → DISCHARGED below
    These are concrete numeric facts: small constants < 2^64 - 2^32 + 1.
-/

import Spectacles.Semiring.Basic
import Spectacles.Semiring.Instances
import Spectacles.Semiring.Goldilocks

namespace Spectacles

-- ═══════════════════════════════════════════════════════════════════════════
-- R2 sorry discharge: trivial inequalities
-- ═══════════════════════════════════════════════════════════════════════════

/-- 0 < p — R2 discharged. -/
theorem zero_lt_goldilocks : 0 < GOLDILOCKS_PRIME := by decide

/-- 1 < p — R2 discharged. -/
theorem one_lt_goldilocks : 1 < GOLDILOCKS_PRIME := by decide

/-- Any vocabulary index (≤ 100000) is less than p — R2 discharged.
    In practice NLP vocabularies are ≤ 250K tokens, all far below 2^64 - 2^32 + 1. -/
theorem vocab_bound_lt_goldilocks : 100000 < GOLDILOCKS_PRIME := by decide

/-- Any string length (≤ 10000) is less than p — R2 discharged. -/
theorem length_bound_lt_goldilocks : 10000 < GOLDILOCKS_PRIME := by decide

-- ═══════════════════════════════════════════════════════════════════════════
-- Boolean → Goldilocks embedding
-- ═══════════════════════════════════════════════════════════════════════════

/-- Embed Bool into GoldilocksField: false ↦ 0, true ↦ 1. -/
def boolToGoldilocks (b : Bool) : GoldilocksField :=
  if b then GoldilocksField.fone else GoldilocksField.fzero

/-- The Boolean embedding preserves zero. -/
theorem boolEmbed_zero : boolToGoldilocks false = GoldilocksField.fzero := by
  simp [boolToGoldilocks]

/-- The Boolean embedding preserves one. -/
theorem boolEmbed_one : boolToGoldilocks true = GoldilocksField.fone := by
  simp [boolToGoldilocks]

/-- The Boolean embedding is injective. -/
theorem boolEmbed_injective (a b : Bool) (h : boolToGoldilocks a = boolToGoldilocks b) :
    a = b := by
  cases a <;> cases b <;> simp [boolToGoldilocks, GoldilocksField.fzero, GoldilocksField.fone] at h
  all_goals (first | rfl | exact absurd h (by decide))

/-- The Boolean embedding preserves addition (OR maps to field add for
    deterministic WFA where at most one input is true).
    Note: This is only valid for deterministic WFA — see Proposition D.1 in the paper. -/
theorem boolEmbed_add_det (a b : Bool) (h_det : ¬(a = true ∧ b = true)) :
    boolToGoldilocks (a || b) =
    GoldilocksField.fadd (boolToGoldilocks a) (boolToGoldilocks b) := by
  cases a <;> cases b <;> simp [boolToGoldilocks, GoldilocksField.fadd, GoldilocksField.fzero,
    GoldilocksField.fone, GoldilocksField.ofNat, GOLDILOCKS_PRIME] at *

-- ═══════════════════════════════════════════════════════════════════════════
-- Counting → Goldilocks embedding
-- ═══════════════════════════════════════════════════════════════════════════

/-- Embed Nat into GoldilocksField via modular reduction.
    Injective for values < p. -/
def natToGoldilocks (n : Nat) : GoldilocksField :=
  GoldilocksField.ofNat n

/-- The counting embedding preserves zero. -/
theorem natEmbed_zero : natToGoldilocks 0 = GoldilocksField.fzero := by
  simp [natToGoldilocks, GoldilocksField.ofNat, GoldilocksField.fzero, GOLDILOCKS_PRIME]

/-- The counting embedding preserves one. -/
theorem natEmbed_one : natToGoldilocks 1 = GoldilocksField.fone := by
  simp [natToGoldilocks, GoldilocksField.ofNat, GoldilocksField.fone, GOLDILOCKS_PRIME]

/-- The counting embedding is injective for values below p. -/
theorem natEmbed_injective (a b : Nat) (ha : a < GOLDILOCKS_PRIME) (hb : b < GOLDILOCKS_PRIME)
    (h : natToGoldilocks a = natToGoldilocks b) : a = b := by
  simp [natToGoldilocks, GoldilocksField.ofNat] at h
  rw [Nat.mod_eq_of_lt ha, Nat.mod_eq_of_lt hb] at h
  exact h

/-- The counting embedding preserves addition when the sum is below p. -/
theorem natEmbed_add (a b : Nat) (h : a + b < GOLDILOCKS_PRIME) :
    natToGoldilocks (a + b) =
    GoldilocksField.fadd (natToGoldilocks a) (natToGoldilocks b) := by
  simp [natToGoldilocks, GoldilocksField.fadd, GoldilocksField.ofNat]

/-- The counting embedding preserves multiplication when the product is below p. -/
theorem natEmbed_mul (a b : Nat) (ha : a < GOLDILOCKS_PRIME) (hb : b < GOLDILOCKS_PRIME)
    (h : a * b < GOLDILOCKS_PRIME) :
    natToGoldilocks (a * b) =
    GoldilocksField.fmul (natToGoldilocks a) (natToGoldilocks b) := by
  simp [natToGoldilocks, GoldilocksField.fmul, GoldilocksField.ofNat]
  congr 1
  rw [Nat.mod_eq_of_lt ha, Nat.mod_eq_of_lt hb]

end Spectacles
