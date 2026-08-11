/// The semiconductor material of the cell.
///
/// **Descriptive, never prescriptive.** No physics code may branch on
/// this type to change fundamental equations. A model may use it to
/// *select published parameters* from a library; it may not use it to
/// choose a different formula. The distinction matters because a
/// technology label is commercial: branching physics on it would make the
/// engine's core modifiable for every new product family, and would leave
/// [`CellTechnology::Unknown`] modules without defined behaviour — which
/// is the majority case for CEC imports, since that database carries no
/// architecture information at all
/// (`docs/design/26-validation-empirique-cec-sandia.md` §2.1).
///
/// **Orthogonal to [`CellArchitecture`]**: material and contact
/// architecture evolve independently. A PERC cell and a TOPCon cell are
/// both monocrystalline silicon.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CellTechnology {
    /// Monocrystalline silicon.
    MonocrystallineSilicon,
    /// Multicrystalline (polycrystalline) silicon.
    MulticrystallineSilicon,
    /// Cadmium telluride thin film.
    CadmiumTelluride,
    /// Copper indium gallium selenide thin film.
    CopperIndiumGalliumSelenide,
    /// Amorphous silicon thin film.
    AmorphousSilicon,
    /// Perovskite.
    Perovskite,
    /// A tandem stack of two technologies.
    ///
    /// Recursive rather than a flat variant per pairing: a flat
    /// enumeration would need one variant per possible combination, and
    /// perovskite-on-silicon is only the first of these to industrialise.
    Tandem {
        /// The upper (wider band gap) sub-cell.
        top: Box<CellTechnology>,
        /// The lower sub-cell.
        bottom: Box<CellTechnology>,
    },
    /// The source states nothing about the technology.
    ///
    /// Distinct from [`CellTechnology::Other`]: `Unknown` means the
    /// information is absent, `Other` means the source named something
    /// this model does not yet classify.
    Unknown,
    /// The source names a technology this model does not classify.
    Other {
        /// The name as the source gave it, e.g. the CEC library's
        /// coarse `"Thin Film"` category.
        name: String,
    },
}

/// The cell's contact architecture.
///
/// Subject to the same descriptive-only rule as [`CellTechnology`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CellArchitecture {
    /// Aluminium back surface field, the historical baseline.
    AluminiumBackSurfaceField,
    /// Passivated emitter and rear cell.
    PERC,
    /// Tunnel oxide passivated contact.
    TOPCon,
    /// Silicon heterojunction.
    HeterojunctionHJT,
    /// Interdigitated back contact.
    InterdigitatedBackContact,
    /// A monolithically scribed thin-film stack, with no discrete cells.
    MonolithicThinFilm,
    /// The source states nothing about the architecture. This is what
    /// every CEC import yields.
    Unknown,
    /// The source names an architecture this model does not classify.
    Other {
        /// The name as the source gave it.
        name: String,
    },
}

/// The module's technology, as the pair of orthogonal classifications.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleTechnology {
    /// The semiconductor material.
    pub technology: CellTechnology,
    /// The contact architecture.
    pub architecture: CellArchitecture,
}

impl ModuleTechnology {
    /// Construct a module technology from its two axes.
    #[inline]
    pub const fn new(technology: CellTechnology, architecture: CellArchitecture) -> Self {
        ModuleTechnology {
            technology,
            architecture,
        }
    }

    /// A technology about which the source says nothing.
    #[inline]
    pub const fn unknown() -> Self {
        ModuleTechnology {
            technology: CellTechnology::Unknown,
            architecture: CellArchitecture::Unknown,
        }
    }

    /// Whether the material is a thin film, and therefore whether the
    /// module is expected to be monolithic rather than built from
    /// discrete cells.
    ///
    /// This is a *descriptive* query used by the structural invariants,
    /// not a physics branch: it informs `INV-STR-004`, which warns when
    /// a thin-film material is paired with a discrete-cell
    /// configuration, without changing any equation.
    #[inline]
    pub fn is_thin_film(&self) -> bool {
        matches!(
            self.technology,
            CellTechnology::CadmiumTelluride
                | CellTechnology::CopperIndiumGalliumSelenide
                | CellTechnology::AmorphousSilicon
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_is_distinct_from_other() {
        assert_ne!(
            CellTechnology::Unknown,
            CellTechnology::Other {
                name: "Thin Film".to_string()
            }
        );
        assert_ne!(
            CellArchitecture::Unknown,
            CellArchitecture::Other {
                name: "something".to_string()
            }
        );
    }

    #[test]
    fn tandem_nests_recursively() {
        let t = CellTechnology::Tandem {
            top: Box::new(CellTechnology::Perovskite),
            bottom: Box::new(CellTechnology::MonocrystallineSilicon),
        };
        match t {
            CellTechnology::Tandem { top, bottom } => {
                assert_eq!(*top, CellTechnology::Perovskite);
                assert_eq!(*bottom, CellTechnology::MonocrystallineSilicon);
            }
            _ => panic!("expected a tandem"),
        }
    }

    #[test]
    fn a_tandem_of_tandems_is_representable() {
        // Not a product that exists, but the recursion must not be
        // artificially limited to one level.
        let inner = CellTechnology::Tandem {
            top: Box::new(CellTechnology::Perovskite),
            bottom: Box::new(CellTechnology::MonocrystallineSilicon),
        };
        let outer = CellTechnology::Tandem {
            top: Box::new(CellTechnology::Perovskite),
            bottom: Box::new(inner),
        };
        assert!(matches!(outer, CellTechnology::Tandem { .. }));
    }

    #[test]
    fn technology_and_architecture_are_independent() {
        // The same material with two architectures, and the same
        // architecture with two materials.
        let perc = ModuleTechnology::new(
            CellTechnology::MonocrystallineSilicon,
            CellArchitecture::PERC,
        );
        let topcon = ModuleTechnology::new(
            CellTechnology::MonocrystallineSilicon,
            CellArchitecture::TOPCon,
        );
        assert_eq!(perc.technology, topcon.technology);
        assert_ne!(perc.architecture, topcon.architecture);
    }

    #[test]
    fn thin_film_materials_are_recognised() {
        assert!(
            ModuleTechnology::new(
                CellTechnology::CadmiumTelluride,
                CellArchitecture::MonolithicThinFilm
            )
            .is_thin_film()
        );
        assert!(
            ModuleTechnology::new(
                CellTechnology::CopperIndiumGalliumSelenide,
                CellArchitecture::MonolithicThinFilm
            )
            .is_thin_film()
        );
        assert!(
            !ModuleTechnology::new(
                CellTechnology::MonocrystallineSilicon,
                CellArchitecture::PERC
            )
            .is_thin_film()
        );
    }

    #[test]
    fn unknown_technology_is_constructible() {
        let t = ModuleTechnology::unknown();
        assert_eq!(t.technology, CellTechnology::Unknown);
        assert_eq!(t.architecture, CellArchitecture::Unknown);
    }
}
