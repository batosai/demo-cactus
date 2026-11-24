use cactus_rs::needle::{Call, Needle, Tool, Weights};
use serde_json::{json, Value};

fn tools() -> Vec<Tool> {
    vec![
        Tool::new(
            "set_thermostat",
            "Set the thermostat.",
            json!({
                "type": "object",
                "properties": {
                    "temperature": { "type": "integer", "description": "target temperature in Celsius" },
                    "mode": { "type": "string", "enum": ["heat", "cool", "auto"], "description": "heating strategy to use" }
                },
                "required": ["temperature"]
            }),
        ),
        Tool::new(
            "set_lights",
            "Turn a room's lights on or off and set brightness.",
            json!({
                "type": "object",
                "properties": {
                    "room": { "type": "string", "description": "which room to control" },
                    "on": { "type": "boolean", "description": "whether the lights are on" },
                    "brightness": { "type": "integer", "minimum": 0, "maximum": 100, "description": "brightness from 0 to 100" }
                },
                "required": ["room", "on"]
            }),
        ),
    ]
}

/// Implémentation « réelle » des outils.
fn dispatch(call: &Call) -> Value {
    let a = call.arguments();
    match call.name() {
        "set_thermostat" => json!({ "temperature": a["temperature"], "mode": a.get("mode").cloned().unwrap_or(json!("auto")) }),
        "set_lights" => json!({ "room": a["room"], "on": a["on"], "brightness": a.get("brightness").cloned().unwrap_or(json!(100)) }),
        other => json!({ "error": format!("unknown tool: {other}") }),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut needle = Needle::builder(Weights::fetch()?).tools(tools()).build()?;

    for q in [
        "make it 21 and cool the room",
        "turn off the kitchen lights",
        "dim the living room to 30 percent",
        "what's the capital of France?",
    ] {
        needle.reset(); // requêtes indépendantes
        let c = needle.complete(q)?;
        println!("{q}  (confiance {:.2})", c.confidence());

        // grounded_calls : seulement les appels dont les valeurs viennent bien de la requête.
        let results: Vec<Value> = c.grounded_calls().iter().map(|call| {
            println!("  appel     {} {}", call.name(), call.arguments_json());
            dispatch(call)
        }).collect();
        for call in c.suppressed_calls() {
            println!("  bloqué    {} {} (confiance trop faible)", call.name(), call.arguments_json());
        }
        if !c.validation().is_grounded() {
            println!("  inventé   {:?} (valeurs absentes de la requête, non exécuté)", c.validation().ungrounded());
        }

        // On renvoie les résultats au modèle pour qu'il formule sa réponse.
        if !results.is_empty() {
            let r = needle.complete(Value::Array(results).to_string())?;
            println!("  réponse   {}", r.reasoning().unwrap_or_default());
        }
        println!();
    }
    Ok(())
}
