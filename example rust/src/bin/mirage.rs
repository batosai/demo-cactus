use cactus_rs::mirage::{Form, Image, Mirage, Weights};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).unwrap_or_else(|| "../test.jpg".into());
    let image = Image::open(&path)?;
    let (w, h) = (image.width(), image.height());

    // Poids téléchargés une fois dans ~/.cache/cactus-rs (ou CACTUS_MIRAGE_WEIGHTS).
    let mut mirage = Mirage::builder(Weights::fetch()?).build()?;

    println!("== describe ==");
    let d = mirage.describe(&image)?;
    for (key, value) in d.scene() {
        println!("scène   {key}: {}", value.as_str().unwrap_or_default());
    }
    for o in d.objects() {
        // Boîtes en échelle 0–1000, ramenées en pixels.
        println!("objet   #{} {:<10} {:?}", o.id(), o.name(), o.bbox().map(|b| b.to_pixels(w, h)).unwrap_or_default());
    }
    for line in d.text() {
        println!("texte   {:?}", line.string());
    }

    println!("== extract ==");
    let labels = ["dog", "cat", "bird", "fish"].map(String::from);
    let answers = mirage.extract(&image, &Form::new().choice("animal", &labels))?;
    let a = answers.get("animal").ok_or("pas de réponse")?;
    println!("animal : {:?} (p = {:.2})", a.as_choice(), a.probability().unwrap_or_default());
    for (label, p) in a.probabilities() {
        println!("  {p:.3}  {label}");
    }
    Ok(())
}
