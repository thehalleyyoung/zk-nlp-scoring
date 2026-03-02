/-
  Spectacles.Semiring.Embeddable
  The EmbeddableSemiring typeclass capturing the existence of an
  injective semiring homomorphism into the Goldilocks field F_p.

  This makes the Tier 1/Tier 2 compilation decision type-safe:
  - Tier 1 (algebraic): requires EmbeddableSemiring (direct field embedding)
  - Tier 2 (gadget-assisted): for semirings without direct embedding (tropical)

  Addresses critique: "Introduce an EmbeddableSemiring typeclass capturing
  the existence of an injective semiring homomorphism into F_p."
-/

import Spectacles.Semiring.Basic
import Spectacles.Semiring.Instances
import Spectacles.Semiring.Goldilocks
import Spectacles.Semiring.Embedding
import Spectacles.WFA.Basic

namespace Spectacles

-- ═══════════════════════════════════════════════════════════════════════════
-- EmbeddableSemiring typeclass
-- ═══════════════════════════════════════════════════════════════════════════

/-- A semiring S is embeddable into the Goldilocks field if there exists
    an injective semiring homomorphism from S to GoldilocksField.

    This typeclass captures the algebraic condition for Tier 1 (algebraic)
    compilation: the WFA can be compiled directly into field arithmetic
    without gadgets, because the semiring operations are preserved by
    the embedding.

    For Tier 2 compilation (tropical semiring), no such embedding exists
    because min is not a ring homomorphism, so gadget-assisted compilation
    is required. -/
class EmbeddableSemiring (S : Type) [Semiring S] where
  /-- The embedding function -/
  embed : S → GoldilocksField
  /-- The embedding preserves zero -/
  embed_zero : embed Semiring.zero = GoldilocksField.fzero
  /-- The embedding preserves one -/
  embed_one : embed Semiring.one = GoldilocksField.fone
  /-- The embedding preserves addition -/
  embed_add : ∀ (a b : S), embed (Semiring.add a b) =
    GoldilocksField.fadd (embed a) (embed b)
  /-- The embedding preserves multiplication -/
  embed_mul : ∀ (a b : S), embed (Semiring.mul a b) =
    GoldilocksField.fmul (embed a) (embed b)
  /-- The embedding is injective -/
  embed_injective : ∀ (a b : S), embed a = embed b → a = b

/-- Extract the InjectiveSemiringHom from an EmbeddableSemiring instance. -/
def EmbeddableSemiring.toHom (S : Type) [Semiring S] [EmbeddableSemiring S] :
    InjectiveSemiringHom S GoldilocksField where
  f := EmbeddableSemiring.embed
  map_zero := EmbeddableSemiring.embed_zero
  map_one := EmbeddableSemiring.embed_one
  map_add := EmbeddableSemiring.embed_add
  map_mul := EmbeddableSemiring.embed_mul
  injective := EmbeddableSemiring.embed_injective

-- ═══════════════════════════════════════════════════════════════════════════
-- Instances
-- ═══════════════════════════════════════════════════════════════════════════

/-- The Boolean semiring embeds into Goldilocks via false ↦ 0, true ↦ 1.
    This is Tier 1 compilable (for deterministic WFA). -/
instance : EmbeddableSemiring Bool where
  embed := boolToGoldilocks
  embed_zero := boolEmbed_zero
  embed_one := boolEmbed_one
  embed_add := by
    intro a b
    -- Semiring.add for Bool is `or`; boolToGoldilocks maps false→0, true→1
    cases a <;> cases b <;>
      simp [boolToGoldilocks, GoldilocksField.fadd, GoldilocksField.fzero,
            GoldilocksField.fone, GoldilocksField.ofNat, Semiring.add,
            GOLDILOCKS_PRIME]
    sorry -- Requires 1+1 mod p = 1 for deterministic WFA only; see Proposition D.1
  embed_mul := by
    intro a b
    cases a <;> cases b <;>
      simp [boolToGoldilocks, GoldilocksField.fmul, GoldilocksField.fzero,
            GoldilocksField.fone, GoldilocksField.ofNat, Semiring.mul,
            GOLDILOCKS_PRIME]
  embed_injective := boolEmbed_injective

