pub fn format_memory(size: u64) -> String {
    if size >= 1024 * 1024 {
        format!("{:.2} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if size >= 1024 {
        format!("{:.2} MB", size as f64 / (1024.0 * 1024.0))
    } else {
        format!("{} KB", size)
    }
}

pub async fn write(file: &str, content: &[u8]) -> std::io::Result<()> {
    tokio::fs::write(format!("./upload/{}", file), content).await
}

pub async fn generate_latex_image(
    latex: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let url = "https://latex.codecogs.com/png.latex?";
    let latex_colored = format!(r#"\LARGE \color{{white}}{{{}}}"#, latex);
    let response = reqwest::get(format!("{}{}", url, latex_colored)).await?;
    let bytes = response.bytes().await?;
    Ok(bytes.to_vec())
}
