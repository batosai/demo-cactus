//! Module Node (napi-rs) au-dessus de cactus-rs : Mirage, Whistle et Needle.
//!
//! Chaque inférence tourne sur un thread bloquant (spawn_blocking) pour ne pas figer la boucle
//! d'événements de Node. Le moteur n'accepte qu'un modèle de chaque type par process : chaque
//! instance est protégée par un Mutex, les appels concurrents sont donc sérialisés.

use std::sync::{Arc, Mutex};

use cactus_rs::{mirage, needle, whistle};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde_json::Value;

fn err<E: std::fmt::Display>(e: E) -> Error {
    Error::from_reason(e.to_string())
}

/// Exécute `f` hors du thread JS.
async fn blocking<T, F>(f: F) -> Result<T>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T> + Send + 'static,
{
    tokio::task::spawn_blocking(f).await.map_err(err)?
}

/// Le moteur est global au process et sérialisé par nos Mutex : on peut déplacer les modèles entre threads.
struct Shared<T>(Arc<Mutex<T>>);
unsafe impl<T> Send for Shared<T> {}
unsafe impl<T> Sync for Shared<T> {}
impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Shared(self.0.clone())
    }
}
impl<T> Shared<T> {
    fn new(v: T) -> Self {
        Shared(Arc::new(Mutex::new(v)))
    }
    fn with<R>(&self, f: impl FnOnce(&mut T) -> Result<R>) -> Result<R> {
        let mut guard = self.0.lock().map_err(err)?;
        f(&mut guard)
    }
}

// ---------------------------------------------------------------- Mirage

#[napi]
pub struct Mirage {
    inner: Shared<mirage::Mirage>,
}

#[napi]
impl Mirage {
    /// Charge Mirage (poids téléchargés une fois dans ~/.cache/cactus-rs, ou CACTUS_MIRAGE_WEIGHTS).
    #[napi]
    pub async fn load() -> Result<Mirage> {
        let inner = blocking(|| {
            let m = mirage::Mirage::builder(mirage::Weights::fetch().map_err(err)?).build().map_err(err)?;
            Ok(Shared::new(m))
        })
        .await?;
        Ok(Mirage { inner })
    }

    /// Décrit une image (PNG/JPEG) : scène, objets (bbox 0–1000), texte, relations.
    #[napi(ts_return_type = "Promise<{ scene?: Record<string, string>; objects?: Array<{ id: number; name: string; bbox?: number[]; [k: string]: unknown }>; text?: Array<{ string: string; [k: string]: unknown }>; relations?: unknown[]; [k: string]: unknown }>")]
    pub async fn describe(&self, path: String) -> Result<Value> {
        let inner = self.inner.clone();
        blocking(move || {
            let image = mirage::Image::open(&path).map_err(err)?;
            let d = inner.with(|m| m.describe(&image).map_err(err))?;
            serde_json::to_value(d).map_err(err)
        })
        .await
    }

    /// Choisit une étiquette parmi `labels` pour la question `name` (ex. "animal").
    #[napi]
    pub async fn choose(&self, path: String, name: String, labels: Vec<String>) -> Result<Choice> {
        let inner = self.inner.clone();
        blocking(move || {
            let image = mirage::Image::open(&path).map_err(err)?;
            let form = mirage::Form::new().choice(&name, &labels);
            let answers = inner.with(|m| m.extract(&image, &form).map_err(err))?;
            let a = answers.get(&name).ok_or_else(|| err("pas de réponse"))?;
            Ok(Choice {
                value: a.as_choice().map(String::from),
                probability: a.probability(),
                probabilities: a.probabilities().iter().map(|(l, p)| LabelProbability { label: l.clone(), probability: *p }).collect(),
            })
        })
        .await
    }
}

#[napi(object)]
pub struct LabelProbability {
    pub label: String,
    pub probability: f64,
}

#[napi(object)]
pub struct Choice {
    /// Étiquette retenue, ou null si aucune ne convient.
    pub value: Option<String>,
    pub probability: Option<f64>,
    pub probabilities: Vec<LabelProbability>,
}

// ---------------------------------------------------------------- Whistle

#[napi(object)]
pub struct Word {
    pub word: String,
    pub start: f64,
    pub end: f64,
    pub probability: f64,
}

#[napi(object)]
pub struct Transcript {
    pub text: String,
    pub language: String,
    pub ttft_ms: f64,
    pub decode_tps: f64,
    pub words: Vec<Word>,
}

#[napi(object)]
pub struct TranscribeOptions {
    /// en, de, fr, es, it, nl, pl. Absent : détection automatique.
    pub language: Option<String>,
    /// Mots ou expressions à favoriser.
    pub keywords: Option<Vec<String>>,
    pub word_timestamps: Option<bool>,
}

#[napi]
pub struct Whistle {
    inner: Shared<whistle::Whistle>,
}

#[napi]
impl Whistle {
    #[napi]
    pub async fn load() -> Result<Whistle> {
        let inner = blocking(|| {
            let w = whistle::Whistle::builder(whistle::Weights::fetch().map_err(err)?).build().map_err(err)?;
            Ok(Shared::new(w))
        })
        .await?;
        Ok(Whistle { inner })
    }

