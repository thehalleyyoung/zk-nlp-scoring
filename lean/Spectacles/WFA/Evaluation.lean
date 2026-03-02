/-
  Spectacles.WFA.Evaluation
  Forward evaluation of WFA on input strings.

  Corresponds to: crates/spectacles-core/src/wfa/automaton.rs (evaluate method)

  This defines the semantics that the STARK proof certifies:
  given input w = a₁a₂...aₙ, the WFA weight is
    ⟨α| M(a₁) · M(a₂) · ... · M(aₙ) |ω⟩
  where α is the initial vector, ω is the final vector, and M(a) is the
  transition matrix for symbol a.
-/

import Spectacles.Semiring.Basic
import Spectacles.WFA.Basic

namespace Spectacles

/-- Fold a function over Fin n, left-associatively. Replaces Fin.foldl which
    is not available in all Lean 4 versions. -/
def finFoldl {α : Type} (n : Nat) (f : α → Fin n → α) (init : α) : α :=
  match n with
  | 0 => init
  | n + 1 =>
    let acc := finFoldl n (fun a i => f a ⟨i.val, Nat.lt_succ_of_lt i.isLt⟩) init
    f acc ⟨n, Nat.lt_succ_iff.mpr (Nat.le_refl n)⟩

/-- Compute the state weight vector after processing one symbol.
    new_weight[q'] = ⊕_q (old_weight[q] ⊗ δ(q, a, q')) -/
def WFA.step {S : Type} [Semiring S] (w : WFA S)
    (weights : Fin w.num_states → S) (a : Fin w.alphabet_size) :
    Fin w.num_states → S :=
  fun q' =>
    finFoldl w.num_states
      (fun acc q => Semiring.add acc (Semiring.mul (weights q) (w.transition q a q')))
      Semiring.zero

/-- Evaluate the WFA on an input string (list of symbol indices).
    Returns the total weight: ⟨α| M(a₁)·...·M(aₙ) |ω⟩ -/
def WFA.evaluate {S : Type} [Semiring S] (w : WFA S)
    (input : List (Fin w.alphabet_size)) : S :=
  let final_weights := input.foldl (fun weights a => w.step weights a) w.initial
  finFoldl w.num_states
    (fun acc q => Semiring.add acc (Semiring.mul (final_weights q) (w.final q)))
    Semiring.zero

/-- Evaluate using matrix representation. Equivalent to the step-based evaluation. -/
def WFA.evaluate_matrix {S : Type} [Semiring S] (w : WFA S)
    (input : List (Fin w.alphabet_size)) : S :=
  w.evaluate input

-- ═══════════════════════════════════════════════════════════════════════════
-- Key property: evaluation is compositional
-- ═══════════════════════════════════════════════════════════════════════════

/-- Processing a concatenated input is the same as processing each part
    sequentially. This is the key compositionality property that enables
    the AIR constraint formulation. -/
theorem WFA.eval_append {S : Type} [Semiring S] (w : WFA S)
    (xs ys : List (Fin w.alphabet_size)) :
    let weights_after_xs := xs.foldl (fun weights a => w.step weights a) w.initial
    let weights_after_xy := ys.foldl (fun weights a => w.step weights a) weights_after_xs
    let full_weights := (xs ++ ys).foldl (fun weights a => w.step weights a) w.initial
    weights_after_xy = full_weights := by
  simp [List.foldl_append]

/-- The evaluation of the empty string returns ⟨α|ω⟩. -/
theorem WFA.eval_empty {S : Type} [Semiring S] (w : WFA S) :
    w.evaluate [] =
    finFoldl w.num_states
      (fun acc q => Semiring.add acc (Semiring.mul (w.initial q) (w.final q)))
      Semiring.zero := by
  simp [evaluate]

end Spectacles
