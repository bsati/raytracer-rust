use rand::Rng;

#[derive(Debug)]
#[allow(dead_code)]
pub enum SuperSampling {
    Uniform(u8),
    Jitter(u8),
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

pub struct SSAADecodeError {
    error: String,
}

impl SSAADecodeError {
    fn new(error: String) -> SSAADecodeError {
        SSAADecodeError { error: error }
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
                let resolution = method_args[1].parse::<u8>();
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
                let resolution = method_args[1].parse::<u8>();
                match resolution {
                    Ok(res) => Ok(SuperSampling::Jitter(res)),
                    Err(_) => Err(SSAADecodeError::new(
                        "resolution has to be an integer".to_string(),
                    )),
                }
            }
            _ => Err(SSAADecodeError::new("unkown method".to_string())),
        }
    }
}

fn uniform_grid_sampling(resolution: u8, base_x: f64, base_y: f64) -> Vec<(f64, f64)> {
    let step: f64 = 1.0 / resolution as f64;
    let mut samples = Vec::with_capacity((resolution * resolution) as usize);
    for i in 0..resolution {
        for j in 0..resolution {
            samples.push((base_x + i as f64 * step, base_y + j as f64 * step));
        }
    }
    samples
}

fn jitter_sampling(resolution: u8, base_x: f64, base_y: f64) -> Vec<(f64, f64)> {
    let mut rng = rand::thread_rng();
    let step: f64 = 1.0 / resolution as f64;
    let mut samples = Vec::with_capacity((resolution * resolution) as usize);
    for i in 1..resolution + 1 {
        for j in 1..resolution + 1 {
            samples.push((
                rng.gen_range(base_x..(base_x + i as f64 * step)),
                rng.gen_range(base_y..(base_y + j as f64 * step)),
            ));
        }
    }
    samples
}
