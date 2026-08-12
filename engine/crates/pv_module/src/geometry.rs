use core_units::{Area, Length, Mass};

use crate::provenance::Measurement;

/// The module's overall outside dimensions.
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleDimensions {
    /// Overall length.
    pub length: Measurement<Length>,
    /// Overall width.
    pub width: Measurement<Length>,
    /// Overall depth (thickness), if stated.
    pub depth: Option<Measurement<Length>>,
}

impl ModuleDimensions {
    /// Construct dimensions from length and width.
    #[inline]
    pub fn new(length: Measurement<Length>, width: Measurement<Length>) -> Self {
        ModuleDimensions {
            length,
            width,
            depth: None,
        }
    }

    /// Record the module's depth.
    #[inline]
    #[must_use]
    pub fn with_depth(mut self, depth: Measurement<Length>) -> Self {
        self.depth = Some(depth);
        self
    }

    /// The area of the bounding rectangle, `length x width`.
    ///
    /// **This is deliberately not called `gross_area`.** It is a derived
    /// quantity with its own name, because `length x width` equals the
    /// stated overall area only for a rectangular framed module — an
    /// approximation that shingled, trimmed or curved modules break.
    ///
    /// Keeping the names distinct is what makes `INV-GEO-002` meaningful:
    /// it compares two explicitly named quantities rather than a value
    /// against its own supposed definition. Direct inspection of the CEC
    /// library found 29 records where these two disagree by more than
    /// 10 %, up to a factor of two, and where it is the *derived* value
    /// that is wrong
    /// (`docs/design/26-validation-empirique-cec-sandia.md` §1).
    #[inline]
    pub fn rectangular_area(&self) -> Area {
        Area::from_square_meters(self.length.get().meters() * self.width.get().meters())
    }
}

/// How the module is built to be mounted, which determines what
/// mechanical limits apply to it.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum MountingForm {
    /// A rigid module, framed or frameless. Static load ratings in the
    /// sense of IEC 61215 apply.
    Rigid {
        /// The frame material, if stated. `None` denotes either a
        /// frameless module or an unstated frame.
        frame_material: Option<String>,
    },
    /// A flexible module on a polymer or metal substrate. Static load
    /// ratings do not apply in the same sense; the binding constraint is
    /// the minimum bend radius.
    Flexible {
        /// The minimum permissible bend radius, if stated.
        min_bend_radius: Option<Measurement<Length>>,
    },
}

impl MountingForm {
    /// A rigid module with an unstated frame.
    #[inline]
    pub const fn rigid() -> Self {
        MountingForm::Rigid {
            frame_material: None,
        }
    }
}

/// The module's physical geometry.
///
/// **Five areas are kept strictly separate**, and none is assumed equal
/// to any other. Only [`ModuleDimensions::rectangular_area`] is derived,
/// and it carries a distinct name so it cannot be confused with the
/// stated overall area
/// (`docs/design/27-specification-finale-pv-module.md` §8).
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleGeometry {
    /// Overall outside dimensions.
    pub dimensions: ModuleDimensions,
    /// Mass, if stated.
    pub mass: Option<Measurement<Mass>>,
    /// Overall area as stated by the source (total footprint).
    ///
    /// Not assumed equal to `dimensions.rectangular_area()`; when the two
    /// disagree, `INV-GEO-002` reports a conflict rather than choosing.
    pub gross_area: Option<Measurement<Area>>,
    /// Electrically active area, if stated.
    pub active_area: Option<Measurement<Area>>,
    /// Optical aperture area, if stated. Not derivable: it depends on the
    /// cell layout.
    pub aperture_area: Option<Measurement<Area>>,
    /// Total area of the cells themselves, if stated.
    pub cell_area: Option<Measurement<Area>>,
}

impl ModuleGeometry {
    /// Construct a geometry from the mandatory dimensions.
    #[inline]
    pub fn new(dimensions: ModuleDimensions) -> Self {
        ModuleGeometry {
            dimensions,
            mass: None,
            gross_area: None,
            active_area: None,
            aperture_area: None,
            cell_area: None,
        }
    }

    /// Record the module's mass.
    #[inline]
    #[must_use]
    pub fn with_mass(mut self, mass: Measurement<Mass>) -> Self {
        self.mass = Some(mass);
        self
    }

    /// Record the stated overall area.
    #[inline]
    #[must_use]
    pub fn with_gross_area(mut self, area: Measurement<Area>) -> Self {
        self.gross_area = Some(area);
        self
    }

