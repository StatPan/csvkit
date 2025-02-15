use crate::reader::QuoteStyle;
use std::collections::HashMap;
use std::error::Error;
use std::io::{BufWriter, Write}; // Cursor 추가
use std::str;

#[derive(Debug, Clone)]
pub struct WriterOptions {
    pub delimiter: u8,
    pub doublequote: bool,
    pub escapechar: Option<u8>,
    pub quotechar: u8,
    pub quoting: QuoteStyle,
    pub skipinitialspace: bool,
    pub strict: bool,
    pub lineterminator: String,
}

impl Default for WriterOptions {
    fn default() -> Self {
        WriterOptions {
            delimiter: b',',
            doublequote: true,
            escapechar: None,
            quotechar: b'"',
            quoting: QuoteStyle::Minimal,
            skipinitialspace: false,
            strict: false,
            lineterminator: "\r\n".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct DictWriter<W: Write> {
    writer: BufWriter<W>,
    fieldnames: Vec<String>,
    options: WriterOptions,
}

impl<W: Write> DictWriter<W> {
    pub fn new(writer: W, fieldnames: Vec<String>, options: WriterOptions) -> Self {
        DictWriter {
            writer: BufWriter::new(writer),
            fieldnames,
            options,
        }
    }

    pub fn writeheader(&mut self) -> Result<usize, Box<dyn Error>> {
        let mut csv_row = String::new();
        for (i, fieldname) in self.fieldnames.iter().enumerate() {
            let quoted_value = self.quote_value(fieldname)?;
            csv_row.push_str(&quoted_value);
            if i < self.fieldnames.len() - 1 {
                csv_row.push(self.options.delimiter as char);
            }
        }
        csv_row.push_str(&self.options.lineterminator);
        let bytes_written = self.writer.write(csv_row.as_bytes())?;
        self.writer.flush()?;
        Ok(bytes_written)
    }

    pub fn writerow(&mut self, row: HashMap<String, String>) -> Result<usize, Box<dyn Error>> {
        let mut csv_row = String::new();
        for (i, fieldname) in self.fieldnames.iter().enumerate() {
            let empty_value = "".to_string();
            let value = row.get(fieldname).unwrap_or(&empty_value); // 값이 없으면 빈 문자열 사용
            let quoted_value = self.quote_value(value)?;
            csv_row.push_str(&quoted_value);
            if i < self.fieldnames.len() - 1 {
                csv_row.push(self.options.delimiter as char);
            }
        }
        csv_row.push_str(&self.options.lineterminator);
        let bytes_written = self.writer.write(csv_row.as_bytes())?;
        self.writer.flush()?;
        Ok(bytes_written)
    }

    fn quote_value(&self, value: &str) -> Result<String, Box<dyn Error>> {
        let needs_quotes = match self.options.quoting {
            QuoteStyle::All => true,
            QuoteStyle::Minimal => {
                value.contains(self.options.delimiter as char)
                    || value.contains(self.options.quotechar as char)
                    || value.contains('\n')
                    || value.contains('\r')
            }
            QuoteStyle::NonNumeric => !value.chars().all(|c| c.is_numeric()),
            QuoteStyle::None => false,
        };

        if needs_quotes {
            let mut quoted_value = String::new();
            quoted_value.push(self.options.quotechar as char);

            for c in value.chars() {
                if c == self.options.quotechar as char {
                    if self.options.doublequote {
                        quoted_value.push(self.options.quotechar as char);
                        quoted_value.push(self.options.quotechar as char);
                    } else if let Some(escapechar) = self.options.escapechar {
                        quoted_value.push(escapechar as char);
                        quoted_value.push(self.options.quotechar as char);
                    } else {
                        return Err(
                            "Need to escape the quote character but no escapechar is set".into(),
                        );
                    }
                } else {
                    quoted_value.push(c);
                }
            }

            quoted_value.push(self.options.quotechar as char);
            Ok(quoted_value)
        } else {
            Ok(value.to_string())
        }
    }

    pub fn writerows(
        &mut self,
        rows: Vec<HashMap<String, String>>,
    ) -> Result<usize, Box<dyn Error>> {
        let mut total_bytes_written = 0;
        for row in rows {
            total_bytes_written += self.writerow(row)?;
        }
        Ok(total_bytes_written)
    }

    pub fn flush(&mut self) -> Result<(), Box<dyn Error>> {
        self.writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_dict_writer_basic() -> Result<(), Box<dyn Error>> {
        let mut buffer = Cursor::new(Vec::new());
        let fieldnames = vec!["header1".to_string(), "header2".to_string()];
        let options = WriterOptions::default();
        {
            let mut writer = DictWriter::new(&mut buffer, fieldnames.clone(), options);
            writer.writeheader()?;

            let mut row1 = HashMap::new();
            row1.insert("header1".to_string(), "value1".to_string());
            row1.insert("header2".to_string(), "value2".to_string());
            writer.writerow(row1)?;

            let mut row2 = HashMap::new();
            row2.insert("header1".to_string(), "value3".to_string());
            row2.insert("header2".to_string(), "value4".to_string());
            writer.writerow(row2)?;
        }
        let contents = String::from_utf8(buffer.into_inner())?;
        assert_eq!(
            contents,
            "header1,header2\r\nvalue1,value2\r\nvalue3,value4\r\n"
        );
        Ok(())
    }

    #[test]
    fn test_dict_writer_custom_delimiter() -> Result<(), Box<dyn Error>> {
        let mut buffer = Cursor::new(Vec::new());
        let fieldnames = vec!["header1".to_string(), "header2".to_string()];
        let options = WriterOptions {
            delimiter: b';',
            ..Default::default()
        };
        {
            let mut writer = DictWriter::new(&mut buffer, fieldnames.clone(), options);
            writer.writeheader()?;

            let mut row1 = HashMap::new();
            row1.insert("header1".to_string(), "value1".to_string());
            row1.insert("header2".to_string(), "value2".to_string());
            writer.writerow(row1)?;
        } // writer drops here, releasing the mutable borrow on buffer
        let contents = String::from_utf8(buffer.into_inner())?;
        assert_eq!(contents, "header1;header2\r\nvalue1;value2\r\n");
        Ok(())
    }

    #[test]
    fn test_dict_writer_quote_all() -> Result<(), Box<dyn Error>> {
        let mut buffer = Cursor::new(Vec::new());
        let fieldnames = vec!["header1".to_string(), "header2".to_string()];
        let options = WriterOptions {
            quoting: QuoteStyle::All,
            ..Default::default()
        };
        {
            let mut writer = DictWriter::new(&mut buffer, fieldnames.clone(), options);
            writer.writeheader()?;

            let mut row1 = HashMap::new();
            row1.insert("header1".to_string(), "value1".to_string());
            row1.insert("header2".to_string(), "value2".to_string());
            writer.writerow(row1)?;
        }
        let contents = String::from_utf8(buffer.into_inner())?;
        assert_eq!(
            contents,
            "\"header1\",\"header2\"\r\n\"value1\",\"value2\"\r\n"
        );
        Ok(())
    }

    #[test]
    fn test_dict_writer_quote_minimal() -> Result<(), Box<dyn Error>> {
        let mut buffer = Cursor::new(Vec::new());
        let fieldnames = vec!["header1".to_string(), "header2".to_string()];
        let options = WriterOptions {
            quoting: QuoteStyle::Minimal,
            ..Default::default()
        };
        {
            let mut writer = DictWriter::new(&mut buffer, fieldnames.clone(), options);
            writer.writeheader()?;

            let mut row1 = HashMap::new();
            row1.insert("header1".to_string(), "value1,".to_string());
            row1.insert("header2".to_string(), "value2".to_string());
            writer.writerow(row1)?;
        }
        let contents = String::from_utf8(buffer.into_inner())?;
        assert_eq!(contents, "header1,header2\r\n\"value1,\",value2\r\n");
        Ok(())
    }

    #[test]
    fn test_dict_writer_escapechar() -> Result<(), Box<dyn Error>> {
        let mut buffer = Cursor::new(Vec::new());
        let fieldnames = vec!["header1".to_string(), "header2".to_string()];
        let mut options = WriterOptions {
            escapechar: Some(b'\\'),
            doublequote: false, // doublequote를 false로 설정
            quoting: QuoteStyle::All,
            ..Default::default()
        };
        {
            let mut writer = DictWriter::new(&mut buffer, fieldnames.clone(), options);
            writer.writeheader()?;

            let mut row1 = HashMap::new();
            row1.insert("header1".to_string(), "value1".to_string());
            row1.insert("header2".to_string(), "value\"2".to_string());
            writer.writerow(row1)?;
        }
        let contents = String::from_utf8(buffer.into_inner())?;
        assert_eq!(
            contents,
            "\"header1\",\"header2\"\r\n\"value1\",\"value\\\"2\"\r\n"
        );
        Ok(())
    }

    #[test]
    fn test_dict_writer_lineterminator() -> Result<(), Box<dyn Error>> {
        let mut buffer = Cursor::new(Vec::new());
        let fieldnames = vec!["header1".to_string(), "header2".to_string()];
        let mut options = WriterOptions::default();
        options.lineterminator = "\n".to_string();
        {
            let mut writer = DictWriter::new(&mut buffer, fieldnames.clone(), options);
            writer.writeheader()?;
            let mut row1 = HashMap::new();
            row1.insert("header1".to_string(), "value1".to_string());
            row1.insert("header2".to_string(), "value2".to_string());
            writer.writerow(row1)?;
        }
        let contents = String::from_utf8(buffer.into_inner())?;
        assert_eq!(contents, "header1,header2\nvalue1,value2\n");
        Ok(())
    }

    #[test]
    fn test_dict_writer_doublequote() -> Result<(), Box<dyn Error>> {
        let mut buffer = Cursor::new(Vec::new());
        let fieldnames = vec!["header1".to_string(), "header2".to_string()];
        let mut options = WriterOptions::default();
        options.doublequote = true;
        options.quoting = QuoteStyle::All;
        {
            let mut writer = DictWriter::new(&mut buffer, fieldnames.clone(), options);
            writer.writeheader()?;
            let mut row1 = HashMap::new();
            row1.insert("header1".to_string(), "value1".to_string());
            row1.insert("header2".to_string(), "value\"2".to_string());
            writer.writerow(row1)?;
        }
        let contents = String::from_utf8(buffer.into_inner())?;
        assert_eq!(
            contents,
            "\"header1\",\"header2\"\r\n\"value1\",\"value\"\"2\"\r\n"
        );
        Ok(())
    }

    #[test]
    fn test_dict_writer_no_quote() -> Result<(), Box<dyn Error>> {
        let mut buffer = Cursor::new(Vec::new());
        let fieldnames = vec!["header1".to_string(), "header2".to_string()];
        let mut options = WriterOptions::default();
        options.quoting = QuoteStyle::None;
        {
            let mut writer = DictWriter::new(&mut buffer, fieldnames.clone(), options);
            writer.writeheader()?;
            let mut row1 = HashMap::new();
            row1.insert("header1".to_string(), "value1".to_string());
            row1.insert("header2".to_string(), "value2".to_string());
            writer.writerow(row1)?;
        }
        let contents = String::from_utf8(buffer.into_inner())?;
        assert_eq!(contents, "header1,header2\r\nvalue1,value2\r\n");
        Ok(())
    }
}
