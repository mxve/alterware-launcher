mod config {
    use crate::{config, fs, structs};
    use serial_test::serial;
    use std::path::Path;

    fn setup_test_path() -> std::path::PathBuf {
        let path = Path::new("tests_tmp").join("config.json");
        if path.exists() {
            fs::remove_file(&path).unwrap();
        }
        path
    }

    #[test]
    #[serial]
    fn load_config() {
        let path = setup_test_path();

        let config = config::load(path.clone());
        assert_eq!(config, structs::Config::default());

        fs::remove_file(path).unwrap();
    }

    #[test]
    #[serial]
    fn save_and_load_config() {
        let path = setup_test_path();

        let config = structs::Config::default();
        config::save(path.clone(), config.clone());
        let loaded_config = config::load(path.clone());
        assert_eq!(loaded_config, config);

        fs::remove_file(path).unwrap();
    }

    #[test]
    #[serial]
    fn save_value() {
        let path = setup_test_path();

        config::save_value(path.clone(), "update_only", true);
        let loaded_config = config::load(path.clone());
        assert_eq!(loaded_config.update_only, true);

        fs::remove_file(path).unwrap();
    }
}

mod misc {
    use crate::{cache, fs, misc, structs, Blake3Path};
    use std::{fs::File, io::Write, path::Path};

    #[test]
    fn file_blake3() {
        let path = Path::new("tests_tmp").join("blake3");
        if path.exists() {
            fs::remove_file(&path).unwrap();
        }
        fs::create_dir_all(path.parent().unwrap()).unwrap();

        File::create(&path)
            .unwrap()
            .write_all(b"alterware")
            .unwrap();
        let blake3 = path.get_blake3().unwrap();
        assert_eq!(
            blake3,
            "f18a70588a620f3a874120dbc2a41f49a0f44349c8a9c10c51f2f1c7bb678daa"
        );

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn human_readable_bytes() {
        assert_eq!(misc::human_readable_bytes(0), "0.00B");
        assert_eq!(misc::human_readable_bytes(1023), "1023.00B");
        assert_eq!(misc::human_readable_bytes(1024), "1.00KB");
        assert_eq!(misc::human_readable_bytes(1099511627776), "1.00TB");
    }

    #[test]
    #[cfg(unix)]
    fn is_program_in_path() {
        assert!(misc::is_program_in_path("ls"));
        assert!(!misc::is_program_in_path("nonexistent"));
    }

    #[test]
    fn cache_operations() {
        let path = Path::new("tests_tmp");
        fs::create_dir_all(path).unwrap();
        let cache_file = path.join("awcache.json");

        // Test initial empty cache
        let initial_cache = cache::get_cache(path);
        assert_eq!(initial_cache, structs::Cache::default());

        // Test saving and loading cache
        let test_cache = structs::Cache {
            iw4x_revision: "r1234".to_string(),
            hashes: [("test".to_string(), "hash".to_string())]
                .into_iter()
                .collect(),
            stored_data: None,
        };
        cache::save_cache(path, test_cache.clone());
        let loaded_cache = cache::get_cache(path);
        assert_eq!(loaded_cache, test_cache);

        fs::remove_file(&cache_file).unwrap();
    }
}

mod stored_data {
    use crate::{cache, fs, structs::StoredGameData};
    use serial_test::serial;
    use std::{collections::HashMap, path::Path};

    #[test]
    #[serial]
    fn stored_game_data() {
        let test_path = "test/path";
        let mut test_clients = HashMap::new();
        test_clients.insert("iw4".to_string(), vec!["iw4x".to_string()]);

        let data = StoredGameData {
            game_path: test_path.to_string(),
            clients: test_clients,
        };

        let path = Path::new("tests_tmp");
        fs::create_dir_all(path).unwrap();

        cache::store_game_data(&data).unwrap();
        let loaded = cache::get_stored_data().unwrap();

        assert_eq!(data.game_path, loaded.game_path);
        assert_eq!(data.clients, loaded.clients);

        fs::remove_dir_all(path).unwrap();
    }
}
