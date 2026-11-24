use cactus_rs::whistle::{Language, TranscribeOptions, Weights, Whistle, SAMPLE_RATE};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).unwrap_or_else(|| "../test.wav".into());
    let pcm = read_wav(&path)?;

    let mut whistle = Whistle::builder(Weights::fetch()?).build()?;
    let options = TranscribeOptions::new()
        .with_language("fr".parse::<Language>()?)
        .with_keyword("Whistle") // biaise la reconnaissance vers ce mot
        .with_word_timestamps(true);
    let t = whistle.transcribe_with_options(&pcm, &options)?;

    println!("texte  : {}", t.text());
    println!("langue : {} | ttft {:.1} ms | {:.0} tok/s", t.language_code(), t.ttft_ms(), t.decode_tps());
    for w in t.words() {
        println!("{:>5.2}s {:>5.2}s  p={:.3}  {}", w.start(), w.end(), w.probability(), w.word());
    }
    Ok(())
}

/// WAV mono 16 kHz 16 bits → f32 dans [-1, 1].
/// Conversion : afconvert -f WAVE -d LEI16@16000 -c 1 in.m4a out.wav
fn read_wav(path: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    let mut reader = hound::WavReader::open(path)?;
    let s = reader.spec();
    if s.sample_rate != SAMPLE_RATE || s.channels != 1 || s.bits_per_sample != 16 {
        return Err(format!("{path} : {} Hz, {} canal(aux), {} bits ; attendu 16 kHz mono 16 bits", s.sample_rate, s.channels, s.bits_per_sample).into());
    }
    Ok(reader.samples::<i16>().map(|x| x.map(|v| f32::from(v) / 32768.0)).collect::<Result<_, _>>()?)
}
