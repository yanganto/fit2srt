#[derive(Copy, Clone, Default)]
pub struct Summary {
    // It is a rough postion,
    // based on `start_position_lat`, `start_position_long`, and/or `end_position_lat`, `end_position_long`
    // fit use `Sint32`for positions
    pub location: (Option<i32>, Option<i32>),

    // Value for temperatures
    pub avg_temperature: Option<i8>,
    pub min_temperature: Option<i8>,

    // 0 for C
    pub temperature_unit: u8,

    // Value for depths
    pub avg_depth: Option<f64>,
    pub max_depth: Option<f64>,

    // 0 for m
    pub depth_unit: u8,

    // use total_elapsed_time to update average
    pub time: f64,
}

impl Summary {
    pub fn temp_unit(&self) -> &'static str {
        if self.temperature_unit == 0 {
            "C"
        } else {
            ""
        }
    }
    pub fn depth_unit(&self) -> &'static str {
        if self.depth_unit == 0 {
            "m"
        } else {
            ""
        }
    }

    pub fn is_empty(&self) -> bool {
        if self.location.0.is_some()
            || self.location.1.is_some()
            || self.avg_temperature.is_some()
            || self.min_temperature.is_some()
            || self.avg_depth.is_some()
            || self.max_depth.is_some()
        {
            false
        } else {
            true
        }
    }
    pub fn location(&self) -> Option<(f64, f64)> {
        if let (Some(lat), Some(long)) = self.location {
            Some((
                (lat as i64 * 180) as f64 / 2147483648f64,
                (long as i64 * 180) as f64 / 2147483648f64,
            ))
        } else {
            None
        }
    }

    pub fn set_unit(&mut self, u: &str) -> Result<(), crate::error::Fit2SrtError> {
        match u {
            "C" => {
                self.temperature_unit = 0;
            }
            "m" => {
                self.depth_unit = 0;
            }
            _ => {
                return Err(crate::error::Fit2SrtError::MergeError(format!(
                    "unsupport unit: {u:}"
                )));
            }
        }
        Ok(())
    }

    pub fn merge(self, other: &Self) -> Result<Self, crate::error::Fit2SrtError> {
        if self.temperature_unit != other.temperature_unit
            || self.depth_unit != other.temperature_unit
        {
            return Err(crate::error::Fit2SrtError::MergeError(
                "unit inconsist".to_string(),
            ));
        }

        let Summary {
            location,
            avg_temperature,
            min_temperature,
            temperature_unit,
            avg_depth,
            max_depth,
            depth_unit,
            time,
        } = self;

        let new_location = match (location, other.location) {
            ((Some(old_lat), Some(old_long)), (Some(other_lat), Some(other_long))) => (
                Some((old_lat + other_lat) / 2),
                Some((old_long + other_long) / 2),
            ),
            ((Some(old_lat), Some(old_long)), (None, None)) => (Some(old_lat), Some(old_long)),
            ((None, None), (Some(other_lat), Some(other_long))) => {
                (Some(other_lat), Some(other_long))
            }
            _ => (None, None),
        };

        let total_time = time + other.time;

        let new_avg_temperature = match (avg_temperature, other.avg_temperature) {
            (Some(old_avg_t), Some(other_avg_t)) => Some(
                ((old_avg_t as f64 * time + other_avg_t as f64 * other.time) / total_time) as i8,
            ),
            (Some(old_avg_t), None) => Some(old_avg_t),
            (None, Some(other_avg_t)) => Some(other_avg_t),
            _ => None,
        };

        let new_avg_depth = match (avg_depth, other.avg_depth) {
            (Some(old_avg_d), Some(other_avg_d)) => {
                Some((old_avg_d as f64 * time + other_avg_d as f64 * other.time) / total_time)
            }
            (Some(old_avg_d), None) => Some(old_avg_d),
            (None, Some(other_avg_d)) => Some(other_avg_d),
            _ => None,
        };

        let new_min_temperature = match (min_temperature, other.min_temperature) {
            (Some(old_min_t), Some(other_min_t)) => {
                if old_min_t < other_min_t {
                    Some(old_min_t)
                } else {
                    Some(other_min_t)
                }
            }
            (Some(old_min_t), None) => Some(old_min_t),
            (None, Some(other_min_t)) => Some(other_min_t),
            _ => None,
        };

        let new_max_depth = match (max_depth, other.max_depth) {
            (Some(old_max_d), Some(other_max_d)) => {
                if old_max_d < other_max_d {
                    Some(old_max_d)
                } else {
                    Some(other_max_d)
                }
            }
            (Some(old_max_d), None) => Some(old_max_d),
            (None, Some(other_max_d)) => Some(other_max_d),
            _ => None,
        };

        Ok(Self {
            location: new_location,
            avg_temperature: new_avg_temperature,
            min_temperature: new_min_temperature,
            avg_depth: new_avg_depth,
            max_depth: new_max_depth,
            temperature_unit,
            depth_unit,
            time: total_time,
        })
    }
}
