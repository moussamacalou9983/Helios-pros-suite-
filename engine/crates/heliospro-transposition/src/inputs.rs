use heliospro_solar::incidence::SurfaceOrientation;
use heliospro_units::{Angle, Irradiance, Ratio};

/// Inputs shared by every transposition model in this crate.
#[derive(Debug, Clone, Copy)]
pub struct TranspositionInputs {
    /// Direct Normal Irradiance.
    pub dni: Irradiance,
    /// Diffuse Horizontal Irradiance.
    pub dhi: Irradiance,
    /// Global Horizontal Irradiance.
    pub ghi: Irradiance,
    /// Extraterrestrial irradiance on a plane normal to the Sun's rays
    /// (`G0n`), used by the anisotropic models ([`crate::hay_davies`],
    /// [`crate::hdkr`], [`crate::perez`]) to gauge how much of the diffuse
    /// sky radiation is concentrated near the solar disk (circumsolar
    /// brightening) versus isotropically distributed.
    pub extraterrestrial_normal: Irradiance,
    /// Solar zenith angle.
    pub solar_zenith: Angle,
    /// Angle of incidence between the Sun's rays and the surface normal
    /// (see [`heliospro_solar::incidence::angle_of_incidence`]).
    pub angle_of_incidence: Angle,
    /// Orientation of the receiving surface.
    pub surface: SurfaceOrientation,
    /// Ground albedo (reflectance) in front of the surface.
    pub ground_albedo: Ratio,
}

/// Plane-of-array irradiance, decomposed into its three physical
/// components plus their sum.
#[derive(Debug, Clone, Copy)]
pub struct PlaneOfArrayIrradiance {
    /// Direct (beam) component: `DNI * cos(angle_of_incidence)`, clamped to
    /// zero when the Sun is behind the surface plane.
    pub beam: Irradiance,
    /// Diffuse sky component (model-dependent).
    pub sky_diffuse: Irradiance,
    /// Ground-reflected component.
    pub ground_reflected: Irradiance,
    /// Sum of the three components: total plane-of-array irradiance.
    pub global: Irradiance,
}

impl PlaneOfArrayIrradiance {
    pub(crate) fn from_components(
        beam: Irradiance,
        sky_diffuse: Irradiance,
        ground_reflected: Irradiance,
    ) -> Self {
        let beam = beam.non_negative();
        let sky_diffuse = sky_diffuse.non_negative();
        let ground_reflected = ground_reflected.non_negative();
        let global = Irradiance::from_watts_per_m2(
            beam.watts_per_m2() + sky_diffuse.watts_per_m2() + ground_reflected.watts_per_m2(),
        );
        PlaneOfArrayIrradiance {
            beam,
            sky_diffuse,
            ground_reflected,
            global,
        }
    }
}

/// The beam (direct) component common to all four models:
/// `DNI * cos(angle_of_incidence)`, clamped to zero.
pub(crate) fn beam_component(inputs: &TranspositionInputs) -> Irradiance {
    Irradiance::from_watts_per_m2(
        inputs.dni.watts_per_m2() * inputs.angle_of_incidence.cos(),
    )
    .non_negative()
}

/// The ground-reflected (isotropic) component common to all four models:
/// `GHI * albedo * (1 - cos(tilt)) / 2`.
///
/// **Source**: Liu & Jordan (1963); reproduced identically in Hay-Davies,
/// HDKR, and Perez (the ground-reflected term is treated isotropically in
/// all four models).
pub(crate) fn ground_reflected_component(inputs: &TranspositionInputs) -> Irradiance {
    let tilt = inputs.surface.tilt;
    Irradiance::from_watts_per_m2(
        inputs.ghi.watts_per_m2() * inputs.ground_albedo.value() * (1.0 - tilt.cos()) / 2.0,
    )
    .non_negative()
}
