use tokio::io::AsyncWriteExt;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CounterResponse {
    count: i32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct IncrementRequest {
    amount: i32,
}

#[tokio::test]
async fn test_file_processing() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let file1_path = temp_dir.path().join("file1.txt");
    let file2_path = temp_dir.path().join("file2.txt");

    let mut file1 = tokio::fs::File::create(&file1_path).await?;
    let mut file2 = tokio::fs::File::create(&file2_path).await?;

    file1.write_all(b"line 1\nerror in line 2\nline 3").await?;
    file2.write_all(b"no errors here\njust normal text").await?;

    // Read and process files
    let content1 = tokio::fs::read_to_string(&file1_path).await?;
    let content2 = tokio::fs::read_to_string(&file2_path).await?;

    assert!(content1.contains("error"));
    assert!(!content2.contains("error"));

    Ok(())
}
