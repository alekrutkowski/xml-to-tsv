// Generated iteratively with 'ChatGPT 4o with canvas'

use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufReader, Write};

fn escape_newlines(value: &str) -> String {
    value.replace("\r\n", "\\n").replace("\n", "\\n")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get command line arguments for input and output file paths
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input_file> <output_file>", args[0]);
        std::process::exit(1);
    }
    let input_file_path = &args[1];
    let output_file_path = &args[2];

    // Open the input file and read its contents
    let input_file = File::open(input_file_path)?;
    let mut reader = Reader::from_reader(BufReader::new(input_file));
    reader.trim_text(true);

    // Prepare TSV output file
    let mut tsv_output = File::create(output_file_path)?;
    writeln!(tsv_output, "hierarchy\tattributes\tvalue")?;

    let mut buf = Vec::new();
    let mut tag_stack: Vec<String> = Vec::new();
    let mut attributes_stack: Vec<HashMap<String, String>> = Vec::new();
    let mut current_value = String::new();

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                // Extract the tag name
                let tag_name = String::from_utf8_lossy(e.name()).to_string();
                tag_stack.push(tag_name.clone());

                // Extract attributes and inherit from parent if available
                let mut current_attributes = HashMap::new();
                if let Some(parent_attrs) = attributes_stack.last() {
                    current_attributes.extend(parent_attrs.clone());
                }
                for attr in e.attributes() {
                    if let Ok(attr) = attr {
                        let key = String::from_utf8_lossy(attr.key).to_string();
                        let value = escape_newlines(&attr.unescape_and_decode_value(&reader)?);
                        current_attributes.insert(key, format!("\"{}\"", value)); // Add quotes around value
                    }
                }

                // Push current attributes to stack
                attributes_stack.push(current_attributes.clone());

                // Clear the value since we are starting a new tag
                current_value.clear();
            }
            Ok(Event::Empty(ref e)) => {
                // Handle self-closing tags (e.g., <tag ... />)
                let tag_name = String::from_utf8_lossy(e.name()).to_string();
                tag_stack.push(tag_name.clone());

                // Extract attributes and inherit from parent if available
                let mut current_attributes = HashMap::new();
                if let Some(parent_attrs) = attributes_stack.last() {
                    current_attributes.extend(parent_attrs.clone());
                }
                for attr in e.attributes() {
                    if let Ok(attr) = attr {
                        let key = String::from_utf8_lossy(attr.key).to_string();
                        let value = escape_newlines(&attr.unescape_and_decode_value(&reader)?);
                        current_attributes.insert(key, format!("\"{}\"", value)); // Add quotes around value
                    }
                }

                // Write the information to the TSV file
                let hierarchy = tag_stack.join("/");
                let attributes_str = current_attributes
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join(", ");

                writeln!(tsv_output, "{}\t{}\t", hierarchy, attributes_str)?;

                // Pop from the stack as this is a self-closing tag
                tag_stack.pop();
            }
            Ok(Event::Text(e)) => {
                // Collect the text content inside the tag
                current_value.push_str(&escape_newlines(&e.unescape_and_decode(&reader)?));
            }
            Ok(Event::End(_)) => {
                // Write the information to the TSV file only if this tag has content or attributes
                let hierarchy = tag_stack.join("/");
                let current_attributes = attributes_stack.last().unwrap();
                let attributes_str = current_attributes
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join(", ");

                if !current_value.is_empty() || !attributes_str.is_empty() {
                    writeln!(
                        tsv_output,
                        "{}\t{}\t{}",
                        hierarchy, attributes_str, current_value
                    )?;
                }

                // Pop the tag from the stack and remove the current attributes
                tag_stack.pop();
                attributes_stack.pop();

                // Clear temporary values
                current_value.clear();
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                eprintln!("Error at position {}: {:?}", reader.buffer_position(), e);
                break;
            }
            _ => {}
        }

        buf.clear();
    }

    Ok(())
}
