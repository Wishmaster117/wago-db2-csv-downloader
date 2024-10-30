use anyhow::Result;
use dialoguer::MultiSelect;

pub fn handle_locale_selection(available_locales: &[&str]) -> Result<Vec<String>> {
    println!("\n🌍 Select the locales (space to select/cancel, Enter to confirm):");
    let chosen = MultiSelect::new()
        .items(available_locales)
        .defaults(&[true, true])
        .interact()?;
    
    if chosen.is_empty() {
        println!("At least one language must be selected!");
        return Ok(vec![]);
    }

    Ok(chosen.iter()
        .map(|&i| available_locales[i].to_string())
        .collect())
}