//! ICD-10 to integer mapping for feature extraction
//!
//! This module provides a static mapping from ICD-10 diagnosis codes (strings)
//! to integers for use in the ZKP computation trace.

use std::collections::HashMap;
use lazy_static::lazy_static;
use anyhow::Result;

lazy_static! {
    /// Static ICD-10 code to integer mapping
    pub static ref ICD_MAP: HashMap<&'static str, u32> = {
        let mut m = HashMap::new();
        
        // Breast cancer / biopsy (1000 series)
        m.insert("C50.911", 1001);  // Malignant neoplasm of unspecified site of right female breast
        m.insert("C50.912", 1002);  // Malignant neoplasm of unspecified site of left female breast
        m.insert("D05.10", 1003);   // Intraductal carcinoma in situ of unspecified breast
        
        // Chest / lung (2000 series)
        m.insert("J18.9", 2001);    // Pneumonia, unspecified organism
        m.insert("C34.90", 2002);   // Malignant neoplasm of unspecified part of unspecified bronchus or lung
        m.insert("R91.1", 2003);    // Solitary pulmonary nodule
        
        // Neurological (3000 series)
        m.insert("G43.909", 3001);  // Migraine, unspecified, not intractable, without status migrainosus
        m.insert("S06.0", 3002);    // Concussion
        m.insert("G40.909", 3003);  // Epilepsy, unspecified, not intractable, without status epilepticus
        m.insert("G45.9", 3004);    // Transient cerebral ischemic attack, unspecified
        
        // Screening / Preventive (4000 series)
        m.insert("Z12.11", 4001);   // Encounter for screening for malignant neoplasm of colon
        
        // Musculoskeletal / Physical Therapy (5000 series)
        m.insert("M54.5", 5001);    // Low back pain
        m.insert("S83.5", 5002);    // Sprain of cruciate ligament of knee
        
        // GI / Rheumatology - Specialty Drugs (6000 series)
        m.insert("K50.90", 6001);   // Crohn's disease, unspecified, without complications
        m.insert("M05.9", 6002);    // Rheumatoid arthritis, unspecified
        
        // Dental (7000 series)
        m.insert("K04.7", 7001);    // Periapical abscess without sinus
        
        // Other / comorbidities (9000 series)
        m.insert("E11.9", 9001);    // Type 2 diabetes mellitus without complications
        
        m
    };
}

/// Map an ICD-10 code string to its integer representation
///
/// # Arguments
/// * `icd10` - The ICD-10 code string (e.g., "C50.912")
///
/// # Returns
/// * `Ok(u32)` - The mapped integer value
/// * `Err` - If the ICD-10 code is not in the mapping table
///
/// # Example
/// ```
/// use zk_agent::icd_map::map_icd10;
/// 
/// let code = map_icd10("C50.912").unwrap();
/// assert_eq!(code, 1002);
/// ```
pub fn map_icd10(icd10: &str) -> Result<u32> {
    ICD_MAP
        .get(icd10)
        .copied()
        .ok_or_else(|| anyhow::anyhow!("Unknown ICD-10 code: {}", icd10))
}

/// Check if an ICD-10 code is in the mapping table
pub fn is_valid_icd10(icd10: &str) -> bool {
    ICD_MAP.contains_key(icd10)
}

/// Get all supported ICD-10 codes
pub fn get_all_codes() -> Vec<&'static str> {
    ICD_MAP.keys().copied().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_breast_cancer_codes() {
        assert_eq!(map_icd10("C50.911").unwrap(), 1001);
        assert_eq!(map_icd10("C50.912").unwrap(), 1002);
        assert_eq!(map_icd10("D05.10").unwrap(), 1003);
    }

    #[test]
    fn test_map_chest_codes() {
        assert_eq!(map_icd10("J18.9").unwrap(), 2001);
        assert_eq!(map_icd10("C34.90").unwrap(), 2002);
        assert_eq!(map_icd10("R91.1").unwrap(), 2003);
    }

    #[test]
    fn test_map_neuro_codes() {
        assert_eq!(map_icd10("G43.909").unwrap(), 3001);
        assert_eq!(map_icd10("S06.0").unwrap(), 3002);
        assert_eq!(map_icd10("G40.909").unwrap(), 3003);
        assert_eq!(map_icd10("G45.9").unwrap(), 3004);
    }

    #[test]
    fn test_unknown_code() {
        assert!(map_icd10("Z99.999").is_err());
    }

    #[test]
    fn test_is_valid() {
        assert!(is_valid_icd10("C50.912"));
        assert!(!is_valid_icd10("INVALID"));
    }
}

