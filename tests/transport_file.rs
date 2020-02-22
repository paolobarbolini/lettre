#[cfg(test)]
#[cfg(feature = "file-transport")]
mod test {
    use lettre::file::FileTransport;
    use lettre::{Email, EmailAddress, Envelope, Transport};
    use std::env::temp_dir;
    use std::fs::{self, remove_file};

    #[test]
    fn file_transport() {
        let mut sender = FileTransport::new(temp_dir());
        let email = Email::new(
            Envelope::new(
                Some(EmailAddress::new("user@localhost".to_string()).unwrap()),
                vec![EmailAddress::new("root@localhost".to_string()).unwrap()],
            )
            .unwrap(),
            "id".to_string(),
            "Hello ß☺ example".to_string().into_bytes(),
        );
        let message_id = email.message_id().to_string();

        let result = sender.send(email);
        assert!(result.is_ok());

        let file = format!("{}/{}.json", temp_dir().to_str().unwrap(), message_id);
        let contents = fs::read_to_string(&file).unwrap();
        remove_file(file).unwrap();

        assert_eq!(
            contents,
            "{\"envelope\":{\"forward_path\":[\"root@localhost\"],\"reverse_path\":\"user@localhost\"},\"message_id\":\"id\",\"message\":\"Hello ß☺ example\"}"
        );
    }
}
