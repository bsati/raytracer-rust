use rand::Rng;

/// Enum representing different SuperSampling techniques
#[derive(Debug)]
#[allow(dead_code)]
pub enum SuperSampling {
    Uniform(usize),
    Jitter(usize),
}

impl SuperSampling {
    pub fn sample(&self, x: usize, y: usize) -> Vec<(f64, f64)> {
        match self {
            SuperSampling::Uniform(resolution) => {
                uniform_grid_sampling(*resolution, x as f64, y as f64)
            }
            SuperSampling::Jitter(resolution) => jitter_sampling(*resolution, x as f64, y as f64),
        }
    }
}

/// Error Type for Decoding a SuperSampling-Variant from a String
pub struct SSAADecodeError {
    error: String,
}

impl SSAADecodeError {
    fn new(error: String) -> SSAADecodeError {
        SSAADecodeError { error }
    }
}

impl std::error::Error for SSAADecodeError {}

impl std::fmt::Display for SSAADecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error parsing ssaa argument: {}", self.error)
    }
}

impl std::fmt::Debug for SSAADecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error parsing ssaa argument: {}", self.error)
    }
}

impl std::str::FromStr for SuperSampling {
    type Err = SSAADecodeError;

    fn from_str(s: &str) -> Result<SuperSampling, SSAADecodeError> {
        let method_args = s.split(":").collect::<Vec<_>>();
        match method_args[0] {
            "jitter" => {
                if method_args.len() < 2 {
                    return Err(SSAADecodeError::new(
                        "no arguments supplied, need resolution for jitter".to_string(),
                    ));
                }
                let resolution = method_args[1].parse::<usize>();
                match resolution {
                    Ok(res) => Ok(SuperSampling::Jitter(res)),
                    Err(_) => Err(SSAADecodeError::new(
                        "resolution has to be an integer".to_string(),
                    )),
                }
            }
            "uniform" => {
                if method_args.len() < 2 {
                    return Err(SSAADecodeError::new(
                        "no arguments supplied, need resolution for uniform".to_string(),
                    ));
                }
                let resolution = method_args[1].parse::<usize>();
                match resolution {
                    Ok(res) => Ok(SuperSampling::Jitter(res)),
                    Err(_) => Err(SSAADecodeError::new(
                        "resolution has to be an integer".to_string(),
                    )),
                }
            }
            _ => Err(SSAADecodeError::new("unknown method".to_string())),
        }
    }
}

/// Returns a list of points supersampled from a grid by uniform distribution
///
/// # Arguments
///
/// * `resolution` Resolution of the grid to span for the pixel
/// * `base_x` lower left x-coordinate of the grid
/// * `base_y` lower left y-coordinate of the grid
fn uniform_grid_sampling(resolution: usize, base_x: f64, base_y: f64) -> Vec<(f64, f64)> {
    let step: f64 = 1.0 / resolution as f64;
    let mut samples = Vec::with_capacity((resolution * resolution) as usize);
    for i in 0..resolution {
        for j in 0..resolution {
            samples.push((base_x + i as f64 * step, base_y + j as f64 * step));
        }
    }
    samples
}

