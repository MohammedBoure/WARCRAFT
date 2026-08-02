const HASH_X: u64 = 0x9E37_79B9_7F4A_7C15;
const HASH_Y: u64 = 0xBF58_476D_1CE4_E5B9;
const HASH_Z: u64 = 0x94D0_49BB_1331_11EB;

pub fn hash3(seed: u64, x: i64, y: i64, z: i64, channel: u64) -> u64 {
    let mut value = seed
        ^ (x as u64).wrapping_mul(HASH_X)
        ^ (y as u64).wrapping_mul(HASH_Y)
        ^ (z as u64).wrapping_mul(HASH_Z)
        ^ channel.wrapping_mul(0xD6E8_FD9D_AA29_1235);

    splitmix64(&mut value)
}

pub fn unit3(seed: u64, x: i64, y: i64, z: i64, channel: u64) -> f64 {
    let value = hash3(seed, x, y, z, channel) >> 11;

    (value as f64) / ((1_u64 << 53) as f64)
}

pub fn fbm2(seed: u64, x: f64, z: f64, octaves: usize, channel: u64) -> f64 {
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut total = 0.0;
    let mut total_amplitude = 0.0;

    for octave in 0..octaves.max(1) {
        total += value_noise2(
            seed,
            x * frequency,
            z * frequency,
            channel + octave as u64 * 17,
        ) * amplitude;
        total_amplitude += amplitude;
        amplitude *= 0.52;
        frequency *= 2.03;
    }

    (total / total_amplitude.max(f64::EPSILON)).clamp(0.0, 1.0)
}

pub fn ridged2(seed: u64, x: f64, z: f64, octaves: usize, channel: u64) -> f64 {
    let base = fbm2(seed, x, z, octaves, channel);

    (1.0 - (base * 2.0 - 1.0).abs()).clamp(0.0, 1.0)
}

pub fn fbm3(seed: u64, x: f64, y: f64, z: f64, octaves: usize, channel: u64) -> f64 {
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut total = 0.0;
    let mut total_amplitude = 0.0;

    for octave in 0..octaves.max(1) {
        total += value_noise3(
            seed,
            x * frequency,
            y * frequency,
            z * frequency,
            channel + octave as u64 * 23,
        ) * amplitude;
        total_amplitude += amplitude;
        amplitude *= 0.50;
        frequency *= 2.0;
    }

    (total / total_amplitude.max(f64::EPSILON)).clamp(0.0, 1.0)
}

fn value_noise2(seed: u64, x: f64, z: f64, channel: u64) -> f64 {
    let x0 = x.floor() as i64;
    let z0 = z.floor() as i64;
    let xf = smooth_fraction(x - x.floor());
    let zf = smooth_fraction(z - z.floor());

    let a = unit3(seed, x0, 0, z0, channel);
    let b = unit3(seed, x0 + 1, 0, z0, channel);
    let c = unit3(seed, x0, 0, z0 + 1, channel);
    let d = unit3(seed, x0 + 1, 0, z0 + 1, channel);

    lerp(lerp(a, b, xf), lerp(c, d, xf), zf)
}

fn value_noise3(seed: u64, x: f64, y: f64, z: f64, channel: u64) -> f64 {
    let x0 = x.floor() as i64;
    let y0 = y.floor() as i64;
    let z0 = z.floor() as i64;
    let xf = smooth_fraction(x - x.floor());
    let yf = smooth_fraction(y - y.floor());
    let zf = smooth_fraction(z - z.floor());

    let n000 = unit3(seed, x0, y0, z0, channel);
    let n100 = unit3(seed, x0 + 1, y0, z0, channel);
    let n010 = unit3(seed, x0, y0 + 1, z0, channel);
    let n110 = unit3(seed, x0 + 1, y0 + 1, z0, channel);
    let n001 = unit3(seed, x0, y0, z0 + 1, channel);
    let n101 = unit3(seed, x0 + 1, y0, z0 + 1, channel);
    let n011 = unit3(seed, x0, y0 + 1, z0 + 1, channel);
    let n111 = unit3(seed, x0 + 1, y0 + 1, z0 + 1, channel);

    let x00 = lerp(n000, n100, xf);
    let x10 = lerp(n010, n110, xf);
    let x01 = lerp(n001, n101, xf);
    let x11 = lerp(n011, n111, xf);
    let y0 = lerp(x00, x10, yf);
    let y1 = lerp(x01, x11, yf);

    lerp(y0, y1, zf)
}

fn smooth_fraction(value: f64) -> f64 {
    let value = value.clamp(0.0, 1.0);

    value * value * value * (value * (value * 6.0 - 15.0) + 10.0)
}

fn lerp(from: f64, to: f64, t: f64) -> f64 {
    from + (to - from) * t.clamp(0.0, 1.0)
}

fn splitmix64(value: &mut u64) -> u64 {
    *value = value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *value;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}
