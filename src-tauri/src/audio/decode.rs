use std::path::Path;

use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

#[derive(Debug)]
pub struct DecodedAudio {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("no audio track found")]
    NoAudioTrack,
    #[error("symphonia error: {0}")]
    Symphonia(#[from] symphonia::core::errors::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

const TARGET_SAMPLE_RATE: u32 = 22050;

struct RawDecode {
    samples: Vec<f32>,
    sample_rate: u32,
    channels: usize,
}

/// Decode all packets from a file, returning interleaved samples at original rate/channels.
fn decode_raw(path: &Path) -> Result<RawDecode, DecodeError> {
    let file = std::fs::File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe().format(
        &hint,
        mss,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    )?;

    let mut format = probed.format;

    let track = format
        .tracks()
        .iter()
        .find(|t| {
            t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL
        })
        .ok_or(DecodeError::NoAudioTrack)?;

    let track_id = track.id;
    let channels = track.codec_params.channels.map(|c| c.count()).unwrap_or(1);
    let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);

    let mut decoder =
        symphonia::default::get_codecs().make(&track.codec_params, &DecoderOptions::default())?;

    let mut all_samples: Vec<f32> = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(symphonia::core::errors::Error::IoError(ref e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(e) => return Err(e.into()),
        };

        if packet.track_id() != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
            Err(e) => return Err(e.into()),
        };

        let spec = *decoded.spec();
        let num_frames = decoded.frames();
        let mut sample_buf = SampleBuffer::<f32>::new(num_frames as u64, spec);
        sample_buf.copy_interleaved_ref(decoded);
        all_samples.extend_from_slice(sample_buf.samples());
    }

    Ok(RawDecode {
        samples: all_samples,
        sample_rate,
        channels,
    })
}

/// Decode to mono PCM at 22050 Hz for energy analysis.
pub fn decode_to_mono_pcm(path: &Path) -> Result<DecodedAudio, DecodeError> {
    let raw = decode_raw(path)?;

    // Mix to mono
    let mono = if raw.channels > 1 {
        raw.samples
            .chunks_exact(raw.channels)
            .map(|frame| frame.iter().sum::<f32>() / raw.channels as f32)
            .collect()
    } else {
        raw.samples
    };

    // Resample to target rate
    let samples = if raw.sample_rate != TARGET_SAMPLE_RATE {
        resample_linear(&mono, raw.sample_rate, TARGET_SAMPLE_RATE)
    } else {
        mono
    };

    Ok(DecodedAudio {
        samples,
        sample_rate: TARGET_SAMPLE_RATE,
        channels: 1,
    })
}

/// Decode to PCM at original sample rate and channel count for playback.
pub fn decode_to_pcm(path: &Path) -> Result<DecodedAudio, DecodeError> {
    let raw = decode_raw(path)?;
    Ok(DecodedAudio {
        samples: raw.samples,
        sample_rate: raw.sample_rate,
        channels: raw.channels,
    })
}

fn resample_linear(input: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if input.is_empty() {
        return Vec::new();
    }
    let ratio = from_rate as f64 / to_rate as f64;
    let output_len = (input.len() as f64 / ratio).ceil() as usize;
    let mut output = Vec::with_capacity(output_len);

    for i in 0..output_len {
        let src_pos = i as f64 * ratio;
        let idx = src_pos as usize;
        let frac = src_pos - idx as f64;

        let sample = if idx + 1 < input.len() {
            input[idx] as f64 * (1.0 - frac) + input[idx + 1] as f64 * frac
        } else {
            input[idx.min(input.len() - 1)] as f64
        };
        output.push(sample as f32);
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_mp3() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/test.mp3");
        let audio = decode_to_mono_pcm(&path).unwrap();
        assert_eq!(audio.sample_rate, TARGET_SAMPLE_RATE);
        assert!(!audio.samples.is_empty());
    }

    #[test]
    fn test_decode_flac() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/test.flac");
        let audio = decode_to_mono_pcm(&path).unwrap();
        assert_eq!(audio.sample_rate, TARGET_SAMPLE_RATE);
        assert!(!audio.samples.is_empty());
    }

    #[test]
    fn test_mono_output() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/test.mp3");
        let audio = decode_to_mono_pcm(&path).unwrap();
        // ~1 second at 22050 Hz should give us roughly 22050 samples
        let expected_min = (TARGET_SAMPLE_RATE as f64 * 0.8) as usize;
        let expected_max = (TARGET_SAMPLE_RATE as f64 * 1.5) as usize;
        assert!(
            audio.samples.len() > expected_min && audio.samples.len() < expected_max,
            "Expected ~{} samples, got {}",
            TARGET_SAMPLE_RATE,
            audio.samples.len()
        );
    }
}
