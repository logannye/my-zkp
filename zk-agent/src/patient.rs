//! Patient data structures and feature extraction
//!
//! This module handles patient medical records and converts them to
//! integer feature vectors for ZKP computation.

use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use chrono::{NaiveDate, Datelike};

use crate::icd_map::map_icd10;

/// Patient record as loaded from JSON (matches input file format)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PatientRecord {
    pub patient_id: String,
    pub dob: String,              // "YYYY-MM-DD"
    pub sex: String,              // "M" or "F"
    pub icd10_list: Vec<String>,  // ICD-10 diagnosis codes
    pub pregnant: bool,
    pub place_of_service: u32,    // HIPAA place of service code
    pub units: u32,               // Units requested
}

/// Extracted integer features (used in ZKP trace)
#[derive(Debug, Clone, Copy)]
pub struct PatientFeatures {
    pub age_years: u32,
    pub sex: u8,              // 0=M, 1=F
    pub primary_icd10: u32,   // Mapped from ICD-10 string
    pub pregnant: u8,         // 0=false, 1=true
    pub pos: u32,             // Place of service
    pub units: u32,           // Units requested
}

/// Patient data extractor and parser
pub struct PatientExtractor;

impl PatientExtractor {
    /// Load a patient record from a JSON file
    ///
    /// # Arguments
    /// * `path` - Path to the patient JSON file
    ///
    /// # Returns
    /// * `Ok(PatientRecord)` - Parsed patient record
    /// * `Err` - If file cannot be read or JSON is invalid
    pub fn from_file(path: &Path) -> Result<PatientRecord> {
        let json = fs::read_to_string(path)
            .with_context(|| format!("Failed to read patient file: {}", path.display()))?;
        
        let record: PatientRecord = serde_json::from_str(&json)
            .with_context(|| format!("Failed to parse patient JSON: {}", path.display()))?;
        
        Ok(record)
    }
    
    /// Extract integer features from a patient record
    ///
    /// # Arguments
    /// * `record` - The patient record to extract features from
    ///
    /// # Returns
    /// * `Ok(PatientFeatures)` - Extracted integer features
    /// * `Err` - If feature extraction fails (invalid data, unknown ICD-10, etc.)
    pub fn extract_features(record: &PatientRecord) -> Result<PatientFeatures> {
        // Compute age from DOB
        let age_years = Self::compute_age(&record.dob)
            .with_context(|| format!("Failed to compute age from DOB: {}", record.dob))?;
        
        // Map sex to integer
        let sex = match record.sex.as_str() {
            "M" => 0,
            "F" => 1,
            _ => anyhow::bail!("Invalid sex value: {}. Must be 'M' or 'F'", record.sex),
        };
        
        // Map primary ICD-10 (first in list) to integer
        if record.icd10_list.is_empty() {
            anyhow::bail!("Patient record has no ICD-10 codes");
        }
        let primary_icd10 = map_icd10(&record.icd10_list[0])
            .with_context(|| format!("Failed to map primary ICD-10: {}", record.icd10_list[0]))?;
        
        // Encode boolean as integer
        let pregnant = if record.pregnant { 1 } else { 0 };
        
        Ok(PatientFeatures {
            age_years,
            sex,
            primary_icd10,
            pregnant,
            pos: record.place_of_service,
            units: record.units,
        })
    }
    
    /// Compute age in years from date of birth
    ///
    /// # Arguments
    /// * `dob` - Date of birth string in "YYYY-MM-DD" format
    ///
    /// # Returns
    /// * `Ok(u32)` - Age in years (as of 2025)
    /// * `Err` - If date format is invalid
    fn compute_age(dob: &str) -> Result<u32> {
        let dob_date = NaiveDate::parse_from_str(dob, "%Y-%m-%d")
            .with_context(|| format!("Invalid date format: {}. Expected YYYY-MM-DD", dob))?;
        
        // Compute age as of 2025 (current year for demo)
        let current_year = 2025;
        let age = current_year - dob_date.year();
        
        if age < 0 {
            anyhow::bail!("Birth year {} is in the future", dob_date.year());
        }
        
        Ok(age as u32)
    }
}

impl PatientFeatures {
    /// Convert features to a byte array for hashing
    ///
    /// This produces a deterministic byte representation of the features
    /// for use in commitment computation.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.age_years.to_le_bytes());
        bytes.push(self.sex);
        bytes.extend_from_slice(&self.primary_icd10.to_le_bytes());
        bytes.push(self.pregnant);
        bytes.extend_from_slice(&self.pos.to_le_bytes());
        bytes.extend_from_slice(&self.units.to_le_bytes());
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_patient_file(json: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(json.as_bytes()).unwrap();
        file
    }

