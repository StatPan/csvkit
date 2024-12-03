use std::{collections::HashMap, io::BufReader};
use std::{
    default,
    io::{BufWriter, Result, Write},
};
use std::{fs, io::BufRead};
use std::{fs::File, io::Read};

pub struct DictWriter {
    writer: BufWriter<File>,
    headers: Vec<String>,
    has_written_headers: bool,
    separator: String,
}

impl DictWriter {
    pub fn new(file_path: &str, headers: Vec<&str>) -> Result<Self> {
        Self::new_with_options(file_path, headers, 8192, ",")
    }
    pub fn new_with_options(
        file_path: &str,
        headers: Vec<&str>,
        bufsize: usize,
        separator: &str,
    ) -> Result<Self> {
        let block_size = get_block_size(file_path).unwrap_or(4096);
        let buffer_size = bufsize.next_multiple_of(block_size);

        let _sep = separator.to_string();
        let file = File::create(file_path)?;
        let writer = BufWriter::with_capacity(buffer_size, file);
        Ok(Self {
            writer,
            headers: headers.into_iter().map(String::from).collect(),
            has_written_headers: false,
            separator: _sep,
        })
    }

    pub fn write_row(&mut self, row: HashMap<&str, &str>) -> Result<()> {
        self.write_row_data(&row)?; // 행 데이터 쓰기
        self.writer.flush()?; // 단일 행 작성 후 flush
        Ok(())
    }

    pub fn write_rows(&mut self, rows: Vec<HashMap<&str, &str>>) -> Result<()> {
        for row in rows {
            self.write_row_data(&row)?; // 행 데이터 쓰기
        }
        self.writer.flush()?; // 모든 행 작성 후 flush
        Ok(())
    }

    pub fn write_headers(&mut self) -> Result<()> {
        if !self.has_written_headers {
            let header_line = self.headers.join(&self.separator) + "\n";
            self.writer.write_all(header_line.as_bytes())?;
            self.has_written_headers = true;
        }
        Ok(())
    }

    fn write_row_data(&mut self, row: &HashMap<&str, &str>) -> Result<()> {
        let row_data: Vec<String> = self
            .headers
            .iter()
            .map(|header| row.get(header.as_str()).unwrap_or(&"").to_string())
            .collect();

        let row_line = row_data.join(&self.separator.to_string()) + "\n";
        self.writer.write_all(row_line.as_bytes())
    }
}

fn get_block_size(file_path: &str) -> Option<usize> {
    use std::os::unix::fs::MetadataExt;
    fs::metadata(file_path)
        .ok()
        .map(|meta| meta.blksize() as usize)
}

trait NextMultipleOf {
    fn next_multiple_of(&self, multiple: usize) -> usize;
}

impl NextMultipleOf for usize {
    fn next_multiple_of(&self, multiple: usize) -> usize {
        if *self % multiple == 0 {
            *self
        } else {
            *self + multiple - (*self % multiple)
        }
    }
}

pub struct DictReader {
    reader: BufReader<File>,
    headers: Vec<String>,
    separator: String,
    content: Vec<HashMap<String, String>>,
}

impl DictReader {
    pub fn new(file_path: &str, separator: &str, has_headers: bool) -> Result<Self> {
        let file = File::open(file_path)?;
        let mut reader = BufReader::new(file);
        let mut content = Vec::new();

        let headers = if has_headers {
            let mut header_line = String::new();
            reader.read_line(&mut header_line)?; // Read the header line

            header_line
                .trim_end()
                .split(separator)
                .map(String::from)
                .collect()
        } else {
            Vec::new()
        };

        Ok(Self {
            reader,
            headers,
            separator: separator.to_string(),
            content,
        })
    }

    pub fn read_all(&mut self) -> Result<()> {
        // No longer returns data (it's stored in self.content)
        for line in self.reader.by_ref().lines() {
            let line = line?;
            let values: Vec<String> = line
                .trim_end()
                .split(&self.separator)
                .map(String::from)
                .collect();

            if !self.headers.is_empty() && values.len() != self.headers.len() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Incorrect number of fields",
                ));
            }

            let mut row = HashMap::new();
            if self.headers.is_empty() {
                for (i, value) in values.into_iter().enumerate() {
                    row.insert(i.to_string(), value);
                }
            } else {
                for (i, value) in values.into_iter().enumerate() {
                    row.insert(self.headers[i].clone(), value);
                }
            }
            self.content.push(row); // Add the row to self.content
        }
        Ok(())
    }

    pub fn get_content(&self) -> &Vec<HashMap<String, String>> {
        &self.content
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::fs;

    #[test]
    fn test_csv_write_and_read() {
        // 테스트용 파일 경로
        let test_file = "test_output.csv";

        // DictWriter 생성
        let headers = vec!["name", "age", "city"];
        let mut writer = DictWriter::new(test_file, headers.clone()).unwrap();

        // 데이터 쓰기
        writer
            .write_row(HashMap::from([
                ("name", "Alice"),
                ("age", "30"),
                ("city", "New York"),
            ]))
            .unwrap();
        writer
            .write_row(HashMap::from([
                ("name", "Bob"),
                ("age", "25"),
                ("city", "San Francisco"),
            ]))
            .unwrap();
        writer.flush().unwrap();

        // 파일 읽기 및 검증
        let content = fs::read_to_string(test_file).unwrap();
        let expected = "name,age,city\nAlice,30,New York\nBob,25,San Francisco\n";
        assert_eq!(content, expected);

        // 테스트 파일 삭제
        fs::remove_file(test_file).unwrap();
    }
}
