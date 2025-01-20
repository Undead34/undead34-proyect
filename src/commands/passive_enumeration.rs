use anyhow::{anyhow, Context, Result};
use clap::Args;
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
    process::{Command, Output},
};
use url::Url;

#[derive(Args, Debug)]
pub struct PassiveEnumerationCommand {
    #[arg(short, long)]
    target: String,
}

impl super::Command for PassiveEnumerationCommand {
    fn execute(&self) -> Result<()> {
        println!("Iniciando enumeración pasiva para: {}", self.target);

        // Crear carpeta para guardar resultados
        let output_dir = "./1_Passive_Enumeration";
        fs::create_dir_all(output_dir).with_context(|| {
            format!(
                "No se pudo crear el directorio de resultados: {}",
                output_dir
            )
        })?;

        // Normalizar la URL (asegurar que tenga protocolo)
        let url = self.normalize_url(&self.target)?;

        // Ejecutar herramientas y guardar resultados
        // self.download_robots_txt(&url, output_dir)?;
        // self.get_http_headers(&url, output_dir)?;
        // self.test_http_methods(&url, output_dir)?;
        // self.run_whatweb(&url, output_dir)?;
        self.run_nmap(&url, output_dir)?;
        // self.run_nikto(&url, output_dir)?;

        Ok(())
    }
}

impl PassiveEnumerationCommand {
    /// Añade el protocolo "http://" si no está presente en la URL.
    fn normalize_url(&self, input: &str) -> Result<Url> {
        let input_with_protocol = if !input.starts_with("http://") && !input.starts_with("https://")
        {
            format!("http://{}", input)
        } else {
            input.to_string()
        };

        Url::parse(&input_with_protocol).context("No se pudo parsear la URL")
    }

    fn get_ip_from_url(&self, url: &Url) -> Result<String> {
        println!("Resolviendo dirección IP para: {}", url);

        let host = url
            .host_str()
            .ok_or_else(|| anyhow!("No se encontró host en la URL"))?;
        let dig_args = ["+short", host];

        let dig_output = self.run_command("dig", &dig_args)?;
        if !dig_output.status.success() {
            return Err(anyhow!("Error resolviendo dirección IP para: {}", host));
        }

        let ip = String::from_utf8_lossy(&dig_output.stdout)
            .lines()
            .next()
            .ok_or_else(|| anyhow!("No se obtuvo dirección IP"))?
            .to_string();

        Ok(ip)
    }

    /// Ejecuta un comando en la terminal y devuelve su salida (stdout + stderr).
    fn run_command(&self, program: &str, args: &[&str]) -> Result<Output> {
        Command::new(program).args(args).output().with_context(|| {
            format!(
                "Error al ejecutar comando: {} con argumentos: {:?}",
                program, args
            )
        })
    }

    /// Crea un archivo y escribe en él el contenido de `data`.
    fn write_output_file<P: AsRef<Path>>(&self, path: P, data: &[u8]) -> Result<()> {
        let path = path.as_ref();
        let mut file = File::create(path)
            .with_context(|| format!("No se pudo crear el archivo: {:?}", path))?;
        file.write_all(data)
            .with_context(|| format!("No se pudo escribir en el archivo: {:?}", path))?;
        Ok(())
    }

    /// Ejecuta WhatWeb con opciones avanzadas y guarda los resultados.
    fn run_whatweb(&self, url: &Url, output_dir: &str) -> Result<()> {
        println!("Ejecutando WhatWeb con opciones avanzadas...");

        // Guardamos la salida "nativa" (verbose) de WhatWeb
        let whatweb_verbose_native_path = format!("{}/whatweb_output_native.txt", output_dir);

        // Argumentos para ejecutar whatweb dentro de WSL con Kali
        let whatweb_args = [
            "-d",
            "kali-linux",
            "whatweb",
            "--follow-redirect=same-site",
            "--max-redirects=10",
            "-a",
            "3",
            "--log-verbose",
            &whatweb_verbose_native_path,
            url.as_str(),
        ];

        // Ejecutamos y comprobamos salida
        let whatweb_output = self.run_command("wsl", &whatweb_args)?;

        // Guardamos la salida estándar en otro archivo
        let whatweb_path = format!("{}/whatweb_output.txt", output_dir);
        self.write_output_file(&whatweb_path, &whatweb_output.stdout)?;

        println!("WhatWeb completado y guardado en: {}", whatweb_path);
        println!("Log detallado guardado en: {}", whatweb_verbose_native_path);

        Ok(())
    }

