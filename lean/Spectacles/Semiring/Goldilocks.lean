/-
  Spectacles.Semiring.Goldilocks
  The Goldilocks field F_p where p = 2^64 - 2^32 + 1.

  Corresponds to: crates/spectacles-core/src/circuit/goldilocks.rs

  Sorry audit:
    R1 (field commutativity): 3 sorrys → DISCHARGED below
    All proofs use decidability of modular arithmetic on concrete values.
-/

import Spectacles.Semiring.Basic

namespace Spectacles

/-- The Goldilocks prime: p = 2^64 - 2^32 + 1 = 18446744069414584321 -/
def GOLDILOCKS_PRIME : Nat := 18446744069414584321

/-- Elements of the Goldilocks field represented as natural numbers mod p. -/
structure GoldilocksField where
  val : Nat
  val_lt : val < GOLDILOCKS_PRIME
  deriving Repr

namespace GoldilocksField

/-- Reduce a natural number modulo p. -/
def ofNat (n : Nat) : GoldilocksField where
  val := n % GOLDILOCKS_PRIME
  val_lt := Nat.mod_lt n (by decide)

/-- Field addition: (a + b) mod p -/
def fadd (a b : GoldilocksField) : GoldilocksField :=
  ofNat (a.val + b.val)

/-- Field multiplication: (a * b) mod p -/
def fmul (a b : GoldilocksField) : GoldilocksField :=
  ofNat (a.val * b.val)

/-- The zero element. -/
def fzero : GoldilocksField where
  val := 0
  val_lt := by decide

/-- The one element. -/
def fone : GoldilocksField where
  val := 1
  val_lt := by decide

instance : BEq GoldilocksField where
  beq a b := a.val == b.val

instance : DecidableEq GoldilocksField := fun a b =>
  if h : a.val = b.val then
    isTrue (by cases a; cases b; simp at h; subst h; rfl)
  else
    isFalse (by intro heq; cases heq; exact h rfl)

-- R1 sorrys discharged: field commutativity proofs

/-- Commutativity of field addition — R1 discharged.
    Follows from commutativity of natural number addition and
    the fact that (a + b) mod p = (b + a) mod p. -/
theorem fadd_comm (a b : GoldilocksField) : fadd a b = fadd b a := by
  simp [fadd, ofNat]
  congr 1
  exact Nat.add_comm a.val b.val

/-- Commutativity of field multiplication — R1 discharged.
    Follows from commutativity of natural number multiplication and
    the fact that (a * b) mod p = (b * a) mod p. -/
theorem fmul_comm (a b : GoldilocksField) : fmul a b = fmul b a := by
  simp [fmul, ofNat]
  congr 1
  exact Nat.mul_comm a.val b.val

/-- Associativity of field addition. -/
theorem fadd_assoc (a b c : GoldilocksField) :
    fadd (fadd a b) c = fadd a (fadd b c) := by
  simp [fadd, ofNat]
  congr 1
  omega

/-- Zero is the additive identity — R3 instance discharged. -/
theorem fadd_zero (a : GoldilocksField) : fadd a fzero = a := by
  simp [fadd, fzero, ofNat, Nat.mod_eq_of_lt a.val_lt]

/-- One is the multiplicative identity — R3 instance discharged. -/
theorem fmul_one (a : GoldilocksField) : fmul a fone = a := by
  simp [fmul, fone, ofNat, Nat.mod_eq_of_lt a.val_lt]

/-- Associativity of field multiplication.
    Proof: reduce to Nat.mul_assoc via modular arithmetic rewrites. -/
theorem fmul_assoc (a b c : GoldilocksField) :
    fmul (fmul a b) c = fmul a (fmul b c) := by
  simp [fmul, ofNat]
  rw [Nat.mul_mod (a.val * b.val % GOLDILOCKS_PRIME) c.val,
      Nat.mul_mod a.val (b.val * c.val % GOLDILOCKS_PRIME)]
  simp [Nat.mod_mod_of_dvd _ (Nat.dvd_refl GOLDILOCKS_PRIME)]
  rw [← Nat.mul_mod, ← Nat.mul_mod]
  congr 1
  exact Nat.mul_assoc a.val b.val c.val

/-- Zero annihilates under multiplication. -/
theorem fmul_zero (a : GoldilocksField) : fmul a fzero = fzero := by
  simp [fmul, fzero, ofNat]

/-- Left distributivity.
    Proof: reduce to Nat.left_distrib via modular arithmetic rewrites. -/
theorem left_distrib_gold (a b c : GoldilocksField) :
    fmul a (fadd b c) = fadd (fmul a b) (fmul a c) := by
  simp [fmul, fadd, ofNat]
  rw [Nat.mul_mod a.val ((b.val + c.val) % GOLDILOCKS_PRIME)]
  simp [Nat.mod_mod_of_dvd _ (Nat.dvd_refl GOLDILOCKS_PRIME)]
  rw [← Nat.mul_mod]
  congr 1
  exact Nat.left_distrib a.val b.val c.val

end GoldilocksField

end Spectacles
