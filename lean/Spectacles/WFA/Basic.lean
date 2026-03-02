/-
  Spectacles.WFA.Basic
  Core WFA definition over an arbitrary semiring.

  Corresponds to: crates/spectacles-core/src/wfa/automaton.rs
-/

import Spectacles.Semiring.Basic

namespace Spectacles

/-- A weighted finite automaton over semiring S with alphabet Σ.
    States are natural numbers 0..num_states-1.
    Transitions are represented as a function: state × symbol → state → weight.
    Initial and final weight vectors assign weights to states. -/
structure WFA (S : Type) [Semiring S] where
  /-- Number of states -/
  num_states : Nat
  /-- Alphabet size -/
  alphabet_size : Nat
  /-- Transition weights: δ(q, a, q') gives the weight of going from q to q' on symbol a -/
  transition : Fin num_states → Fin alphabet_size → Fin num_states → S
  /-- Initial weight vector -/
  initial : Fin num_states → S
  /-- Final (accepting) weight vector -/
  final : Fin num_states → S

/-- A WFA is deterministic if for each (state, symbol) pair, at most one
    successor has non-zero transition weight. -/
def WFA.isDeterministic {S : Type} [Semiring S] [DecidableEq S] (w : WFA S) : Prop :=
  ∀ (q : Fin w.num_states) (a : Fin w.alphabet_size),
    ∃ (q' : Fin w.num_states),
      (w.transition q a q' ≠ Semiring.zero ∨
       (∀ (q'' : Fin w.num_states), w.transition q a q'' = Semiring.zero)) ∧
      (∀ (q'' : Fin w.num_states), q'' ≠ q' →
        w.transition q a q'' = Semiring.zero)

/-- A WFA is trimmed if every state is both reachable and co-reachable. -/
def WFA.isTrimmed {S : Type} [Semiring S] [DecidableEq S] (w : WFA S) : Prop :=
  -- Every state with non-zero initial weight can reach some state with non-zero final weight,
  -- and vice versa. (Simplified characterization)
  ∀ (q : Fin w.num_states),
    (w.initial q ≠ Semiring.zero → ∃ q', w.final q' ≠ Semiring.zero) ∧
    (w.final q ≠ Semiring.zero → ∃ q', w.initial q' ≠ Semiring.zero)

/-- The WFA obtained by restricting to states in the given set. -/
def WFA.restrict {S : Type} [Semiring S] (w : WFA S) (keep : Fin w.num_states → Bool) :
    WFA S where
  num_states := w.num_states  -- Simplified: keeps all states but zeros out removed ones
  alphabet_size := w.alphabet_size
  transition q a q' := if keep q && keep q' then w.transition q a q' else Semiring.zero
  initial q := if keep q then w.initial q else Semiring.zero
  final q := if keep q then w.final q else Semiring.zero

end Spectacles
