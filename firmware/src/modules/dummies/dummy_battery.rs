use crate::{
    command_match,
    modules::{
        battery::{BatteryData, BatteryModuleCommands},
        module::{Module, ModuleError, ModuleKind, ModuleMetadata},
        system_controller::{CriticalEvent, SystemController},
    },
};
use anyhow::Result;
use std::{sync::Arc, time::Instant};
use uom::si::electric_current::ampere;
use uom::si::electric_potential::volt;
use uom::si::f64::{ElectricCurrent, ElectricPotential, ThermodynamicTemperature};
use uom::si::thermodynamic_temperature::degree_celsius;

pub struct DummyBatteryModule {
    id: u16,
    data: BatteryData,
    last_update: Instant,
    system_controller: Option<Arc<SystemController>>,
}

impl DummyBatteryModule {
    pub fn new(id: u16) -> Self {
        Self {
            data: BatteryData {
                charge: 100,
                voltage: ElectricPotential::new::<volt>(21.0),
                current: ElectricCurrent::new::<ampere>(0.0),
                temperature: ThermodynamicTemperature::new::<degree_celsius>(25.0),
                output_enabled: true,
            },
            id,
            last_update: Instant::now(),
            system_controller: None,
        }
    }

    fn is_charging(&self) -> bool {
        self.data.current.is_sign_positive()
            && self.data.voltage > ElectricPotential::new::<volt>(18.5)
    }

    /// **Helper function to update charge & voltage**
    fn update_charge_and_voltage(&mut self, delta_time: f64) {
        let mut net_current = ElectricCurrent::new::<ampere>(0.0);

        if self.data.output_enabled {
            net_current += ElectricCurrent::new::<ampere>(-0.5);
        }

        if self.is_charging() && self.data.charge < 100 {
            net_current += ElectricCurrent::new::<ampere>(1.5);
        }

        let net_mah = net_current.get::<ampere>() * delta_time * 1000.0;
        let percent_change = (net_mah / 6000.0) * 100.0;
        self.data.charge = (self.data.charge as f64 + percent_change).clamp(0.0, 100.0) as u8;

        self.data.voltage =
            ElectricPotential::new::<volt>(15.0 + (self.data.charge as f64 / 100.0) * 6.0);
        self.data.current = net_current;

        if self.data.charge < 10 {
            self.data.output_enabled = false;
        }
    }

    /// **Detects warnings and critical errors**
    fn detect_warnings_and_errors(&self) -> (Vec<BatteryModuleWarning>, Vec<BatteryModuleError>) {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        let charge = self.data.charge;
        let voltage = self.data.voltage.get::<volt>();
        let current = self.data.current.get::<ampere>();
        let temperature = self.data.temperature.get::<degree_celsius>();

        if current.abs() > 4.5 {
            warnings.push(BatteryModuleWarning::HighCurrentDraw);
        }
        if temperature > 40.0 {
            warnings.push(BatteryModuleWarning::HighTemperature);
        }
        if charge < 15 {
            warnings.push(BatteryModuleWarning::LowBattery);
        }

        if current.abs() > 5.0 {
            errors.push(BatteryModuleError::Overcurrent);
        }
        if voltage < 15.0 {
            errors.push(BatteryModuleError::Undervoltage);
        }
        if temperature > 50.0 {
            errors.push(BatteryModuleError::Overheating);
        }

        (warnings, errors)
    }

    /// **Refactored update_state function**
    pub fn update_state(&mut self) {
        let now = Instant::now();
        let delta_time = now.duration_since(self.last_update).as_secs_f64() / 3600.0;
        self.last_update = now;

        self.update_charge_and_voltage(delta_time);

        let controller = self.system_controller.clone().unwrap();

        // 🔥 Trigger Events for Critical Failures
        let (_, errors) = self.detect_warnings_and_errors();
        for error in errors {
            match error {
                BatteryModuleError::Overcurrent => {
                    controller.emit_event(crate::modules::system_controller::ModuleEvent::Critical(
                        CriticalEvent::OverCurrent(self.data.current),
                    ))
                }
                BatteryModuleError::Undervoltage => {
                    controller.emit_event(crate::modules::system_controller::ModuleEvent::Critical(
                        CriticalEvent::UnderVoltage(self.data.voltage),
                    ))
                }
                BatteryModuleError::Overheating => {
                    controller.emit_event(crate::modules::system_controller::ModuleEvent::Critical(
                        CriticalEvent::OverTemperature(self.data.temperature),
                    ))
                }
                BatteryModuleError::Overvoltage => {
                    controller.emit_event(
                        crate::modules::system_controller::ModuleEvent::Critical(
                            CriticalEvent::OverVoltage(self.data.voltage),
                        ),
                    );
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum BatteryModuleError {
    Overcurrent,
    Overvoltage,
    Undervoltage,
    Overheating,
}

#[derive(Debug)]
pub enum BatteryModuleWarning {
    HighCurrentDraw,
    HighTemperature,
    LowBattery,
}

pub struct BatteryModuleStatus {
    pub charge: u8,
    pub voltage: ElectricPotential,
    pub current: ElectricCurrent,
    pub temperature: ThermodynamicTemperature,
    pub warnings: Vec<BatteryModuleWarning>,
    pub errors: Vec<BatteryModuleError>,
    pub last_updated: Instant,
}

impl Module for DummyBatteryModule {
    type ModuleCommand = BatteryModuleCommands;
    type ModuleStatus = BatteryModuleStatus;

    fn metadata(&self) -> ModuleMetadata {
        ModuleMetadata {
            id: self.id,
            module_kind: ModuleKind::Battery,
            name: "Dummy Battery Module".into(),
            version: "1".into(),
        }
    }

    fn process_command(
        &mut self,
        command: Self::ModuleCommand,
    ) -> Result<Box<dyn std::any::Any>, ModuleError> {
        command_match!(command, crate::modules::battery, BatteryModuleCommands,
            SetOutput { state } => {
                dbg!(state);
            },
            GetVoltage {} => ElectricPotential::new::<volt>(0.0),
        )
    }

    fn status(&self) -> Self::ModuleStatus {
        let (warnings, errors) = self.detect_warnings_and_errors();

        BatteryModuleStatus {
            charge: self.data.charge,
            voltage: self.data.voltage,
            current: self.data.current,
            temperature: self.data.temperature,
            warnings,
            errors,
            last_updated: self.last_update,
        }
    }

    fn initialize(&mut self, system_controller: Arc<SystemController>) -> Result<(), ModuleError> {
        self.system_controller = Some(system_controller);
        Ok(())
    }
}
