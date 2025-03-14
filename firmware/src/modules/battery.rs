use uom::si::f64::{ElectricCurrent, ElectricPotential, ThermodynamicTemperature};

/// Represents the state of a battery module.
pub struct BatteryData {
    /// Battery charge level in percentage (0-100)%.
    ///
    /// - **100% = Fully charged** (eg ~14.8V).
    /// - **10% = Near empty** (eg ~10.5V).
    /// - **0% = Deep discharge state** (output disabled).
    pub charge: u8,

    /// Battery voltage, which varies based on charge level.
    /// - Directly affects system power availability.
    /// - **Higher charge = higher voltage**.
    pub voltage: ElectricPotential,

    /// Battery current flow (negative = discharge, positive = charge).
    pub current: ElectricCurrent,

    pub temperature: ThermodynamicTemperature,

    /// Determines if the battery is supplying power (`true = enabled`).
    pub output_enabled: bool,
}
