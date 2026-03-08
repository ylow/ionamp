use realfft::RealFftPlanner;

const ENERGY_VECTOR_LEN: usize = 128;

/// Computes a 128-point energy vector with absolute (unnormalized) values.
/// Values are comparable across tracks. Normalization should be done at
/// render time across all displayed tracks.
pub fn compute_energy_vector(samples: &[f32]) -> Vec<f32> {
    if samples.is_empty() {
        return vec![0.0; ENERGY_VECTOR_LEN];
    }

    let segment_len = samples.len() / ENERGY_VECTOR_LEN;
    if segment_len == 0 {
        // Fewer samples than segments: spread what we have
        let mut result = vec![0.0; ENERGY_VECTOR_LEN];
        for (i, &s) in samples.iter().enumerate() {
            let idx = i * ENERGY_VECTOR_LEN / samples.len();
            result[idx] += s * s;
        }
        return result;
    }

    // FFT planner — pick a power-of-2 window size for efficiency
    let fft_size = segment_len.next_power_of_two().max(16);
    let mut planner = RealFftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(fft_size);
    let mut fft_input = vec![0.0f32; fft_size];
    let mut fft_output = fft.make_output_vec();
    let spectrum_len = fft_output.len();

    let mut result = Vec::with_capacity(ENERGY_VECTOR_LEN);

    for seg_idx in 0..ENERGY_VECTOR_LEN {
        let start = seg_idx * segment_len;
        let end = (start + segment_len).min(samples.len());
        let segment = &samples[start..end];

        // RMS — absolute, naturally in [0, ~1] for float audio
        let rms = rms_of(segment);

        // Spectral centroid via FFT, normalized by spectrum length to [0, 1]
        fft_input.fill(0.0);
        for (i, &s) in segment.iter().enumerate().take(fft_size) {
            fft_input[i] = s;
        }
        let _ = fft.process(&mut fft_input, &mut fft_output);
        let centroid = spectral_centroid(&fft_output) / spectrum_len.max(1) as f32;

        // Onset strength — absolute
        let onset = onset_strength(segment);

        // Blend with absolute values — no normalization
        result.push(0.5 * rms + 0.3 * centroid + 0.2 * onset);
    }

    result
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
    // Divide segment into sub-windows and compute RMS of each
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

    // Mean absolute difference between consecutive sub-window RMS values
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
    fn test_energy_vector_length() {
        let samples = vec![0.5f32; 44100]; // 1 second at 44100
        let energy = compute_energy_vector(&samples);
        assert_eq!(energy.len(), ENERGY_VECTOR_LEN);
    }

    #[test]
    fn test_energy_vector_non_negative() {
        let samples: Vec<f32> = (0..44100).map(|i| (i as f32 * 0.01).sin()).collect();
        let energy = compute_energy_vector(&samples);
        for &v in &energy {
            assert!(v >= 0.0, "value {} is negative", v);
        }
        // Should have some non-zero energy
        assert!(energy.iter().any(|&v| v > 0.0));
    }

    #[test]
    fn test_silence_is_zero() {
        let samples = vec![0.0f32; 44100];
        let energy = compute_energy_vector(&samples);
        assert_eq!(energy.len(), ENERGY_VECTOR_LEN);
        for &v in &energy {
            assert_eq!(v, 0.0);
        }
    }

    #[test]
    fn test_loud_vs_quiet_signal() {
        let loud: Vec<f32> = (0..44100).map(|i| (i as f32 * 0.1).sin() * 0.9).collect();
        let quiet: Vec<f32> = (0..44100).map(|i| (i as f32 * 0.1).sin() * 0.1).collect();
        let loud_energy = compute_energy_vector(&loud);
        let quiet_energy = compute_energy_vector(&quiet);
        let loud_avg: f32 = loud_energy.iter().sum::<f32>() / loud_energy.len() as f32;
        let quiet_avg: f32 = quiet_energy.iter().sum::<f32>() / quiet_energy.len() as f32;
        // Loud signal should have higher absolute energy than quiet one
        assert!(
            loud_avg > quiet_avg * 2.0,
            "loud_avg ({}) should be much greater than quiet_avg ({})",
            loud_avg,
            quiet_avg
        );
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
    fn test_varying_signal() {
        // Signal that ramps up from silence to loud
        let samples: Vec<f32> = (0..44100)
            .map(|i| {
                let t = i as f32 / 44100.0;
                (i as f32 * 0.1).sin() * t
            })
            .collect();
        let energy = compute_energy_vector(&samples);

        // Last quarter should have higher average energy than first quarter
        let first_quarter: f32 = energy[..32].iter().sum::<f32>() / 32.0;
        let last_quarter: f32 = energy[96..].iter().sum::<f32>() / 32.0;
        assert!(
            last_quarter > first_quarter,
            "last_quarter ({}) should > first_quarter ({})",
            last_quarter,
            first_quarter
        );
    }

    #[test]
    fn test_short_input() {
        // Fewer than 128 samples
        let samples = vec![0.5f32; 10];
        let energy = compute_energy_vector(&samples);
        assert_eq!(energy.len(), ENERGY_VECTOR_LEN);
    }
}
