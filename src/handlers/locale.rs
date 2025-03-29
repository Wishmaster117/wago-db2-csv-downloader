use anyhow::Result;
use dialoguer::MultiSelect;

pub fn handle_locale_selection(available_locales: &[&str]) -> anyhow::Result<Vec<String>> {
    println!("\n🌍 Select the locales (space to select/cancel, Enter to confirm):");
    
    // Par exemple, si available_locales est dans l'ordre et contient frFR, esES et enUS à des positions connues
    // Vous pouvez créer un vecteur de booléens avec ces valeurs à true
    let defaults: Vec<bool> = available_locales.iter().map(|&loc| {
        loc == "frFR" || loc == "esES" || loc == "enUS"
    }).collect();
    
    let chosen = dialoguer::MultiSelect::new()
        .items(available_locales)
        .defaults(&defaults)
        .interact()?;
    
    if chosen.is_empty() {
        println!("At least one language must be selected!");
        return Ok(vec![]);
    }

    Ok(chosen.iter()
        .map(|&i| available_locales[i].to_string())
        .collect())
}
