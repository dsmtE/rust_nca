use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::{fs::File, path::Path};

use crate::nca_app::{simulation_data::KernelSymmetryMode, DisplayFramesMode};
use egui_widgets::IqGradient;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Preset {
    pub kernel: [f32; 9],
    pub kernel_symmetry_mode: KernelSymmetryMode,
    pub activation_code: String,
    pub display_frames_mode: DisplayFramesMode,
    pub gradient: IqGradient,
}

impl Default for Preset {
    fn default() -> Self {
        Preset {
            kernel: [1., 1., 1., 1., 9., 1., 1., 1., 1.],
            kernel_symmetry_mode: KernelSymmetryMode::Any,
            activation_code: "fn activationFunction(kernelOutput: f32) -> vec4<f32> {
                return vec4<f32>(kernelOutput, kernelOutput, kernelOutput, 1.0);
            }"
            .to_owned(),
            display_frames_mode: DisplayFramesMode::All,
            gradient: IqGradient::default(),
        }
    }
}

pub fn load_preset<P: AsRef<Path>>(path: P) -> anyhow::Result<Preset> {
    fn inner(path: &Path) -> anyhow::Result<Preset>  {
        let string_path: &str = path.to_str().unwrap_or("");
        let file = File::open(path).with_context(|| format!("Could not open file `{}`", string_path))?;
        serde_json::from_reader(std::io::BufReader::new(file)).with_context(|| format!("Unable to Parse the file `{}`", string_path))
    }

    inner(path.as_ref())
}

pub fn save_preset<P: AsRef<Path>>(path: P, preset: &Preset) -> std::io::Result<()> { std::fs::write(path, serde_json::to_string_pretty(preset)?) }