/// Returns a list of points supersampled from a grid by jittering.
///
/// # Arguments
///
/// * `resolution` Resolution of the grid to span for the pixel
/// * `base_x` lower left x-coordinate of the grid
/// * `base_y` lower left y-coordinate of the grid
fn jitter_sampling(resolution: usize, base_x: f64, base_y: f64) -> Vec<(f64, f64)> {
    let mut rng = rand::thread_rng();
    let step: f64 = 1.0 / resolution as f64;
    let mut samples = Vec::with_capacity(resolution * resolution);
    for i in 0..resolution {
        for j in 0..resolution {
            samples.push((
                base_x + (i as f64 * step) + rng.gen_range(0.0..step),
                base_y + (j as f64 * step) + rng.gen_range(0.0..step),
            ));
        }
    }
    samples
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    #[test]
    fn test_grid_sampling_single() {
        let samples = super::uniform_grid_sampling(1, 0.0, 0.0);
        assert_eq!(
            samples.len(),
            1,
            "invalid sample count: expected 1 got {}",
            samples.len()
        );
        assert_eq!(
            samples[0],
            (0.0, 0.0),
            "invalid sample: expected (0.0, 0.0) got {:?}",
            samples[0]
        );
    }

    #[test]
    fn test_grid_sampling_multiple() {
        let samples = super::uniform_grid_sampling(2, 0.0, 0.0);
        assert_eq!(
            samples.len(),
            4,
            "invalid sample count: expected 4 got {}",
            samples.len()
        );
        assert_eq!(
            samples[0],
            (0.0, 0.0),
            "invalid samples: expected sample 0 to be (0.0, 0.0) got {:?}",
            samples[0]
        );
        assert_eq!(
            samples[1],
            (0.0, 0.5),
            "invalid samples: expected sample 1 to be (0.0, 0.5) got {:?}",
            samples[1]
        );
        assert_eq!(
            samples[2],
            (0.5, 0.0),
            "invalid samples: expected sample 2 to be (0.5, 0.0) got {:?}",
            samples[2]
        );
        assert_eq!(
            samples[3],
            (0.5, 0.5),
            "invalid samples: expected sample 3 to be (0.5, 0.5) got {:?}",
            samples[3]
        );
    }

    #[test]
    fn test_jitter_sampling() {
        let samples = super::jitter_sampling(2, 0.0, 0.0);
        assert_eq!(
            samples.len(),
            4,
            "invalid sample count: expected 4 got {}",
            samples.len()
        );
        assert!(
            samples[0].0 >= 0.0
                && samples[0].0 <= 0.5
                && samples[0].1 >= 0.0
                && samples[0].1 <= 0.5, "invalid samples: expected sample 0 to be (x, y) with x in [0.0, 0.5] and y in [0.0, 0.5] got {:?}", samples[0]
        );
        assert!(
            samples[1].0 >= 0.0
                && samples[1].0 <= 0.5
                && samples[1].1 >= 0.5
                && samples[1].1 <= 1.0, "invalid samples: expected sample 0 to be (x, y) with x in [0.0, 0.5] and y in [0.5, 1.0] got {:?}", samples[0]
        );
        assert!(
            samples[2].0 >= 0.5
                && samples[2].0 <= 1.0
                && samples[2].1 >= 0.0
                && samples[2].1 <= 0.5, "invalid samples: expected sample 0 to be (x, y) with x in [0.5, 1.0] and y in [0.0, 0.5] got {:?}", samples[0]
        );
        assert!(
            samples[3].0 >= 0.5
                && samples[3].0 <= 1.0
                && samples[3].1 >= 0.5
                && samples[3].1 <= 1.0, "invalid samples: expected sample 0 to be (x, y) with x in [0.5, 1.0] and y in [0.5, 1.0] got {:?}", samples[0]
        );
    }

    #[test]
    fn test_ssaa_uniform_from_str() {
        let valid_uniform = super::SuperSampling::from_str("uniform:2");
        let invalid_uniform = super::SuperSampling::from_str("uniform");
        let invalid_uniform_arg = super::SuperSampling::from_str("uniform:invalid");

        assert!(
            valid_uniform.is_ok(),
            "expected from_str to return ok got {:?}",
            valid_uniform.err().unwrap()
        );
        let valid_samples_len = valid_uniform.unwrap().sample(0, 0).len();

        assert_eq!(
            valid_samples_len,
            4,
            "expected resolution 2 got {}",
            f64::sqrt(valid_samples_len as f64)
        );

        assert!(
            invalid_uniform.is_err()
                && invalid_uniform.err().unwrap().error
                    == "no arguments supplied, need resolution for uniform",
            "expected no args error from invalid uniform"
        );

        assert!(
            invalid_uniform_arg.is_err()
                && invalid_uniform_arg.err().unwrap().error == "resolution has to be an integer",
            "expected wrong datatype error from invalid uniform"
        );
    }

    #[test]
    fn test_ssaa_jitter_from_str() {
        let valid_jitter = super::SuperSampling::from_str("jitter:2");
        let invalid_jitter = super::SuperSampling::from_str("jitter");
        let invalid_uniform_arg = super::SuperSampling::from_str("jitter:invalid");

        assert!(
            valid_jitter.is_ok(),
            "expected from_str to return ok got {:?}",
            valid_jitter.err().unwrap()
        );
        let valid_samples_len = valid_jitter.unwrap().sample(0, 0).len();

        assert_eq!(
            valid_samples_len,
            4,
            "expected resolution 2 got {}",
            f64::sqrt(valid_samples_len as f64)
        );

        assert!(
            invalid_jitter.is_err()
                && invalid_jitter.err().unwrap().error
                    == "no arguments supplied, need resolution for jitter",
            "expected no args error from invalid jitter"
        );

        assert!(
            invalid_uniform_arg.is_err()
                && invalid_uniform_arg.err().unwrap().error == "resolution has to be an integer",
            "expected wrong datatype error from invalid jitter"
        );
    }

    #[test]
    fn test_ssaa_invalid_method_from_str() {
        let invalid_method = super::SuperSampling::from_str("invalid");

        assert!(
            invalid_method.is_err() && invalid_method.err().unwrap().error == "unknown method",
            "expected unknown method error"
        )
    }
}
