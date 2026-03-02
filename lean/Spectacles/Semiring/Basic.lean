/-
  Spectacles.Semiring.Basic
  Core semiring typeclass and basic algebraic laws.

  Corresponds to: crates/spectacles-core/src/wfa/semiring.rs (Semiring trait)

  Sorry audit: No sorrys in this file. All axioms are part of the typeclass.
-/

namespace Spectacles

/-- A semiring (S, ⊕, ⊗, 0, 1) satisfying the standard axioms:
    - (S, ⊕, 0) is a commutative monoid
    - (S, ⊗, 1) is a monoid
    - ⊗ distributes over ⊕ from both sides
    - 0 annihilates under ⊗ from both sides -/
class Semiring (S : Type) where
  zero : S
  one : S
  add : S → S → S
  mul : S → S → S
  add_comm : ∀ (a b : S), add a b = add b a
  add_assoc : ∀ (a b c : S), add (add a b) c = add a (add b c)
  add_zero : ∀ (a : S), add a zero = a
  zero_add : ∀ (a : S), add zero a = a
  mul_assoc : ∀ (a b c : S), mul (mul a b) c = mul a (mul b c)
  mul_one : ∀ (a : S), mul a one = a
  one_mul : ∀ (a : S), mul one a = a
  mul_zero : ∀ (a : S), mul a zero = zero
  zero_mul : ∀ (a : S), mul zero a = zero
  left_distrib : ∀ (a b c : S), mul a (add b c) = add (mul a b) (mul a c)
  right_distrib : ∀ (a b c : S), mul (add a b) c = add (mul a c) (mul b c)

/-- A commutative semiring additionally satisfies mul_comm. -/
class CommSemiring (S : Type) extends Semiring S where
  mul_comm : ∀ (a b : S), mul a b = mul b a

/-- A star semiring extends a semiring with a Kleene star operation. -/
class StarSemiring (S : Type) extends Semiring S where
  star : S → S
  star_unfold_left : ∀ (a : S), star a = Semiring.add Semiring.one (Semiring.mul a (star a))
  star_unfold_right : ∀ (a : S), star a = Semiring.add Semiring.one (Semiring.mul (star a) a)

/-- A semiring homomorphism preserves both operations and identities. -/
structure SemiringHom (S T : Type) [Semiring S] [Semiring T] where
  f : S → T
  map_zero : f Semiring.zero = Semiring.zero
  map_one : f Semiring.one = Semiring.one
  map_add : ∀ (a b : S), f (Semiring.add a b) = Semiring.add (f a) (f b)
  map_mul : ∀ (a b : S), f (Semiring.mul a b) = Semiring.mul (f a) (f b)

/-- An injective semiring homomorphism. Used for field embeddings. -/
structure InjectiveSemiringHom (S T : Type) [Semiring S] [Semiring T]
    extends SemiringHom S T where
  injective : ∀ (a b : S), f a = f b → a = b

-- Basic derived lemmas

theorem Semiring.add_left_cancel {S : Type} [Semiring S] [DecidableEq S]
    (a b c : S) (h : Semiring.add a b = Semiring.add a c)
    (h_cancel : ∀ (x y z : S), Semiring.add x y = Semiring.add x z → y = z) :
    b = c :=
  h_cancel a b c h

end Spectacles