    /// Record the electrically active area.
    #[inline]
    #[must_use]
    pub fn with_active_area(mut self, area: Measurement<Area>) -> Self {
        self.active_area = Some(area);
        self
    }

    /// Record the optical aperture area.
    #[inline]
    #[must_use]
    pub fn with_aperture_area(mut self, area: Measurement<Area>) -> Self {
        self.aperture_area = Some(area);
        self
    }

    /// Record the total cell area.
    #[inline]
    #[must_use]
    pub fn with_cell_area(mut self, area: Measurement<Area>) -> Self {
        self.cell_area = Some(area);
        self
    }

    /// The area of the bounding rectangle. See
    /// [`ModuleDimensions::rectangular_area`] for why this is not called
    /// `gross_area`.
    #[inline]
    pub fn rectangular_area(&self) -> Area {
        self.dimensions.rectangular_area()
    }

    /// The area that should be used for an efficiency calculation, and
    /// which one it is.
    ///
    /// Order of physical preference: aperture, then stated gross, then
    /// the bounding rectangle. **This preference is reported, never
    /// applied silently** — the caller is told which area was used so
    /// that an efficiency figure is never ambiguous
    /// (`docs/design/27-specification-finale-pv-module.md` §8.2).
    #[inline]
    pub fn preferred_area_for_efficiency(&self) -> (Area, &'static str) {
        if let Some(aperture) = &self.aperture_area {
            (aperture.get(), "aperture_area")
        } else if let Some(gross) = &self.gross_area {
            (gross.get(), "gross_area")
        } else {
            (self.rectangular_area(), "rectangular_area")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dims(length_m: f64, width_m: f64) -> ModuleDimensions {
        ModuleDimensions::new(
            Measurement::stated(Length::from_meters(length_m)),
            Measurement::stated(Length::from_meters(width_m)),
        )
    }

    #[test]
    fn rectangular_area_is_the_product_of_the_two_dimensions() {
        let d = dims(1.762, 1.134);
        // 1.762 m x 1.134 m = 1.998108 m2.
        assert!((d.rectangular_area().square_meters() - 1.998_108).abs() < 1e-9);
    }

    #[test]
    fn rectangular_area_and_gross_area_are_separate_quantities() {
        // The Sharp_ND_72ELUF case from the CEC library: the stated area
        // is right and the dimensions are wrong. The model must be able
        // to hold both without either overwriting the other.
        let g = ModuleGeometry::new(dims(1.165, 0.990))
            .with_gross_area(Measurement::stated(Area::from_square_meters(0.577)));
        assert!((g.rectangular_area().square_meters() - 1.153_35).abs() < 1e-6);
        assert!((g.gross_area.as_ref().unwrap().get().square_meters() - 0.577).abs() < 1e-12);
    }

    #[test]
    fn efficiency_area_prefers_aperture_then_gross_then_rectangle() {
        let bare = ModuleGeometry::new(dims(2.0, 1.0));
        assert_eq!(bare.preferred_area_for_efficiency().1, "rectangular_area");

        let with_gross = bare
            .clone()
            .with_gross_area(Measurement::stated(Area::from_square_meters(1.9)));
        assert_eq!(with_gross.preferred_area_for_efficiency().1, "gross_area");

        let with_aperture =
            with_gross.with_aperture_area(Measurement::stated(Area::from_square_meters(1.8)));
        assert_eq!(
            with_aperture.preferred_area_for_efficiency().1,
            "aperture_area"
        );
        assert!(
            (with_aperture
                .preferred_area_for_efficiency()
                .0
                .square_meters()
                - 1.8)
                .abs()
                < 1e-12
        );
    }

    #[test]
    fn depth_is_optional_and_recorded_explicitly() {
        let d = dims(2.0, 1.0);
        assert!(d.depth.is_none());
        let d = d.with_depth(Measurement::stated(Length::from_meters(0.030)));
        assert!((d.depth.unwrap().get().meters() - 0.030).abs() < 1e-12);
    }

    #[test]
    fn flexible_modules_carry_a_bend_radius_instead_of_a_frame() {
        let flexible = MountingForm::Flexible {
            min_bend_radius: Some(Measurement::stated(Length::from_meters(0.5))),
        };
        match flexible {
            MountingForm::Flexible { min_bend_radius } => {
                assert!((min_bend_radius.unwrap().get().meters() - 0.5).abs() < 1e-12);
            }
            MountingForm::Rigid { .. } => panic!("expected a flexible module"),
        }
        assert_eq!(
            MountingForm::rigid(),
            MountingForm::Rigid {
                frame_material: None
            }
        );
    }
}
