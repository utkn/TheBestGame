mod vehicle_bundle;
mod vehicle_insights;
mod vehicle_interaction;
mod vehicle_system;
mod vehicle_tags;

pub use vehicle_bundle::*;
pub use vehicle_insights::*;
pub use vehicle_interaction::*;
pub use vehicle_system::*;

#[derive(Clone, Copy, Debug)]
pub struct Vehicle;
