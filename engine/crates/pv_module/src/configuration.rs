/// How the cells of a module are divided.
///
/// **This is not decorative.** A module advertised as "144 half-cells" is
/// electrically equivalent to 72 full cells in series with two half
/// modules in parallel. A model that read only "144" would compute a
/// thermal voltage wrong by a factor of about two — a silent and gross
/// modelling error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CellDivision {
    /// Whole cells.
    Full,
    /// Cells cut in two.
    Half,
    /// Cells cut in three.
    Third,
    /// Overlapping shingled strips.
    Shingled,
    /// Cells cut into some other number of pieces.
    Other {
        /// How many pieces each original cell was cut into.
        divisor: u32,
    },
}

impl CellDivision {
    /// How many pieces each original cell is cut into. Shingled cells
    /// report 1, since shingling changes the interconnection rather than
    /// subdividing a cell into electrically independent parts.
    #[inline]
    pub const fn divisor(self) -> u32 {
        match self {
            CellDivision::Full | CellDivision::Shingled => 1,
            CellDivision::Half => 2,
            CellDivision::Third => 3,
            CellDivision::Other { divisor } => divisor,
        }
    }
}

/// How the module's photovoltaic conversion is physically organised.
///
/// The two variants are the major structural divergence in the PV
/// landscape: crystalline silicon modules are assemblies of countable
/// discrete cells, while CdTe and CIGS modules are monolithic plates
/// laser-scribed into series strips. "Number of cells in series" does not
/// mean the same thing in both cases. Providing
/// [`CellConfiguration::Monolithic`] from the outset is what lets a CdTe
/// module be represented without inventing fields
/// (`docs/design/25-specification-pv-module.md` §4.4).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CellConfiguration {
    /// Discrete, countable cells: crystalline silicon.
    Discrete {
        /// Number of cell junctions in series. Sets the module's thermal
        /// voltage in the single-diode equation (De Soto et al., 2006).
        cells_in_series: u32,
        /// Number of parallel cell strings. One for the great majority of
        /// modules.
        cells_in_parallel: u32,
        /// How the cells are divided.
        cell_division: CellDivision,
        /// Number of sub-strings delimited by bypass diodes, if stated.
        substrings: Option<u32>,
        /// Number of bypass diodes, if stated. Required by the future
        /// partial-shading model.
        bypass_diodes: Option<u32>,
    },
    /// A monolithic thin-film plate scribed into series segments.
    Monolithic {
        /// Number of series-connected scribed segments.
        series_segments: u32,
    },
}

impl CellConfiguration {
    /// A discrete configuration of whole cells in a single series string.
    #[inline]
    pub const fn discrete(cells_in_series: u32) -> Self {
        CellConfiguration::Discrete {
            cells_in_series,
            cells_in_parallel: 1,
            cell_division: CellDivision::Full,
            substrings: None,
            bypass_diodes: None,
        }
    }

    /// The number of junctions in series, whichever variant this is.
    ///
    /// This is the quantity a single-diode model needs in order to scale
    /// the thermal voltage from cell to module.
    #[inline]
    pub const fn junctions_in_series(&self) -> u32 {
        match self {
            CellConfiguration::Discrete {
                cells_in_series, ..
            } => *cells_in_series,
            CellConfiguration::Monolithic { series_segments } => *series_segments,
        }
    }

    /// The number of parallel paths, whichever variant this is.
    #[inline]
    pub const fn parallel_paths(&self) -> u32 {
        match self {
            CellConfiguration::Discrete {
                cells_in_parallel, ..
            } => *cells_in_parallel,
            CellConfiguration::Monolithic { .. } => 1,
        }
    }

    /// Whether this is a monolithic thin-film configuration.
    #[inline]
    pub const fn is_monolithic(&self) -> bool {
        matches!(self, CellConfiguration::Monolithic { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn divisors_match_the_division() {
        assert_eq!(CellDivision::Full.divisor(), 1);
        assert_eq!(CellDivision::Half.divisor(), 2);
        assert_eq!(CellDivision::Third.divisor(), 3);
        assert_eq!(CellDivision::Shingled.divisor(), 1);
        assert_eq!(CellDivision::Other { divisor: 4 }.divisor(), 4);
    }

    #[test]
    fn half_cut_topology_is_representable_without_ambiguity() {
        // 144 half-cells: 72 junctions in series, two parallel paths.
        // Storing only "144" would double the thermal voltage.
        let c = CellConfiguration::Discrete {
            cells_in_series: 72,
            cells_in_parallel: 2,
            cell_division: CellDivision::Half,
            substrings: Some(3),
            bypass_diodes: Some(3),
        };
        assert_eq!(c.junctions_in_series(), 72);
        assert_eq!(c.parallel_paths(), 2);
        match c {
            CellConfiguration::Discrete { cell_division, .. } => {
                assert_eq!(cell_division, CellDivision::Half);
            }
            CellConfiguration::Monolithic { .. } => panic!("expected a discrete configuration"),
        }
    }

    #[test]
    fn monolithic_reports_segments_as_series_junctions() {
        let c = CellConfiguration::Monolithic {
            series_segments: 154,
        };
        assert_eq!(c.junctions_in_series(), 154);
        assert_eq!(c.parallel_paths(), 1);
        assert!(c.is_monolithic());
    }

    #[test]
    fn a_simple_discrete_configuration_defaults_sensibly() {
        let c = CellConfiguration::discrete(60);
        assert_eq!(c.junctions_in_series(), 60);
        assert_eq!(c.parallel_paths(), 1);
        assert!(!c.is_monolithic());
    }
}
