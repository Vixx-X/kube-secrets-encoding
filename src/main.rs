use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(value_name = "FILE")]
    file: PathBuf,

    #[arg(short, long, value_name = "OUTPUT_FILE")]
    output: Option<PathBuf>,
}

fn yaml_value_to_string(value: &serde_yml::Value) -> String {
    let str = match value {
        serde_yml::Value::Number(v) => v.as_i64().unwrap().to_string(),
        serde_yml::Value::String(v) => v.as_str().to_string(),
        serde_yml::Value::Bool(v) => {
            if *v {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        serde_yml::Value::Null => "null".to_string(),
        _ => panic!("Invalid value {:?}", value),
    };

    str.trim().to_string()
}

fn process_key_map(
    yaml: &mut serde_yml::Mapping,
    key: &str,
    processor: impl FnMut(&mut serde_yml::Value),
) {
    if yaml.contains_key(key) {
        match yaml[key].as_mapping_mut() {
            Some(m) => m
                .values_mut()
                .filter(|value| !value.is_null())
                .for_each(processor),
            _ => {}
        };
    }
}

fn process_mapping(yaml: &mut serde_yml::Mapping) {
    process_key_map(yaml, "data", |value| {
        let string = yaml_value_to_string(value);
        let base64 = BASE64_STANDARD.encode(string);
        *value = serde_yml::Value::String(base64);
    });
    process_key_map(yaml, "dataString", |value| {
        let string = yaml_value_to_string(value);
        *value = serde_yml::Value::String(string);
    });
}

fn process_yaml(mut yaml: serde_yml::Value) -> serde_yml::Value {
    if yaml.is_mapping() {
        process_mapping(yaml.as_mapping_mut().unwrap());
    }
    yaml
}

fn main() {
    let args = Args::parse();

    let file_path = &args.file;
    let file = std::fs::File::open(file_path).expect("File not found");
    let yaml: serde_yml::Value = serde_yml::from_reader(file).expect("Invalid YAML");

    if args.output.is_some() {
        let output_path = args.output.unwrap();
        let output_file = std::fs::File::create(output_path).expect("Unable to create file");
        serde_yml::to_writer(output_file, &process_yaml(yaml)).expect("Unable to write file");
    } else {
        println!("{}", serde_yml::to_string(&process_yaml(yaml)).unwrap());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_test(input: &str, expected: &str) {
        let input_yaml = serde_yml::from_str(input.trim()).expect("Invalid YAML");
        let result = serde_yml::to_string(&process_yaml(input_yaml)).unwrap();

        let expected_yaml: serde_yml::Value =
            serde_yml::from_str(expected.trim()).expect("Invalid YAML");
        let expected_result = serde_yml::to_string(&expected_yaml).unwrap();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn data() {
        run_test(
            r###"apiVersion: v1
data:
  STRING: text
  NUMBER: 123
  BOOL: true
"###,
            r###"apiVersion: v1
data:
  STRING: "dGV4dA=="
  NUMBER: "MTIz"
  BOOL: "dHJ1ZQ=="
"###,
        );
    }

    #[test]
    fn data_string() {
        run_test(
            r###"apiVersion: v1
dataString:
  STRING: text
  NUMBER: 123
  BOOL: true
"###,
            r###"apiVersion: v1
dataString:
  STRING: "text"
  NUMBER: "123"
  BOOL: "true"
"###,
        );
    }

    #[test]
    fn data_and_data_string() {
        run_test(
            r###"apiVersion: v1
data:
  STRING: text
  NUMBER: 123
  BOOL: true
dataString:
  STRING: "text"
  NUMBER: 123
  BOOL: true
"###,
            r###"apiVersion: v1
data:
  STRING: "dGV4dA=="
  NUMBER: "MTIz"
  BOOL: "dHJ1ZQ=="
dataString:
  STRING: "text"
  NUMBER: "123"
  BOOL: "true"
"###,
        );
    }
}
