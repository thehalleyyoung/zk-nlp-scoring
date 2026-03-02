import Lake
open Lake DSL

package spectacles where
  leanOptions := #[
    ⟨`autoImplicit, false⟩
  ]

@[default_target]
lean_lib Spectacles where
  srcDir := "."
  roots := #[`Spectacles]