    #[test]
    fn test_load_patient_record() {
        let json = r#"{
            "patient_id": "test-123",
            "dob": "1979-03-15",
            "sex": "F",
            "icd10_list": ["C50.912", "E11.9"],
            "pregnant": false,
            "place_of_service": 22,
            "units": 1
        }"#;
        
        let file = create_test_patient_file(json);
        let record = PatientExtractor::from_file(file.path()).unwrap();
        
        assert_eq!(record.patient_id, "test-123");
        assert_eq!(record.dob, "1979-03-15");
        assert_eq!(record.sex, "F");
        assert_eq!(record.icd10_list, vec!["C50.912", "E11.9"]);
        assert_eq!(record.pregnant, false);
        assert_eq!(record.place_of_service, 22);
        assert_eq!(record.units, 1);
    }

    #[test]
    fn test_extract_features() {
        let record = PatientRecord {
            patient_id: "test-456".to_string(),
            dob: "1979-03-15".to_string(),
            sex: "F".to_string(),
            icd10_list: vec!["C50.912".to_string()],
            pregnant: false,
            place_of_service: 22,
            units: 1,
        };
        
        let features = PatientExtractor::extract_features(&record).unwrap();
        
        assert_eq!(features.age_years, 46); // 2025 - 1979
        assert_eq!(features.sex, 1);        // F
        assert_eq!(features.primary_icd10, 1002); // C50.912 mapped
        assert_eq!(features.pregnant, 0);   // false
        assert_eq!(features.pos, 22);
        assert_eq!(features.units, 1);
    }

    #[test]
    fn test_extract_features_male() {
        let record = PatientRecord {
            patient_id: "test-789".to_string(),
            dob: "1970-04-02".to_string(),
            sex: "M".to_string(),
            icd10_list: vec!["J18.9".to_string()],
            pregnant: false,
            place_of_service: 22,
            units: 1,
        };
        
        let features = PatientExtractor::extract_features(&record).unwrap();
        
        assert_eq!(features.age_years, 55);
        assert_eq!(features.sex, 0); // M
        assert_eq!(features.primary_icd10, 2001); // J18.9
    }

    #[test]
    fn test_extract_features_pregnant() {
        let record = PatientRecord {
            patient_id: "test-999".to_string(),
            dob: "1990-11-05".to_string(),
            sex: "F".to_string(),
            icd10_list: vec!["C50.912".to_string()],
            pregnant: true,
            place_of_service: 22,
            units: 1,
        };
        
        let features = PatientExtractor::extract_features(&record).unwrap();
        
        assert_eq!(features.pregnant, 1); // true
    }

    #[test]
    fn test_invalid_sex() {
        let record = PatientRecord {
            patient_id: "test-bad".to_string(),
            dob: "1980-01-01".to_string(),
            sex: "X".to_string(),
            icd10_list: vec!["C50.912".to_string()],
            pregnant: false,
            place_of_service: 22,
            units: 1,
        };
        
        assert!(PatientExtractor::extract_features(&record).is_err());
    }

    #[test]
    fn test_empty_icd10_list() {
        let record = PatientRecord {
            patient_id: "test-bad2".to_string(),
            dob: "1980-01-01".to_string(),
            sex: "F".to_string(),
            icd10_list: vec![],
            pregnant: false,
            place_of_service: 22,
            units: 1,
        };
        
        assert!(PatientExtractor::extract_features(&record).is_err());
    }

    #[test]
    fn test_compute_age() {
        assert_eq!(PatientExtractor::compute_age("1979-03-15").unwrap(), 46);
        assert_eq!(PatientExtractor::compute_age("2009-06-20").unwrap(), 16);
        assert_eq!(PatientExtractor::compute_age("1990-11-05").unwrap(), 35);
    }

    #[test]
    fn test_features_to_bytes() {
        let features = PatientFeatures {
            age_years: 45,
            sex: 1,
            primary_icd10: 1002,
            pregnant: 0,
            pos: 22,
            units: 1,
        };
        
        let bytes = features.to_bytes();
        assert!(!bytes.is_empty());
        
        // Verify determinism
        let bytes2 = features.to_bytes();
        assert_eq!(bytes, bytes2);
    }
}

