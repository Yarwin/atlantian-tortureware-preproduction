use serde::{Deserialize, Serialize};
use crate::ai::ai_stimulus::AIStimulusType;
use crate::sensors::sensor_types::{SensorArguments, SensorEvent};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageSensor {}


impl SensorEvent for DamageSensor {
    fn stimulate(&mut self, _stim: AIStimulusType, _args: &mut SensorArguments) -> bool {
        true
    }
}
