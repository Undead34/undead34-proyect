use std::fs;

use anyhow::Result;
use clap::Args;
use inquire::{Confirm, Text};
use sugar_path::SugarPath;

use super::Command;

#[derive(Args, Debug)]
pub struct InitCommand {
    #[arg(short, long)]
    name: Option<String>,
    #[arg(short, long, default_value = ".")]
    output: String,
}

impl InitCommand {
    fn create_pentesting_workspace(
        &self,
        base_path: impl Into<std::path::PathBuf>,
        client_name: String,
    ) -> Result<()> {
        let base_path = base_path.into();

        fs::create_dir_all(&base_path)?;

        let passive_dir = base_path.join("1_Passive_Enumeration");
        let active_dir = base_path.join("2_Active_Enumeration");
        let vulnerability_dir = base_path.join("3_Vulnerability_Assessment");
        let exploitation_dir = base_path.join("4_Exploitation");
        let reporting_dir = base_path.join("5_Reporting");

        fs::create_dir_all(&passive_dir)?;
        fs::create_dir_all(&active_dir)?;
        fs::create_dir_all(&vulnerability_dir)?;
        fs::create_dir_all(&exploitation_dir)?;
        fs::create_dir_all(&reporting_dir)?;

        // 3. Crea archivos iniciales (ejemplo: README)
        let main_readme = base_path.join("README.md");
        let content_main_readme = format!(
            "# Proyecto de Pentesting: {}\n\n\
        Esta estructura ha sido generada automáticamente.\n\n\
        - **1_Passive_Enumeration**: Recolección de información pasiva.\n\
        - **2_Active_Enumeration**: Enumeración activa y escaneo.\n\
        - **3_Vulnerability_Assessment**: Análisis y hallazgo de vulnerabilidades.\n\
        - **4_Exploitation**: Explotación de vulnerabilidades.\n\
        - **5_Reporting**: Generación de reportes finales.\n",
            client_name
        );
        fs::write(main_readme, content_main_readme)?;

        // Creamos un README en cada subdirectorio (opcional)
        let readme_sub = "# Documenta aquí las actividades y resultados de este paso.\n";
        fs::write(passive_dir.join("README.md"), readme_sub)?;
        fs::write(active_dir.join("README.md"), readme_sub)?;
        fs::write(vulnerability_dir.join("README.md"), readme_sub)?;
        fs::write(exploitation_dir.join("README.md"), readme_sub)?;
        fs::write(reporting_dir.join("README.md"), readme_sub)?;

        Ok(())
    }
}

impl Command for InitCommand {
    fn execute(&self) -> Result<()> {
        let client_name = match &self.name {
            Some(n) => n.to_string(),
            None => Text::new("Ingresa el nombre del cliente/proyecto:").prompt()?,
        };

        let confirm_msg = format!(
            "¿Crear la estructura de pentesting para \"{}\" en \"{}\"?",
            client_name, self.output
        );
        let confirmed = Confirm::new(&confirm_msg).with_default(true).prompt()?;

        if !confirmed {
            println!("Operación cancelada por el usuario.");
            return Ok(());
        }

        let base_path = self.output.as_path().join(&client_name).normalize();

        self.create_pentesting_workspace(base_path, client_name)?;

        Ok(())
    }
}
