use std::collections::HashMap;
use walkdir::WalkDir;

use crate::{formats::ImageFormatType, util::to_dir};

#[derive(Debug)]
pub struct FormatResult {
    pub decode_min_time: u128,
    pub encode_min_time: u128,
    pub encode_size: usize,
}

#[derive(Debug)]
pub struct Test {
    pub name: String,
    pub input_size: usize,
    pub results: HashMap<ImageFormatType, FormatResult>,
}

#[derive(Debug)]
pub struct TestSuite {
    pub name: String,
    pub files: Vec<String>,
    pub tests: Vec<Test>,
}

pub fn generate_test_suites(path: &str) -> HashMap<String, TestSuite>  {
  let mut suites: HashMap<String, TestSuite> = HashMap::new();

  for entry in WalkDir::new(path) {
      let Ok(entry) = entry else {
          continue; 
      };

      let Some(path) = entry.path().to_str() else {
          continue;
      };

      if path == "images" || path.contains('.') && !path.ends_with(".png") {
          continue;
      }

      if entry.file_type().is_dir() {
          suites.insert(
              path.to_string(),
              TestSuite {
                  name: path.to_string(),
                  files: Vec::new(),
                  tests: Vec::new(),
              },
          );

          continue;
      }

      let Some(suite) = suites.get_mut(&to_dir(path)) else {
          println!("No suite for {}", path);
          continue;
      };

      suite.files.push(path.to_string());
  }
  suites
}