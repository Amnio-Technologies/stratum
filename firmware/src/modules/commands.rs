use crate::modules::module::def_module_commands;

def_module_commands! {
    // Commands that can be sent to the battery module
    battery_module_commands {
        SetOutput(state: bool) -> ();
        GetVoltage() -> uom::si::f64::ElectricPotential;
        Dummy();
    }
}
