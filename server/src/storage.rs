use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};  // Seek va SeekFrom olib tashlandi
use rand::Rng;
use serde::{Serialize, Deserialize};
use std::path::Path;

const DATA_FOLDER: &str = "data";
const DICTIONARY_FILE: &str = "data/dictionary.index";

#[derive(Clone, Serialize, Deserialize)]
pub struct Storage {
    pub tables: HashMap<String, Vec<String>>, // table_name -> columns
    pub dictionary_index: HashMap<String, String>,
}

impl Storage {
    pub fn new() -> Self {
        fs::create_dir_all(DATA_FOLDER).unwrap();
        let dictionary_index = if Path::new(DICTIONARY_FILE).exists() {
            let mut f = File::open(DICTIONARY_FILE).unwrap();
            let mut buf = vec![];
            f.read_to_end(&mut buf).unwrap();
            bincode::deserialize(&buf).unwrap_or_default()
        } else {
            HashMap::new()
        };
        Storage {
            tables: HashMap::new(),
            dictionary_index,
        }
    }

    fn save_index(&self) {
        let data = bincode::serialize(&self.dictionary_index).unwrap();
        let mut f = File::create(DICTIONARY_FILE).unwrap();
        f.write_all(&data).unwrap();
    }

    fn generate_id(&self) -> String {
        let chars: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789".chars().collect();
        loop {
            let id: String = (0..5).map(|_| {
                chars[rand::thread_rng().gen_range(0..chars.len())]
            }).collect();
            if !self.dictionary_index.contains_key(&id) {
                return id;
            }
        }
    }

    pub fn create_table(&mut self, table_name: &str, columns: Vec<String>) -> (bool, String) {
        let table_file = format!("{}/{}.bin", DATA_FOLDER, table_name);
        let cols_file = format!("{}/{}.cols", DATA_FOLDER, table_name);

        if Path::new(&table_file).exists() {
            if Path::new(&cols_file).exists() {
                let cols = fs::read_to_string(&cols_file).unwrap();
                self.tables.insert(table_name.to_string(), cols.split(',').map(|s| s.to_string()).collect());
            } else {
                self.tables.insert(table_name.to_string(), columns.clone());
            }
            return (true, format!("Table {} already exists.", table_name));
        }

        File::create(&table_file).unwrap();
        fs::write(&cols_file, columns.join(",")).unwrap();
        self.tables.insert(table_name.to_string(), columns);
        (true, format!("Table {} created.", table_name))
    }

    pub fn insert(&mut self, table_name: &str, values: Vec<String>) -> (bool, Option<String>) {
        let columns = match self.tables.get(table_name) {
            Some(cols) => cols,
            None => return (false, None),
        };
        if columns.len() != values.len() { return (false, None); }

        let table_file = format!("{}/{}.bin", DATA_FOLDER, table_name);
        let mut ids_to_write = vec![];

        for val in values {
            let id = self.dictionary_index.iter().find(|(_, v)| v == &&val).map(|(k, _)| k.clone()).unwrap_or_else(|| {
                let new_id = self.generate_id();
                self.dictionary_index.insert(new_id.clone(), val);
                new_id
            });
            ids_to_write.push(id);
        }

        let mut f = OpenOptions::new().append(true).open(&table_file).unwrap();
        for id in &ids_to_write {
            f.write_all(id.as_bytes()).unwrap();
        }

        self.save_index();
        (true, Some(ids_to_write[0].clone()))
    }

    pub fn select(&self, table_name: &str) -> (bool, Vec<HashMap<String, String>>) {
        let columns = match self.tables.get(table_name) {
            Some(cols) => cols,
            None => return (false, vec![]),
        };
        let table_file = format!("{}/{}.bin", DATA_FOLDER, table_name);
        if !Path::new(&table_file).exists() { return (false, vec![]); }

        let mut f = File::open(&table_file).unwrap();
        let mut rows = vec![];
        let mut buf = vec![0u8; columns.len() * 5];
        while f.read_exact(&mut buf).is_ok() {
            let mut row = HashMap::new();
            for (i, col) in columns.iter().enumerate() {
                let id = std::str::from_utf8(&buf[i*5..i*5+5]).unwrap();
                row.insert(col.clone(), self.dictionary_index.get(id).cloned().unwrap_or_default());
            }
            row.insert("id".to_string(), std::str::from_utf8(&buf[0..5]).unwrap().to_string());
            rows.push(row);
        }
        (true, rows)
    }
}
