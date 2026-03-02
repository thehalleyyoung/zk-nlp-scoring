-- Spectacles: Lean 4 Formalization of WFA-based NLP Metric Scoring
-- Root module importing all sub-theories

-- Semiring layer: algebraic foundations
import Spectacles.Semiring.Basic
import Spectacles.Semiring.Instances
import Spectacles.Semiring.Goldilocks
import Spectacles.Semiring.Embedding
import Spectacles.Semiring.KleeneSemiring
import Spectacles.Semiring.Embeddable

-- WFA layer: weighted finite automata
import Spectacles.WFA.Basic
import Spectacles.WFA.Evaluation
import Spectacles.WFA.Minimization

-- Circuit/AIR layer: zero-knowledge proof compilation
import Spectacles.Circuit.TraceLayout
import Spectacles.Circuit.Compiler
import Spectacles.Circuit.Gadgets

-- EvalSpec DSL: metric specification language
import Spectacles.EvalSpec.Syntax
import Spectacles.EvalSpec.Typing
import Spectacles.EvalSpec.Semantics
import Spectacles.EvalSpec.Compiler