    /// Descarga el archivo robots.txt (siguiendo redirecciones) y lo guarda.
    fn download_robots_txt(&self, url: &Url, output_dir: &str) -> Result<()> {
        println!("Descargando robots.txt...");

        // Construir la URL para robots.txt
        let mut robots_url = url.clone();
        robots_url.set_path("/robots.txt");

        // Ejecutar cURL con la opción de seguir redirecciones
        let curl_args = [
            "-L", // Seguir redirecciones
            "--max-time",
            "10", // Tiempo máximo de espera
            "--retry",
            "3",                   // Reintentos en caso de fallo
            "--retry-connrefused", // Reintentar si la conexión es rechazada
            robots_url.as_str(),
        ];

        let curl_output = self
            .run_command("curl", &curl_args)
            .context("Error descargando robots.txt con cURL")?;

        // Validar si la ejecución fue exitosa
        if !curl_output.status.success() {
            eprintln!(
                "Error descargando robots.txt para {}: {}",
                robots_url,
                String::from_utf8_lossy(&curl_output.stderr)
            );
            return Err(anyhow!("Error al descargar robots.txt"));
        }

        // Guardar robots.txt en un archivo
        let robots_path = format!("{}/robots.txt", output_dir);
        self.write_output_file(&robots_path, &curl_output.stdout)?;
        println!("robots.txt guardado en: {}", robots_path);

        Ok(())
    }

    /// Obtiene los headers HTTP y HTTPS. Si hay redirección (códigos 3xx), sigue redirecciones en una segunda petición.
    fn get_http_headers(&self, url: &Url, output_dir: &str) -> Result<()> {
        // Asegurarnos de que el directorio existe
        fs::create_dir_all(output_dir)?;

        println!("Obteniendo headers HTTP y HTTPS...");

        let schemes = ["http", "https"];
        for scheme in &schemes {
            // Ajustamos la URL con el esquema correspondiente
            let mut adjusted_url = url.clone();
            adjusted_url.set_scheme(scheme).unwrap();

            // --- PRIMERA PETICIÓN (sin seguir redirecciones) ---
            let curl_args = [
                "-I", // Solo encabezados
                "--max-time",
                "10", // Tiempo máximo de espera
                "--retry",
                "3", // Reintentos en caso de fallo
                "--retry-connrefused",
                adjusted_url.as_str(),
            ];

            let curl_headers_output = self
                .run_command("curl", &curl_args)
                .with_context(|| format!("Error ejecutando cURL para esquema {}", scheme))?;

            // Guardamos la salida de la primera petición
            let headers_path = format!("{}/{}_headers.txt", output_dir, scheme);
            self.write_output_file(&headers_path, &curl_headers_output.stdout)?;
            println!("Headers de {} guardados en: {}", scheme, headers_path);

            // Si el proceso de cURL no fue exitoso, pasamos al siguiente esquema
            if !curl_headers_output.status.success() {
                eprintln!(
                    "Error obteniendo headers para {}: {}",
                    adjusted_url,
                    String::from_utf8_lossy(&curl_headers_output.stderr)
                );
                continue;
            }

            // --- VERIFICAMOS CÓDIGO DE ESTADO ---
            // Obtenemos el código HTTP (ej. "200", "301", etc.) de la salida
            let stdout_str = String::from_utf8_lossy(&curl_headers_output.stdout);
            let status_code = stdout_str
                .lines()
                .find_map(|line| {
                    if line.starts_with("HTTP/") {
                        // Normalmente la segunda "palabra" es el código de estado
                        line.split_whitespace()
                            .nth(1)
                            .and_then(|code_str| code_str.parse::<u16>().ok())
                    } else {
                        None
                    }
                })
                .unwrap_or(0);

            // Si el código de estado está en el rango 3xx, hacemos una segunda petición (siguiendo redirecciones)
            if (300..400).contains(&status_code) {
                let redirect_args = [
                    "-I", // Solo encabezados
                    "-L", // Seguir redirecciones
                    "--max-time",
                    "10",
                    "--retry",
                    "3",
                    "--retry-connrefused",
                    adjusted_url.as_str(),
                ];

                let redirect_output =
                    self.run_command("curl", &redirect_args).with_context(|| {
                        format!("Error ejecutando cURL (redirect) para esquema {}", scheme)
                    })?;

                // Guardamos la salida de la segunda petición (redirecciones)
                let redirect_path = format!("{}/{}_headers_redirect.txt", output_dir, scheme);
                self.write_output_file(&redirect_path, &redirect_output.stdout)?;
                println!(
                    "Headers con redirecciones de {} guardados en: {}",
                    scheme, redirect_path
                );
            }
        }

        println!("Proceso de obtención de headers completado.");
        Ok(())
    }

