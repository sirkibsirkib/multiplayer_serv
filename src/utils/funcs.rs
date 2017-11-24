pub fn sigmoid(x : f32, amplifier : f32) -> f32 {
    let o = 1.0 /
    (1.0 + ::std::f32::consts::E.powf(-x*amplifier));
    2.0 * (o - 0.5)
}

pub fn sig_0_pt5(x : f32, amplifier : f32) -> f32 {
    sigmoid(x * 2.0 - 1.0, amplifier) * 0.5 + 0.5
}
