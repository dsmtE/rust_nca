use serde::{Serialize, Deserialize};
use std::fs::File;
use std::path::Path;
use std::io::{Read, Write};
use anyhow::{Context};

use crate::nca_app::DisplayFramesMode;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Preset {
    pub kernel: [f32; 9],
    pub activation_code: String,
    pub display_frames_mode: DisplayFramesMode,
}

impl Default for Preset {
    fn default() -> Self {
        Preset {
            kernel: [1., 1., 1., 1., 9., 1., 1., 1., 1.],
            activation_code: "fn activationFunction(kernelOutput: f32) -> vec4<f32> {
                return vec4<f32>(kernelOutput, kernelOutput, kernelOutput, 1.0);
            }".to_owned(),
            display_frames_mode: DisplayFramesMode::All,
        }
    }
}

pub fn load_preset<P: AsRef<Path>>(path: P) -> anyhow::Result<Preset> {

    let string_path: &str = path.as_ref().to_str().unwrap_or("");
    let mut file = File::open(path.as_ref())
    .with_context(|| format!("Could not open file `{}`", string_path))?;

    let mut buf = vec![];
    file.read_to_end(&mut buf)
    .with_context(|| format!("Could not read file `{}`", string_path))?;

    serde_json::from_slice(&buf[..])
    .with_context(|| format!("Unable to Parse the file `{}`", string_path))
}

pub fn save_preset<P: AsRef<Path>>(path: P, preset: &Preset) -> std::io::Result<()> {
    let mut f = File::create(path)?;
    let buf = serde_json::to_vec(preset)?;
    f.write_all(&buf[..])?;
    Ok(())
}