use anyhow::Result;
use dialoguer::MultiSelect;
use crate::entities::Build;

pub fn handle_build_selection(available_builds: &[&str]) -> Result<Vec<Build>> {
    println!("\n📦 Select builds (space to select/cancel, Enter to confirm):");
    let chosen = MultiSelect::new()
        .items(available_builds)
        .defaults(&[true])
        .interact()?;

    if chosen.is_empty() {
        println!("At least one build must be chosen!");
        return Ok(vec![]);
    }

    Ok(chosen.iter()
        .map(|&i| {
            let parts: Vec<&str> = available_builds[i].split('.').collect();
            let version = parts[..3].join(".");
            let build_number = parts[3].parse().unwrap_or(0);
            Build::new(&version, build_number)
        })
        .collect())
}