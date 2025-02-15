# csvkit: CSV Processing Toolkit in Rust

csvkit is a CSV file processing toolkit written in Rust. It aims to provide a user-friendly interface, closely mirroring the Python csvkit API, for efficient and reliable CSV manipulation. csvkit-rs leverages Rust's performance, safety, and ease of use to provide a superior CSV processing experience.

### Key Features

*   **Familiar API:** Designed to be as close as possible to the Python csvkit API, making it easy for Python users to transition.
*   **High Performance:** Utilizes Rust's performance benefits for fast and efficient CSV processing.
*   **Reliability:** Leverages Rust's strong type system and memory safety guarantees for robust and stable operation.
*   **Modular Design:** Separates Reader and Writer functionalities, enabling flexible CSV processing pipelines.

### Installation

1.  **Install Rust:** If you haven't already, install Rust from [rustup.rs](https://rustup.rs/).
2.  **add csvkit from github:**

    ```bash
    cargo add --git https://github.com/StatPan/csvkit
    ```

### Usage

#### Reader (DictReader)

The `DictReader` reads CSV files and returns each row as a `HashMap<String, String>`.

```rust
use csvkit::{
    reader::{DictReader, ReaderOptions},
    writer::{DictWriter, WriterOptions},
};
use std::fs::File;
use std::io::BufReader;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("data.csv")?;
    let reader = BufReader::new(file);
    let options = ReaderOptions::default();
    let mut dict_reader = DictReader::new(reader, options)?;
    for record in dict_reader {
        let row: HashMap<String, String> = record?;
        println!("{:?}", row);
    }
    Ok(())
}
```

*   `DictReader::new(reader, options)`: Creates a `DictReader` with a `BufReader` and `ReaderOptions`.
*   `for record in dict_reader`: `DictReader` implements the `Iterator` trait, allowing you to iterate over each row in the CSV file.
*   `record?`: Each row is returned as a `Result<HashMap<String, String>, Box<dyn Error>>`, so the `?` operator is used for error handling.

#### Writer (DictWriter)

The `DictWriter` writes data in the form of `HashMap<String, String>` to a CSV file.

```rust
use csvkit::{
    reader::ReaderOptions, // Import necessary for options configuration
    writer::{DictWriter, WriterOptions},
};
use std::fs::File;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create("output.csv")?;
    let fieldnames = vec!["header1".to_string(), "header2".to_string()];
    let options = WriterOptions::default();
    let mut writer = DictWriter::new(file, fieldnames, options);
    writer.writeheader()?;
    let mut row1 = HashMap::new();
    row1.insert("header1".to_string(), "value1".to_string());
    row1.insert("header2".to_string(), "value2".to_string());
    writer.writerow(row1)?;
    let mut row2 = HashMap::new();
    row2.insert("header1".to_string(), "value3".to_string());
    row2.insert("header2".to_string(), "value4".to_string());
    writer.writerow(row2)?;
    Ok(())
}
```

#### Writerows

The `writerows` method allows you to write multiple rows at once.

```rust
use csvkit::{
    reader::ReaderOptions, // Import necessary for options configuration
    writer::{DictWriter, WriterOptions},
};
use std::fs::File;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create("output.csv")?;
    let fieldnames = vec!["header1".to_string(), "header2".to_string()];
    let options = WriterOptions::default();
    let mut writer = DictWriter::new(file, fieldnames, options);
    writer.writeheader()?;
    let mut rows: Vec<HashMap<String, String>> = Vec::new();
    let mut row1 = HashMap::new();
    row1.insert("header1".to_string(), "value1".to_string());
    row1.insert("header2".to_string(), "value2".to_string());
    rows.push(row1);
    let mut row2 = HashMap::new();
    row2.insert("header1".to_string(), "value3".to_string());
    row2.insert("header2".to_string(), "value4".to_string());
    rows.push(row2);
    writer.writerows(rows)?;
    Ok(())
}
```

### Options

You can control the CSV processing behavior using the `ReaderOptions` and `WriterOptions` structs.

*   `delimiter`: The field delimiter (default: `,`)
*   `quotechar`: The quote character (default: `"`)