lazy_static! {
    pub static ref PRESETS: std::collections::HashMap<&'static str, Preset> = std::collections::HashMap::from([
        (
            "Game Of life",
            Preset {
                kernel: [1., 1., 1., 1., 9., 1., 1., 1., 1.],
                kernel_symmetry_mode: KernelSymmetryMode::Any,
                activation_code: "
fn activationFunction(kernelOutput: vec4<f32>) -> vec4<f32> {
var condition: bool = kernelOutput.x == 3.0 || kernelOutput.x == 11.0 || kernelOutput.x == 12.0;
var r: f32 = select(0.0, 1.0, condition);
return vec4<f32>(r, r, r, 1.0);
}"
                .to_owned(),
                display_frames_mode: DisplayFramesMode::All,
                gradient: IqGradient::default(),
            },
        ),
        (
            "Slime",
            Preset {
                kernel: [0.8, -0.85, 0.8, -0.85, -0.2, -0.85, 0.8, -0.85, 0.8],
                kernel_symmetry_mode: KernelSymmetryMode::Any,
                activation_code: "
// an inverted gaussian function, 
// where f(0) = 0. 
// Graph: https://www.desmos.com/calculator/torawryxnq
fn activationFunction(kernelOutput: vec4<f32>) -> vec4<f32> {
var r: f32 = -1./(0.89*pow(kernelOutput.x, 2.)+1.)+1.;
return vec4<f32>(r, r, r, 1.0);
}"
                .to_owned(),
                display_frames_mode: DisplayFramesMode::Evens,
                gradient: IqGradient::default(),
            },
        ),
        (
            "Waves",
            Preset {
                kernel: [0.564599, -0.715900, 0.564599, -0.715900, 0.626900, -0.715900, 0.564599, -0.715900, 0.564599,],
                kernel_symmetry_mode: KernelSymmetryMode::Any,
                activation_code: "
fn activationFunction(kernelOutput: vec4<f32>) -> vec4<f32> {
var r: f32 = abs(1.2*kernelOutput.x);
return vec4<f32>(r, r, r, 1.0);
}"
                .to_owned(),
                display_frames_mode: DisplayFramesMode::All,
                gradient: IqGradient::default(),
            },
        ),
        (
            "Stars",
            Preset {
                kernel: [0.56459, -0.71590, 0.56459, -0.75859, 0.62690, -0.75859, 0.56459, -0.71590, 0.56459],
                kernel_symmetry_mode: KernelSymmetryMode::Any,
                activation_code: "
fn activationFunction(kernelOutput: vec4<f32>) -> vec4<f32> {
var r: f32 = abs(kernelOutput.x);
return vec4<f32>(r, r, r, 1.0);
}"
                .to_owned(),
                display_frames_mode: DisplayFramesMode::All,
                gradient: IqGradient::default(),
            },
        ),
        (
            "Pathways",
            Preset {
                kernel: [0., 1., 0., 1., 1., 1., 0., 1., 0.],
                kernel_symmetry_mode: KernelSymmetryMode::Any,
                activation_code: "
fn gaussian(x: f32, b: f32) -> f32{
return 1./pow(2., (pow(x-b, 2.)));
}

fn activationFunction(kernelOutput: vec4<f32>) -> vec4<f32> {
var r: f32 = gaussian(kernelOutput.x, 3.5);
return vec4<f32>(r, r, r, 1.0);
}"
                .to_owned(),
                display_frames_mode: DisplayFramesMode::All,
                gradient: IqGradient::default(),
            },
        ),
        (
            "Mitosis",
            Preset {
                kernel: [-0.939, 0.879, -0.939, 0.879, 0.4, 0.879, -0.939, 0.879, -0.939],
                kernel_symmetry_mode: KernelSymmetryMode::Any,
                activation_code: "
// an inverted gaussian function, 
// where f(0) = 0. 
// Graph: https://www.desmos.com/calculator/torawryxnqfn
fn activationFunction(kernelOutput: vec4<f32>) -> vec4<f32> {
var r: f32 = -1. / (0.9*pow(kernelOutput.x, 2.)+1.)+1.;
return vec4<f32>(r, r, r, 1.0);
}"
                .to_owned(),
                display_frames_mode: DisplayFramesMode::All,
                gradient: IqGradient::default(),
            },
        ),
        (
            "Blob",
            Preset {
                kernel: [
                    0.7795687913894653,
                    -0.7663648128509521,
                    0.7795687913894653,
                    -0.7663648128509521,
                    -0.29899999499320984,
                    -0.7663648128509521,
                    0.7795687913894653,
                    -0.7663648128509521,
                    0.7795687913894653,
                ],
                kernel_symmetry_mode: KernelSymmetryMode::Any,
                activation_code: "
fn activationFunction(kernelOutput: vec4<f32>) -> vec4<f32> {
var r: f32 = -1. / pow(2., (pow(kernelOutput.x, 2.)))+1.;
return vec4<f32>(r, r, r, 1.0);
}"
                .to_owned(),
                display_frames_mode: DisplayFramesMode::All,
                gradient: IqGradient::default(),
            },
        ),
        (
            "test",
            Preset {
                kernel: [
                    0.5669999718666077,
                    -0.7149999737739563,
                    0.5669999718666077,
                    -0.7149999737739563,
                    0.6370000243186951,
                    -0.7149999737739563,
                    0.5669999718666077,
                    -0.7149999737739563,
                    0.5669999718666077,
                ],
                kernel_symmetry_mode: KernelSymmetryMode::Any,
                activation_code: "
fn activationFunction(kernelOutput: vec4<f32>) -> vec4<f32> {
var r: f32 = abs(kernelOutput.x);
return vec4<f32>(r, r, r, 1.0);
}"
                .to_owned(),
                display_frames_mode: DisplayFramesMode::Evens,
                gradient: IqGradient::default(),
            },
        ),
        (
            "test2",
            Preset {
                kernel: [
                    91.627685546875,
                    -59.281097412109375,
                    91.627685546875,
                    -59.281097412109375,
                    -42.35920715332031,
                    -59.281097412109375,
                    91.627685546875,
                    -59.281097412109375,
                    91.627685546875,
                ],
                kernel_symmetry_mode: KernelSymmetryMode::Any,
                activation_code: "
fn activationFunction(kernelOutput: vec4<f32>) -> vec4<f32> {
var r: f32 = (exp(2.*kernelOutput.x) - 1.) / (exp(2.*kernelOutput.x) + 1.);
return vec4<f32>(r, r, r, 1.0);
}"
                .to_owned(),
                display_frames_mode: DisplayFramesMode::Evens,
                gradient: IqGradient::default(),
            },
        ),
    ]);
}