    /// Ejecuta Nikto y guarda los resultados.
    fn run_nikto(&self, url: &Url, output_dir: &str) -> Result<()> {
        println!("Ejecutando Nikto...");

        // Construcción de los argumentos de Nikto según las especificaciones.
        let nikto_args = [
            "-h",
            url.as_str(),
            "-nointeractive",
            "-no404",
            "-Tuning",
            "x",
            "-Plugins",
            "all",
            "-C",
            "all",
            "-o",
            &format!("{}/nikto_native.txt", output_dir),
            "-Format",
            "txt",
            "-followredirect",
            "-ssl",
            "-timeout",
            "10",
        ];

        // Ejecuta el comando Nikto usando WSL o directamente según el entorno.
        let nikto_output = self.run_command("nikto", &nikto_args)?;
        let nikto_path = format!("{}/nikto.txt", output_dir);

        self.write_output_file(&nikto_path, &nikto_output.stdout)?;
        println!("Nikto completado y guardado en: {}", nikto_path);
        Ok(())
    }

    /// Ejecuta Nmap y guarda los resultados.
    fn run_nmap(&self, url: &Url, output_dir: &str) -> Result<()> {
        println!("Ejecutando Nmap...");

        // Extraer el host de la URL
        let host = url
            .host_str()
            .ok_or_else(|| anyhow!("No se pudo obtener el host de la URL"))?;

        let nmap_path = &format!("{}/nmap_native.txt", output_dir);
        let nmap_args = [
            "--script",
            "http-title,http-server-header",
            "-p",
            "80,443",
            host,
            "-oN",
            nmap_path,
        ];

        let nmap_output = self.run_command("nmap", &nmap_args)?;

        if !nmap_output.status.success() {
            eprintln!(
                "Error en Nmap: {}",
                String::from_utf8_lossy(&nmap_output.stderr)
            );
        }

        println!("Nmap completado y guardado en: {}", nmap_path);
        Ok(())
    }

    /// Prueba métodos HTTP y guarda los resultados en archivos separados.
    fn test_http_methods(&self, url: &Url, output_dir: &str) -> Result<()> {
        println!("Probando métodos HTTP...");
        let methods = [
            "OPTIONS", "GET", "HEAD", "POST", "PUT", "DELETE", "PATCH", "TRACE", "CONNECT",
        ];

        for method in &methods {
            let curl_args = ["-X", method, "-i", url.as_str()];
            let curl_output = self
                .run_command("curl", &curl_args)
                .with_context(|| format!("Error probando método HTTP {}", method))?;

            let method_path = format!("{}/{}_method.txt", output_dir, method.to_lowercase());
            self.write_output_file(&method_path, &curl_output.stdout)?;
            println!(
                "Resultado del método {} guardado en: {}",
                method, method_path
            );
        }
        Ok(())
    }
}
