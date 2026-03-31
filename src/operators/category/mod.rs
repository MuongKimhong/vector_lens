pub mod io;
pub mod cleaning;
pub mod deep_learning;
pub mod machine_learning;

pub use io::*;
pub use cleaning::*;
pub use deep_learning::*;
pub use machine_learning::*;

use super::*;

// Things need to do when creating new built in operator:
// 1. add operator function in operators/category/ and its execution handler
// 2. Update OperatorKind in operators/mod.rs
// 3. Update OperatorList resource in resources.rs to include new operator in create_default_operators()
// 4. Update PropertyType enum in ui/widgets/property_panel.rs
// 5. Create property container in property_containers.rs
// 6. Update on_property_btn_clicked() in ui/widgets/operator_context.rs to return PropertyType