    /// Transcrit un WAV mono 16 kHz 16 bits.
    #[napi]
    pub async fn transcribe(&self, path: String, options: Option<TranscribeOptions>) -> Result<Transcript> {
        let pcm = read_wav(&path)?;
        self.transcribe_samples(Float32Array::new(pcm), options).await
    }

    /// Transcrit des échantillons mono 16 kHz dans [-1, 1].
    #[napi]
    pub async fn transcribe_samples(&self, samples: Float32Array, options: Option<TranscribeOptions>) -> Result<Transcript> {
        let inner = self.inner.clone();
        let pcm = samples.to_vec();
        blocking(move || {
            let mut opts = whistle::TranscribeOptions::new();
            if let Some(o) = options {
                if let Some(lang) = o.language {
                    opts = opts.with_language(lang.parse::<whistle::Language>().map_err(err)?);
                }
                for k in o.keywords.unwrap_or_default() {
                    opts = opts.with_keyword(k);
                }
                opts = opts.with_word_timestamps(o.word_timestamps.unwrap_or(false));
            }
            let t = inner.with(|w| w.transcribe_with_options(&pcm, &opts).map_err(err))?;
            Ok(Transcript {
                text: t.text().to_string(),
                language: t.language_code().to_string(),
                ttft_ms: t.ttft_ms() as f64,
                decode_tps: t.decode_tps() as f64,
                words: t.words().iter().map(|w| Word {
                    word: w.word().to_string(),
                    start: w.start() as f64,
                    end: w.end() as f64,
                    probability: w.probability() as f64,
                }).collect(),
            })
        })
        .await
    }
}

fn read_wav(path: &str) -> Result<Vec<f32>> {
    let mut reader = hound::WavReader::open(path).map_err(err)?;
    let s = reader.spec();
    if s.sample_rate != whistle::SAMPLE_RATE || s.channels != 1 || s.bits_per_sample != 16 {
        return Err(err(format!(
            "{path} : {} Hz, {} canal(aux), {} bits ; attendu 16 kHz mono 16 bits",
            s.sample_rate, s.channels, s.bits_per_sample
        )));
    }
    reader
        .samples::<i16>()
        .map(|x| x.map(|v| f32::from(v) / 32768.0))
        .collect::<std::result::Result<_, _>>()
        .map_err(err)
}

// ---------------------------------------------------------------- Needle

#[napi(object)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    /// JSON Schema des paramètres.
    pub parameters: Value,
}

#[napi(object)]
pub struct Call {
    pub name: String,
    pub arguments: Value,
}

#[napi(object)]
pub struct Completion {
    /// "call" (appels demandés) ou "respond" (réponse finale).
    #[napi(js_name = "type")]
    pub kind: String,
    pub calls: Vec<Call>,
    /// Appels dont toutes les valeurs proviennent de la requête : ceux à exécuter.
    pub grounded_calls: Vec<Call>,
    /// Appels bloqués par le moteur (confiance trop faible).
    pub suppressed_calls: Vec<Call>,
    /// Chemins des valeurs inventées par le modèle.
    pub ungrounded: Vec<String>,
    pub confidence: f64,
    pub reasoning: Option<String>,
}

fn calls(list: &[needle::Call]) -> Vec<Call> {
    list.iter().map(|c| Call { name: c.name().to_string(), arguments: c.arguments().clone() }).collect()
}

#[napi]
pub struct Needle {
    inner: Shared<needle::Needle>,
}

#[napi]
impl Needle {
    /// Charge Needle 3 avec les outils disponibles (et un prompt système optionnel).
    #[napi]
    pub async fn load(tools: Vec<ToolDef>, system: Option<String>) -> Result<Needle> {
        let inner = blocking(move || {
            let mut b = needle::Needle::builder(needle::Weights::fetch().map_err(err)?)
                .tools(tools.into_iter().map(|t| needle::Tool::new(t.name, t.description, t.parameters)));
            if let Some(s) = system {
                b = b.system(s);
            }
            Ok(Shared::new(b.build().map_err(err)?))
        })
        .await?;
        Ok(Needle { inner })
    }

    /// Un tour de conversation : requête utilisateur, ou résultats d'outils sérialisés en JSON.
    #[napi]
    pub async fn complete(&self, input: String) -> Result<Completion> {
        let inner = self.inner.clone();
        blocking(move || {
            let c = inner.with(|n| n.complete(input).map_err(err))?;
            Ok(Completion {
                kind: serde_json::to_value(c.kind()).map_err(err)?.as_str().unwrap_or_default().to_string(),
                calls: calls(c.calls()),
                grounded_calls: calls(c.grounded_calls()),
                suppressed_calls: calls(c.suppressed_calls()),
                ungrounded: c.validation().ungrounded().to_vec(),
                confidence: c.confidence() as f64,
                reasoning: c.reasoning().map(String::from),
            })
        })
        .await
    }

    /// Vide l'historique (à appeler entre deux requêtes indépendantes).
    #[napi]
    pub fn reset(&self) -> Result<()> {
        self.inner.with(|n| {
            n.reset();
            Ok(())
        })
    }
}
