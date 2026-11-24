import { Mirage, Whistle, Needle } from './index.js';

const image = process.argv[2] ?? '../test.jpg';
const wav = process.argv[3] ?? '../test.wav';

// ---------------------------------------------------------------- Mirage
console.log('######## mirage');
const mirage = await Mirage.load();
const d = await mirage.describe(image);
for (const [k, v] of Object.entries(d.scene ?? {})) console.log(`scène   ${k}: ${v}`);
for (const o of d.objects ?? []) console.log(`objet   #${o.id} ${o.name.padEnd(10)} [${o.bbox}]`);
for (const t of d.text ?? []) console.log(`texte   ${t.string}`);

const c = await mirage.choose(image, 'animal', ['dog', 'cat', 'bird', 'fish']);
console.log(`animal : ${c.value} (p = ${c.probability?.toFixed(2)})`);

// ---------------------------------------------------------------- Whistle
console.log('\n######## whistle');
const whistle = await Whistle.load();
const t = await whistle.transcribe(wav, { language: 'fr', keywords: ['Whistle'], wordTimestamps: true });
console.log(`texte  : ${t.text}`);
console.log(`langue : ${t.language} | ttft ${t.ttftMs.toFixed(1)} ms | ${t.decodeTps.toFixed(0)} tok/s`);
for (const w of t.words) console.log(`${w.start.toFixed(2).padStart(5)}s ${w.end.toFixed(2).padStart(5)}s  p=${w.probability.toFixed(3)}  ${w.word}`);

// ---------------------------------------------------------------- Needle
console.log('\n######## needle');
const tools = [
  {
    name: 'set_thermostat',
    description: 'Set the thermostat.',
    parameters: {
      type: 'object',
      properties: {
        temperature: { type: 'integer', description: 'target temperature in Celsius' },
        mode: { type: 'string', enum: ['heat', 'cool', 'auto'], description: 'heating strategy to use' },
      },
      required: ['temperature'],
    },
  },
  {
    name: 'set_lights',
    description: "Turn a room's lights on or off and set brightness.",
    parameters: {
      type: 'object',
      properties: {
        room: { type: 'string', description: 'which room to control' },
        on: { type: 'boolean', description: 'whether the lights are on' },
        brightness: { type: 'integer', minimum: 0, maximum: 100, description: 'brightness from 0 to 100' },
      },
      required: ['room', 'on'],
    },
  },
];

// Implémentation « réelle » des outils, côté JS.
const handlers = {
  set_thermostat: ({ temperature, mode = 'auto' }) => ({ temperature, mode }),
  set_lights: ({ room, on, brightness = 100 }) => ({ room, on, brightness }),
};

const needle = await Needle.load(tools);
for (const q of [
  'make it 21 and cool the room',
  'turn off the kitchen lights',
  'dim the living room to 30 percent',
  "what's the capital of France?",
]) {
  needle.reset(); // requêtes indépendantes
  const r = await needle.complete(q);
  console.log(`${q}  (confiance ${r.confidence.toFixed(2)})`);

  const results = r.groundedCalls.map((call) => {
    console.log(`  appel     ${call.name} ${JSON.stringify(call.arguments)}`);
    return handlers[call.name]?.(call.arguments) ?? { error: `unknown tool: ${call.name}` };
  });
  for (const call of r.suppressedCalls) console.log(`  bloqué    ${call.name} ${JSON.stringify(call.arguments)}`);
  if (r.ungrounded.length) console.log(`  inventé   ${r.ungrounded.join(', ')} (non exécuté)`);

  if (results.length) {
    const final = await needle.complete(JSON.stringify(results));
    console.log(`  réponse   ${final.reasoning ?? ''}`);
  }
  console.log();
}
