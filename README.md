# Cactus

https://cactuscompute.com/
https://docs.cactuscompute.com/v2.2.2/

# Example Python  (`example python/`)

cd ~/Sites/_labs/cactus
CACTUS_TELEMETRY=0 .venv/bin/python test_mirage.py [image]
CACTUS_TELEMETRY=0 .venv/bin/python test_whistle.py [fichier.wav]
CACTUS_TELEMETRY=0 .venv/bin/python test_needle.py


# Exemples Rust (`example rust/`)

Basés sur le crate [`cactus-rs`](https://github.com/hxphsts/cactus-rs) 0.3 (non officiel, couvre Needle 3, Whistle et Mirage).
Le moteur est téléchargé au build, les poids au premier lancement dans `~/.cache/cactus-rs` : aucune dépendance à Python.
Chemins surchargeables : `CACTUS_NEEDLE_WEIGHTS`, `CACTUS_WHISTLE_WEIGHTS`, `CACTUS_MIRAGE_WEIGHTS`.

cd "example rust"
cargo build --release
./target/release/mirage [image]
./target/release/whistle [fichier.wav]   # WAV mono 16 kHz 16 bits
./target/release/needle


# Module Node (`example node/`)

Module natif [napi-rs](https://napi.rs) au-dessus de `cactus-rs` : le moteur est intégré au `.node`, pas de Python.
Inférences asynchrones (threads de travail), un modèle de chaque type par process (appels sérialisés).

cd "example node"
npm install
npm run build          # compile cactus-node.<plateforme>.node + index.js/index.d.ts
npm test               # ou : node test.mjs [image] [fichier.wav]

API :
  const mirage = await Mirage.load();   await mirage.describe(path);   await mirage.choose(path, 'animal', ['dog', 'cat', 'bird']);
  const whistle = await Whistle.load(); await whistle.transcribe(wav, { language: 'fr', keywords: ['Whistle'], wordTimestamps: true });
  const needle = await Needle.load(tools, system?); await needle.complete(text); needle.reset();


# Résultats

Mirage (vision, 22 Mo), testé sur une photo de perroquet ara :
- extract donne la bonne réponse : « bird » avec une probabilité de 0,54.
- describe détecte bien l'œil, mais classe le reste comme « petal », « flower » et « center ». C'est une détection d'objets avec boîtes englobantes, pas une description libre de l'image. Les coordonnées semblent normalisées entre 0 et 1000.
- On ne peut pas lui poser de question en texte libre : on lui demande des sections (objects, text) ou on lui donne un schéma à remplir avec extract.

Whistle (transcription audio, 16,9 Mo), testé sur une phrase en français générée avec say :
- Transcription : « Bonjour, je teste la reconnaissance vocale de Oystle sur mon Mac. »
- Seul le nom propre est raté (probabilité 0,37) ; le paramètre keywords sert à corriger ce genre de cas.
- Les horodatages par mot fonctionnent. Premier token en 7,6 ms, puis environ 1 190 tokens/s.

Needle (appel d'outils), testé avec deux outils, thermostat et lumières :
- Les trois demandes utiles sont bien comprises et les outils sont exécutés. Par exemple, « make it 21 and cool » donne 21 °C en mode cool.
- Défaut : pour « dim the living room to 30 percent », il éteint la lumière (on: False) au lieu de baisser l'intensité à 30 %.
- Défaut : pour une question hors sujet (« capitale de la France »), il invente un appel au thermostat à 68 °C. La validation interne le marque ungrounded et ne l'exécute pas.
- Environ 180 Mo de RAM et 1 400 à 1 500 tokens/s.
