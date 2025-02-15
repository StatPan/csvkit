use csvkit::{
    reader::{DictReader, QuoteStyle, ReaderOptions},
    writer::{DictWriter, WriterOptions},
};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Cursor, Read, Write};
use std::str;

fn main() -> Result<(), Box<dyn Error>> {
    let file_name = "random.csv";
    let file = File::create(file_name)?;
    let fieldnames = vec![
        "header1".to_string(),
        "header2".to_string(),
        "header3".to_string(),
    ];
    let options = WriterOptions::default();
    let mut writer = DictWriter::new(file, fieldnames.clone(), options);

    writer.writeheader()?;

    // writerow를 사용하여 데이터 쓰기
    let mut row1 = HashMap::new();
    row1.insert("header1".to_string(), "value1".to_string());
    row1.insert("header2".to_string(), "value2".to_string());
    row1.insert("header3".to_string(), "value3".to_string());
    writer.writerow(row1)?;

    // writerows를 사용하여 여러 행 쓰기
    let mut rows: Vec<HashMap<String, String>> = Vec::new();
    for i in 0..100 {
        let mut row = HashMap::new();
        row.insert("header1".to_string(), format!("value1_{}", i));
        row.insert("header2".to_string(), format!("value2_{}", i));
        row.insert("header3".to_string(), format!("value3_{}", i));
        rows.push(row);
    }
    writer.writerows(rows)?;

    Ok(())
}
