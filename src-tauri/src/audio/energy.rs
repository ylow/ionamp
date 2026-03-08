use realfft::RealFftPlanner;

const ENERGY_VECTOR_LEN: usize = 128;

/// Three separate 128-point energy component vectors with absolute values.
pub struct EnergyComponents {
    pub rms: Vec<f32>,
    pub centroid: Vec<f32>,
    pub onset: Vec<f32>,
}

/// Computes 128-point RMS, spectral centroid, and onset strength vectors.
/// Values are absolute (unnormalized) and comparable across tracks.
pub fn compute_energy_components(samples: &[f32]) -> EnergyComponents {
    let empty = EnergyComponents {
        rms: vec![0.0; ENERGY_VECTOR_LEN],
        centroid: vec![0.0; ENERGY_VECTOR_LEN],
        onset: vec![0.0; ENERGY_VECTOR_LEN],
    };

    if samples.is_empty() {
        return empty;
    }

    let segment_len = samples.len() / ENERGY_VECTOR_LEN;
    if segment_len == 0 {
        let mut rms_vec = vec![0.0; ENERGY_VECTOR_LEN];
        for (i, &s) in samples.iter().enumerate() {
            let idx = i * ENERGY_VECTOR_LEN / samples.len();
            rms_vec[idx] += s * s;
        }
        return EnergyComponents {
            rms: rms_vec,
            centroid: vec![0.0; ENERGY_VECTOR_LEN],
            onset: vec![0.0; ENERGY_VECTOR_LEN],
        };
    }

    let fft_size = segment_len.next_power_of_two().max(16);
    let mut planner = RealFftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(fft_size);
    let mut fft_input = vec![0.0f32; fft_size];
    let mut fft_output = fft.make_output_vec();
    let spectrum_len = fft_output.len();

    let mut rms_vec = Vec::with_capacity(ENERGY_VECTOR_LEN);
    let mut centroid_vec = Vec::with_capacity(ENERGY_VECTOR_LEN);
    let mut onset_vec = Vec::with_capacity(ENERGY_VECTOR_LEN);

    for seg_idx in 0..ENERGY_VECTOR_LEN {
        let start = seg_idx * segment_len;
        let end = (start + segment_len).min(samples.len());
        let segment = &samples[start..end];

        rms_vec.push(rms_of(segment));

        fft_input.fill(0.0);
        for (i, &s) in segment.iter().enumerate().take(fft_size) {
            fft_input[i] = s;
        }
        let _ = fft.process(&mut fft_input, &mut fft_output);
        centroid_vec.push(spectral_centroid(&fft_output) / spectrum_len.max(1) as f32);

        onset_vec.push(onset_strength(segment));
    }

    EnergyComponents {
        rms: rms_vec,
        centroid: centroid_vec,
        onset: onset_vec,
    }
}

fn rms_of(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = samples.iter().map(|&s| s * s).sum();
    (sum_sq / samples.len() as f32).sqrt()
}

fn spectral_centroid(spectrum: &[realfft::num_complex::Complex<f32>]) -> f32 {
    let mut weighted_sum = 0.0f64;
    let mut magnitude_sum = 0.0f64;

    for (i, c) in spectrum.iter().enumerate() {
        let mag = (c.re * c.re + c.im * c.im).sqrt() as f64;
        weighted_sum += i as f64 * mag;
        magnitude_sum += mag;
    }

    if magnitude_sum < 1e-10 {
        0.0
    } else {
        (weighted_sum / magnitude_sum) as f32
    }
}

fn onset_strength(segment: &[f32]) -> f32 {
    let sub_window_count = 8;
    let sub_len = segment.len() / sub_window_count;
    if sub_len == 0 {
        return 0.0;
    }

    let sub_rms: Vec<f32> = (0..sub_window_count)
        .map(|i| {
            let start = i * sub_len;
            let end = (start + sub_len).min(segment.len());
            rms_of(&segment[start..end])
        })
        .collect();

    if sub_rms.len() < 2 {
        return 0.0;
    }

    let diffs: f32 = sub_rms.windows(2).map(|w| (w[1] - w[0]).abs()).sum();
    diffs / (sub_rms.len() - 1) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_vector_lengths() {
        let samples = vec![0.5f32; 44100];
        let ec = compute_energy_components(&samples);
        assert_eq!(ec.rms.len(), ENERGY_VECTOR_LEN);
        assert_eq!(ec.centroid.len(), ENERGY_VECTOR_LEN);
        assert_eq!(ec.onset.len(), ENERGY_VECTOR_LEN);
    }

    #[test]
    fn test_components_non_negative() {
        let samples: Vec<f32> = (0..44100).map(|i| (i as f32 * 0.01).sin()).collect();
        let ec = compute_energy_components(&samples);
        for &v in ec.rms.iter().chain(ec.centroid.iter()).chain(ec.onset.iter()) {
            assert!(v >= 0.0, "value {} is negative", v);
        }
        assert!(ec.rms.iter().any(|&v| v > 0.0));
    }

    #[test]
    fn test_silence_is_zero() {
        let samples = vec![0.0f32; 44100];
        let ec = compute_energy_components(&samples);
        for &v in ec.rms.iter().chain(ec.centroid.iter()).chain(ec.onset.iter()) {
            assert_eq!(v, 0.0);
        }
    }

    #[test]
    fn test_loud_vs_quiet_rms() {
        let loud: Vec<f32> = (0..44100).map(|i| (i as f32 * 0.1).sin() * 0.9).collect();
        let quiet: Vec<f32> = (0..44100).map(|i| (i as f32 * 0.1).sin() * 0.1).collect();
        let loud_ec = compute_energy_components(&loud);
        let quiet_ec = compute_energy_components(&quiet);
        let loud_avg: f32 = loud_ec.rms.iter().sum::<f32>() / loud_ec.rms.len() as f32;
        let quiet_avg: f32 = quiet_ec.rms.iter().sum::<f32>() / quiet_ec.rms.len() as f32;
        assert!(loud_avg > quiet_avg * 2.0);
    }

    #[test]
    fn test_energy_bytes_roundtrip() {
        use crate::db::tracks::{energy_from_bytes, energy_to_bytes};
        let original = vec![0.0f32, 0.25, 0.5, 0.75, 1.0];
        let bytes = energy_to_bytes(&original);
        let restored = energy_from_bytes(&bytes).unwrap();
        assert_eq!(original, restored);
    }

    #[test]
    fn test_varying_signal_rms() {
        let samples: Vec<f32> = (0..44100)
            .map(|i| {
                let t = i as f32 / 44100.0;
                (i as f32 * 0.1).sin() * t
            })
            .collect();
        let ec = compute_energy_components(&samples);
        let first_quarter: f32 = ec.rms[..32].iter().sum::<f32>() / 32.0;
        let last_quarter: f32 = ec.rms[96..].iter().sum::<f32>() / 32.0;
        assert!(last_quarter > first_quarter);
    }

    #[test]
    fn test_short_input() {
        let samples = vec![0.5f32; 10];
        let ec = compute_energy_components(&samples);
        assert_eq!(ec.rms.len(), ENERGY_VECTOR_LEN);
    }
}
