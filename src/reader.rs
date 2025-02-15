use std::collections::HashMap;
use std::error::Error;
use std::io::{BufRead, BufReader, Read}; // Cursor 추가
use std::str;

#[derive(Debug, Clone, Copy)]
pub enum QuoteStyle {
    All,
    Minimal,
    NonNumeric,
    None,
}

impl Default for QuoteStyle {
    fn default() -> Self {
        QuoteStyle::Minimal
    }
}

#[derive(Debug, Clone, Copy)] // Clone and Copy added for testing
pub struct ReaderOptions {
    pub delimiter: u8,
    pub doublequote: bool,
    pub escapechar: Option<u8>,
    pub quotechar: u8,
    pub quoting: QuoteStyle,
    pub skipinitialspace: bool,
    pub strict: bool,
}

impl Default for ReaderOptions {
    fn default() -> Self {
        ReaderOptions {
            delimiter: b',',
            doublequote: true,
            escapechar: None,
            quotechar: b'"',
            quoting: QuoteStyle::Minimal,
            skipinitialspace: false,
            strict: false,
        }
    }
}

#[derive(Debug)]
pub struct DictReader<R: Read> {
    reader: BufReader<R>,
    header: Vec<String>,
    delimiter: u8,
    doublequote: bool,
    escapechar: Option<u8>,
    quotechar: u8,
    quoting: QuoteStyle,
    skipinitialspace: bool,
    strict: bool,
}

