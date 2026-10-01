pub fn extract_file_id(value: &str) -> Option<String> {
    let start = value.find("file_")?;
    let file_id = value[start..]
        .chars()
        .take_while(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
        .collect::<String>();
    (file_id.len() > "file_".len()).then_some(file_id)
}

#[cfg(test)]
mod tests {
    use super::extract_file_id;

    #[test]
    fn extracts_the_file_id_from_image_and_file_urls() {
        assert_eq!(
            extract_file_id(
                "https://api.vrchat.cloud/api/1/image/file_1234abcd-0000-1111-2222-abcdefabcdef/2/256"
            )
            .as_deref(),
            Some("file_1234abcd-0000-1111-2222-abcdefabcdef")
        );
        assert_eq!(
            extract_file_id("https://api.vrchat.cloud/api/1/file/file_abc/1/file").as_deref(),
            Some("file_abc")
        );
        assert_eq!(extract_file_id("file_abc"), Some("file_abc".into()));
    }

    #[test]
    fn rejects_urls_without_a_file_id() {
        assert_eq!(extract_file_id("https://images.example/avatar.png"), None);
        assert_eq!(extract_file_id("file_"), None);
        assert_eq!(extract_file_id(""), None);
    }
}
