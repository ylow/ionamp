use realfft::RealFftPlanner;

const ENERGY_VECTOR_LEN: usize = 128;

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
        normalize_vec(&mut result);
        return result;
    }

    let mut rms_values = Vec::with_capacity(ENERGY_VECTOR_LEN);
    let mut centroid_values = Vec::with_capacity(ENERGY_VECTOR_LEN);
    let mut onset_values = Vec::with_capacity(ENERGY_VECTOR_LEN);

    // FFT planner — pick a power-of-2 window size for efficiency
    let fft_size = segment_len.next_power_of_two().max(16);
    let mut planner = RealFftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(fft_size);
    let mut fft_input = vec![0.0f32; fft_size];
    let mut fft_output = fft.make_output_vec();

    for seg_idx in 0..ENERGY_VECTOR_LEN {
        let start = seg_idx * segment_len;
        let end = (start + segment_len).min(samples.len());
        let segment = &samples[start..end];

        // RMS
        let rms = rms_of(segment);
        rms_values.push(rms);

        // Spectral centroid via FFT
        fft_input.fill(0.0);
        for (i, &s) in segment.iter().enumerate().take(fft_size) {
            fft_input[i] = s;
        }
        let _ = fft.process(&mut fft_input, &mut fft_output);

        let centroid = spectral_centroid(&fft_output);
        centroid_values.push(centroid);

        // Onset strength: mean absolute difference of sub-window RMS values
        let onset = onset_strength(segment);
        onset_values.push(onset);
    }

    // Normalize each component to [0, 1]
    normalize_vec(&mut rms_values);
    normalize_vec(&mut centroid_values);
    normalize_vec(&mut onset_values);

    // Blend: 0.5*RMS + 0.3*centroid + 0.2*onset
    let mut result: Vec<f32> = rms_values
        .iter()
        .zip(centroid_values.iter())
        .zip(onset_values.iter())
        .map(|((&r, &c), &o)| 0.5 * r + 0.3 * c + 0.2 * o)
        .collect();

    normalize_vec(&mut result);
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

fn normalize_vec(v: &mut [f32]) {
    let max = v.iter().cloned().fold(0.0f32, f32::max);
    if max > 1e-10 {
        for x in v.iter_mut() {
            *x /= max;
        }
    }
}

pub fn energy_to_bytes(energy: &[f32]) -> Vec<u8> {
    energy.iter().flat_map(|f| f.to_le_bytes()).collect()
}

pub fn energy_from_bytes(bytes: &[u8]) -> Option<Vec<f32>> {
    if bytes.len() % 4 != 0 {
        return None;
    }
    Some(
        bytes
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes(chunk.try_into().unwrap()))
            .collect(),
    )
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
    fn test_energy_vector_normalized() {
        let samples: Vec<f32> = (0..44100).map(|i| (i as f32 * 0.01).sin()).collect();
        let energy = compute_energy_vector(&samples);
        for &v in &energy {
            assert!(v >= 0.0 && v <= 1.0, "value {} not in [0, 1]", v);
        }
        // At least one value should be 1.0 (max after normalization)
        assert!(energy.iter().any(|&v| (v - 1.0).abs() < 0.01));
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
    fn test_loud_signal_has_high_energy() {
        let loud: Vec<f32> = (0..44100).map(|i| (i as f32 * 0.1).sin() * 0.9).collect();
        let energy = compute_energy_vector(&loud);
        let avg: f32 = energy.iter().sum::<f32>() / energy.len() as f32;
        // A loud uniform signal should have high average energy
        assert!(avg > 0.3, "avg energy {} too low for loud signal", avg);
    }

    #[test]
    fn test_energy_bytes_roundtrip() {
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
