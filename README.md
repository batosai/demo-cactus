# Démo Cactus : Needle, Whistle et Mirage

Tests en local des trois petits modèles de [Cactus Compute](https://cactuscompute.com/), en **Python**, **Rust** et **Node.js**.

| Modèle | Rôle | Taille |
|---|---|---|
| **Needle 3** | Appel d'outils (function calling) | ~35 Mo |
| **Whistle** | Transcription audio (en, de, fr, es, it, nl, pl) | 16,9 Mo |
| **Mirage** | Vision : objets, texte, choix contraints | 22 Mo |

Tout tourne en local sur le CPU, sans clé d'API. Moteur et poids se téléchargent au premier lancement.

## Sommaire

- [Structure](#structure)
- [Python](#python)
- [Rust](#rust)
- [Node.js](#nodejs)
- [Résultats](#résultats)
- [Liens](#liens)

## Structure

```
.
├── example python/   # SDK officiel cactus-compute
├── example rust/     # crate cactus-rs
├── example node/     # module natif napi-rs au-dessus de cactus-rs
├── test.jpg          # image de test (perroquet ara)
└── test.wav          # phrase en français, WAV mono 16 kHz
```

## Python

SDK officiel [`cactus-compute`](https://huggingface.co/Cactus-Compute/cactus). Python 3.10 ou plus.

```bash
uv venv --python 3.12 .venv
uv pip install --python .venv/bin/python -U cactus-compute

export CACTUS_TELEMETRY=0   # désactive les statistiques d'usage anonymes
.venv/bin/python "example python/test_mirage.py" [image]
.venv/bin/python "example python/test_whistle.py" [fichier.wav]
.venv/bin/python "example python/test_needle.py"
```

## Rust

Basé sur le crate [`cactus-rs`](https://github.com/hxphsts/cactus-rs) 0.3, **non officiel**, qui couvre les trois modèles. Le moteur est téléchargé à la compilation, les poids au premier lancement dans `~/.cache/cactus-rs`.

```bash
cd "example rust"
cargo build --release
./target/release/mirage [image]
./target/release/whistle [fichier.wav]
./target/release/needle
```

## Node.js

Module natif [napi-rs](https://napi.rs) au-dessus de `cactus-rs`. Le moteur est intégré au fichier `.node` : pas de Python.

```bash
cd "example node"
npm install
npm run build   # compile cactus-node.<plateforme>.node, index.js et index.d.ts
npm test        # ou : node test.mjs [image] [fichier.wav]
```

### API

```ts
import { Mirage, Whistle, Needle } from './index.js';

const mirage = await Mirage.load();
await mirage.describe('photo.jpg');                                // scène, objets, texte
await mirage.choose('photo.jpg', 'animal', ['dog', 'cat', 'bird']); // { value, probability, probabilities }

const whistle = await Whistle.load();
await whistle.transcribe('clip.wav', { language: 'fr', keywords: ['Whistle'], wordTimestamps: true });
await whistle.transcribeSamples(float32Array);                     // micro ou flux audio

const needle = await Needle.load(tools, system);
const r = await needle.complete('turn off the kitchen lights');    // r.groundedCalls à exécuter côté JS
await needle.complete(JSON.stringify(results));                    // renvoie les résultats au modèle
needle.reset();                                                    // entre deux requêtes indépendantes
```

- Chaque inférence tourne sur un thread de travail : la boucle d'événements reste libre.
- Un seul modèle de chaque type par process : les appels concurrents sont traités les uns après les autres.
- Seule la cible macOS Apple Silicon a été compilée. `linux-x64-gnu` est déclarée mais nécessite une CI.

## Résultats

Mesurés sur Mac Apple Silicon, avec les fichiers de test du dépôt.

### Mirage

- `choose` / `extract` : **« bird »** avec une probabilité de 0,53.
- `describe` : décrit bien la scène (« blurred green », « natural », « macro photography ») et trouve l'œil, mais découpe le perroquet en « flower » et « petal ».
- Pas de question en texte libre : on demande des sections (`objects`, `text`) ou un formulaire à remplir.
- Boîtes englobantes sur une échelle de 0 à 1000.

### Whistle

- Transcription exacte : « Bonjour, je teste la reconnaissance vocale de Whistle sur mon Mac. »
- Sans le mot-clé `Whistle`, le nom propre devient « Oystle ».
- Horodatage par mot. Premier token en 6 à 13 ms, environ 700 à 1 200 tokens/s.

### Needle

| Requête | Résultat |
|---|---|
| « make it 21 and cool the room » | ✅ thermostat à 21 °C, mode `cool` |
| « turn off the kitchen lights » | ✅ lumières de la cuisine éteintes |
| « dim the living room to 30 percent » | ⚠️ intensité à 30, mais aussi `on: false` |
| « what's the capital of France? » | ✅ aucun appel (confiance 0,92) |

- Les appels dont les valeurs ne viennent pas de la requête (`ungrounded`) ne sont pas exécutés.
- Appeler `reset()` entre deux requêtes indépendantes : sinon l'historique fausse les réponses.
- Environ 180 Mo de RAM, 1 400 à 1 500 tokens/s.

## Liens

- [Cactus Compute](https://cactuscompute.com/)
- [Documentation v2.2.2](https://docs.cactuscompute.com/v2.2.2/)
- Modèles : [Needle 3](https://huggingface.co/Cactus-Compute/needle3), [Whistle](https://huggingface.co/Cactus-Compute/whistle), [Mirage](https://huggingface.co/Cactus-Compute/mirage)
- [cactus-rs](https://github.com/hxphsts/cactus-rs) (crate Rust non officiel)