impl<R: Read> Iterator for DictReader<R> {
    type Item = Result<HashMap<String, String>, Box<dyn Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.read_record() {
            Ok(Some(record)) => Some(Ok(record)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

impl<R: Read> DictReader<R> {
    pub fn new(reader: R, options: ReaderOptions) -> Result<Self, Box<dyn Error>> {
        let mut buf_reader = BufReader::new(reader);
        let mut header_line = String::new();
        buf_reader.read_line(&mut header_line)?;

        let header = Self::parse_line(
            &header_line,
            options.delimiter,
            options.doublequote,
            options.escapechar,
            options.quotechar,
            options.quoting,
            options.skipinitialspace,
            options.strict,
        )?;

        Ok(DictReader {
            reader: buf_reader,
            header,
            delimiter: options.delimiter,
            doublequote: options.doublequote,
            escapechar: options.escapechar,
            quotechar: options.quotechar,
            quoting: options.quoting,
            skipinitialspace: options.skipinitialspace,
            strict: options.strict,
        })
    }

    pub fn read_record(&mut self) -> Result<Option<HashMap<String, String>>, Box<dyn Error>> {
        let mut current_line = String::new();
        let bytes_read = self.reader.read_line(&mut current_line)?;
        if bytes_read == 0 {
            return Ok(None);
        }

        let values = Self::parse_line(
            &current_line,
            self.delimiter,
            self.doublequote,
            self.escapechar,
            self.quotechar,
            self.quoting,
            self.skipinitialspace,
            self.strict,
        )?;

        if values.len() != self.header.len() {
            return Err(format!(
                "Number of fields in row does not match header: expected {}, got {}",
                self.header.len(),
                values.len()
            )
            .into());
        }

        let mut record = HashMap::new();
        for (i, field) in self.header.iter().enumerate() {
            record.insert(field.clone(), values[i].clone());
        }

        Ok(Some(record))
    }

    fn parse_line(
        line: &str,
        delimiter: u8,
        doublequote: bool,
        escapechar: Option<u8>,
        quotechar: u8,
        quoting: QuoteStyle,
        skipinitialspace: bool,
        strict: bool,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let mut fields = Vec::new();
        let mut current_field = String::new();
        let mut in_quote = false;
        let mut chars = line.chars().peekable();

        while let Some(c) = chars.next() {
            if in_quote {
                if c == quotechar as char {
                    // 따옴표 닫기 또는 이중 따옴표 처리
                    if doublequote && chars.peek() == Some(&(quotechar as char)) {
                        current_field.push(quotechar as char);
                        chars.next(); // Consume the second quote
                    } else {
                        in_quote = false;
                    }
                } else if let Some(escapechar) = escapechar {
                    if c == escapechar as char {
                        // 이스케이프 문자 처리
                        if let Some(next_c) = chars.next() {
                            current_field.push(next_c);
                        } else {
                            // 이스케이프 문자 뒤에 문자가 없으면 에러 처리
                            return Err("Invalid escape sequence at the end of the line".into());
                        }
                    } else {
                        current_field.push(c);
                    }
                } else {
                    current_field.push(c);
                }
            } else {
                if c == delimiter as char {
                    // 필드 구분자
                    fields.push(current_field.trim().to_string());
                    current_field.clear();
                } else if c == quotechar as char {
                    // 따옴표 열기
                    in_quote = true;
                } else if skipinitialspace && current_field.is_empty() && c.is_whitespace() {
                    // skipinitialspace가 true일 때, 구분자 뒤의 공백 무시
                    continue;
                } else {
                    current_field.push(c);
                }
            }
        }

        if strict && in_quote {
            return Err("Unclosed quote in strict mode".into());
        }

        fields.push(current_field.trim().to_string()); // 마지막 필드 추가
        Ok(fields)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_basic_csv_parsing() -> Result<(), Box<dyn Error>> {
        let data = "header1,header2\nvalue1,value2".to_string();
        let cursor = Cursor::new(data);
        let dict_reader = DictReader::new(cursor, ReaderOptions::default());
        let mut dict_reader = dict_reader?;
        let record = dict_reader.read_record()?.unwrap();
        assert_eq!(record.get("header1").unwrap(), "value1");
        assert_eq!(record.get("header2").unwrap(), "value2");
        Ok(())
    }

    #[test]
    fn test_custom_delimiter() -> Result<(), Box<dyn Error>> {
        let data = "header1;header2\nvalue1;value2".to_string();
        let cursor = Cursor::new(data);
        let options = ReaderOptions {
            delimiter: b';',
            ..Default::default()
        };
        let dict_reader = DictReader::new(cursor, options);
        let mut dict_reader = dict_reader?;
        let record = dict_reader.read_record()?.unwrap();
        assert_eq!(record.get("header1").unwrap(), "value1");
        assert_eq!(record.get("header2").unwrap(), "value2");
        Ok(())
    }

    #[test]
    fn test_quoted_fields() -> Result<(), Box<dyn Error>> {
        let data = "header1,header2\n\"value1,with,comma\",\"value2\"".to_string();
        let cursor = Cursor::new(data);
        let dict_reader = DictReader::new(cursor, ReaderOptions::default());
        let mut dict_reader = dict_reader?;
        let record = dict_reader.read_record()?.unwrap();
        assert_eq!(record.get("header1").unwrap(), "value1,with,comma");
        assert_eq!(record.get("header2").unwrap(), "value2");
        Ok(())
    }

    #[test]
    fn test_escape_char() -> Result<(), Box<dyn Error>> {
        let data = "header1,header2\nvalue1,\"value2\\\"with\\\"quotes\"".to_string();
        let cursor = Cursor::new(data);
        let options = ReaderOptions {
            escapechar: Some(b'\\'),
            ..Default::default()
        };
        let dict_reader = DictReader::new(cursor, options);
        let mut dict_reader = dict_reader?;
        let record = dict_reader.read_record()?.unwrap();
        assert_eq!(record.get("header1").unwrap(), "value1");
        assert_eq!(record.get("header2").unwrap(), "value2\"with\"quotes");
        Ok(())
    }

    #[test]
    fn test_empty_fields() -> Result<(), Box<dyn Error>> {
        let data = "header1,header2\nvalue1,".to_string();
        let cursor = Cursor::new(data);
        let dict_reader = DictReader::new(cursor, ReaderOptions::default());
        let mut dict_reader = dict_reader?;
        let record = dict_reader.read_record()?.unwrap();
        assert_eq!(record.get("header1").unwrap(), "value1");
        assert_eq!(record.get("header2").unwrap(), "");
        Ok(())
    }

    #[test]
    fn test_skip_initial_space() -> Result<(), Box<dyn Error>> {
        let data = "header1,header2\nvalue1, value2".to_string();
        let cursor = Cursor::new(data);
        let options = ReaderOptions {
            skipinitialspace: true,
            ..Default::default()
        };
        let dict_reader = DictReader::new(cursor, options);
        let mut dict_reader = dict_reader?;
        let record = dict_reader.read_record()?.unwrap();
        assert_eq!(record.get("header1").unwrap(), "value1");
        assert_eq!(record.get("header2").unwrap(), "value2");
        Ok(())
    }

    #[test]
    fn test_strict_mode_invalid_csv() -> Result<(), Box<dyn Error>> {
        let data = "header1,header2\nvalue1,\"value2".to_string(); // 닫히지 않은 따옴표
        let cursor = Cursor::new(data);
        let options = ReaderOptions {
            strict: true,
            ..Default::default()
        };
        let dict_reader = DictReader::new(cursor, options);
        let mut dict_reader = dict_reader?; // DictReader 생성
        let result = dict_reader.read_record(); // read_record 호출
        assert!(result.is_err()); // strict 모드에서는 에러가 발생해야 함
        Ok(())
    }

    #[test]
    fn test_non_default_quote_char() -> Result<(), Box<dyn Error>> {
        let data = "header1|header2\nvalue1|\'value with quote\'".to_string();
        let cursor = Cursor::new(data);
        let options = ReaderOptions {
            delimiter: b'|',
            quotechar: b'\'',
            ..Default::default()
        };
        let dict_reader = DictReader::new(cursor, options);
        let mut dict_reader = dict_reader?;
        let record = dict_reader.read_record()?.unwrap();
        assert_eq!(record.get("header1").unwrap(), "value1");
        assert_eq!(record.get("header2").unwrap(), "value with quote");
        Ok(())
    }

    #[test]
    fn test_different_quoting_styles() -> Result<(), Box<dyn Error>> {
        let data = "header1,header2\n\"value1\",\"value2\"".to_string();
        let cursor = Cursor::new(data);
        let options = ReaderOptions {
            quoting: QuoteStyle::All,
            ..Default::default()
        };
        let dict_reader = DictReader::new(cursor, options);
        let mut dict_reader = dict_reader?;
        let record = dict_reader.read_record()?.unwrap();
        assert_eq!(record.get("header1").unwrap(), "value1");
        assert_eq!(record.get("header2").unwrap(), "value2");
        Ok(())
    }

    #[test]
    fn test_default_options() -> Result<(), Box<dyn Error>> {
        let data = "header1,header2\nvalue1,value2".to_string();
        let cursor = Cursor::new(data);
        let options = ReaderOptions::default();
        let dict_reader = DictReader::new(cursor, options);
        let mut dict_reader = dict_reader?;
        let record = dict_reader.read_record()?.unwrap();
        assert_eq!(record.get("header1").unwrap(), "value1");
        assert_eq!(record.get("header2").unwrap(), "value2");
        Ok(())
    }
}