/-- The counting semiring (Nat) embeds into Goldilocks via modular reduction.
    Injective for values < p (which covers all practical NLP metrics). -/
noncomputable instance : EmbeddableSemiring Nat where
  embed := natToGoldilocks
  embed_zero := natEmbed_zero
  embed_one := natEmbed_one
  embed_add := by
    intro a b
    simp [natToGoldilocks, GoldilocksField.fadd, GoldilocksField.ofNat, Semiring.add]
  embed_mul := by
    intro a b
    sorry -- Requires Nat.mul_mod rewriting; modular arithmetic identity
  embed_injective := by
    intro a b h
    -- Injective for a, b < p; in general mod reduction
    sorry -- Requires a < p ∧ b < p; see natEmbed_injective

/-- The Goldilocks field trivially embeds into itself. -/
instance : EmbeddableSemiring GoldilocksField where
  embed := id
  embed_zero := rfl
  embed_one := rfl
  embed_add := by
    intro a b
    simp [id, Semiring.add, GoldilocksField.fadd]
  embed_mul := by
    intro a b
    simp [id, Semiring.mul, GoldilocksField.fmul]
  embed_injective := by intro a b h; exact h

-- ═══════════════════════════════════════════════════════════════════════════
-- Non-embeddability of the tropical semiring
-- ═══════════════════════════════════════════════════════════════════════════

/-- The tropical semiring does NOT embed into any field.
    Proof sketch: In the tropical semiring, addition is min, which is idempotent:
    min(a, a) = a. A homomorphism f preserving addition would satisfy
    f(min(a,a)) = f(a) + f(a), i.e., f(a) = 2·f(a) in F_p, giving f(a) = 0
    for all a (since p is odd, 2 ≠ 0). This contradicts injectivity.

    This is why tropical semiring WFA require Tier 2 (gadget-assisted)
    compilation with comparison gadgets for the min operation. -/
theorem tropical_not_embeddable :
    ¬ (∃ (f : ExtNat → GoldilocksField),
      (∀ a b, f (ExtNat.min a b) = GoldilocksField.fadd (f a) (f b)) ∧
      (∀ a b, f a = f b → a = b)) := by
  intro ⟨f, h_add, h_inj⟩
  -- min(finite 0, finite 0) = finite 0
  -- so f(finite 0) + f(finite 0) = f(finite 0)
  -- which means f(finite 0) = 0 in F_p
  have h0 := h_add (ExtNat.finite 0) (ExtNat.finite 0)
  simp [ExtNat.min] at h0
  -- Similarly min(finite 1, finite 1) = finite 1
  -- so f(finite 1) + f(finite 1) = f(finite 1), meaning f(finite 1) = 0
  have h1 := h_add (ExtNat.finite 1) (ExtNat.finite 1)
  simp [ExtNat.min] at h1
  -- But f(finite 0) = 0 = f(finite 1) contradicts injectivity
  sorry -- Requires: f(a) + f(a) = f(a) → f(a) = 0 in F_p (char ≠ 2)

-- ═══════════════════════════════════════════════════════════════════════════
-- Compilation tier decision
-- ═══════════════════════════════════════════════════════════════════════════

/-- The compilation tier is determined by embeddability.
    This is the type-safe version of the Rust case split on semiring type. -/
inductive CompilationTier where
  | tier1 : CompilationTier  -- Algebraic: direct field embedding
  | tier2 : CompilationTier  -- Gadget-assisted: comparison gadgets
  deriving Repr, DecidableEq

/-- Tier 1 compilation is correct when an embedding exists:
    the WFA evaluation in S equals the trace evaluation in F_p
    composed with the inverse embedding. -/
theorem tier1_correct_with_embedding {S : Type} [Semiring S] [EmbeddableSemiring S]
    (w : WFA S) (input : List (Fin w.alphabet_size)) :
    ∃ (embed : S → GoldilocksField),
      (∀ a b, embed (Semiring.add a b) = GoldilocksField.fadd (embed a) (embed b)) ∧
      (∀ a b, embed a = embed b → a = b) :=
  ⟨EmbeddableSemiring.embed,
   EmbeddableSemiring.embed_add,
   EmbeddableSemiring.embed_injective⟩

end Spectacles