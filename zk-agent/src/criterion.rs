//! Criterion evaluation logic
//!
//! This module handles parsing and evaluation of policy criteria
//! (inclusion and exclusion rules) against patient features.

use serde_json::Value;
use anyhow::{Result, Context};
use myzkp::F;

use crate::patient::PatientFeatures;

/// A criterion from a policy (e.g., age >= 18, ICD in list)
#[derive(Debug, Clone)]
pub enum Criterion {
    Eq { field: String, value: i64 },
    Neq { field: String, value: i64 },
    Lt { field: String, value: i64 },
    Lte { field: String, value: i64 },
    Gt { field: String, value: i64 },
    Gte { field: String, value: i64 },
    In { field: String, values: Vec<u32> },
}

impl Criterion {
    /// Parse a criterion from JSON
    ///
    /// # Arguments
    /// * `json` - JSON value representing the criterion
    ///
    /// # Returns
    /// * `Ok(Criterion)` - Parsed criterion
    /// * `Err` - If JSON format is invalid
    ///
    /// # Expected JSON formats:
    /// - `{"eq": ["field_name", value]}`
    /// - `{"gte": ["field_name", value]}`
    /// - `{"in": ["field_name", [val1, val2, val3]]}`
    pub fn from_json(json: &Value) -> Result<Self> {
        let obj = json.as_object()
            .ok_or_else(|| anyhow::anyhow!("Criterion must be a JSON object"))?;
        
        if obj.len() != 1 {
            anyhow::bail!("Criterion must have exactly one operator key");
        }
        
        let (op, args) = obj.iter().next().unwrap();
        let args_arr = args.as_array()
            .ok_or_else(|| anyhow::anyhow!("Criterion arguments must be an array"))?;
        
        match op.as_str() {
            "eq" => Self::parse_comparison(args_arr, "eq"),
            "neq" => Self::parse_comparison(args_arr, "neq"),
            "lt" => Self::parse_comparison(args_arr, "lt"),
            "lte" => Self::parse_comparison(args_arr, "lte"),
            "gt" => Self::parse_comparison(args_arr, "gt"),
            "gte" => Self::parse_comparison(args_arr, "gte"),
            "in" => Self::parse_in(args_arr),
            _ => anyhow::bail!("Unknown criterion operator: {}", op),
        }
    }
    
    fn parse_comparison(args: &[Value], op: &str) -> Result<Self> {
        if args.len() != 2 {
            anyhow::bail!("{} criterion must have exactly 2 arguments [field, value]", op);
        }
        
        let field = args[0].as_str()
            .ok_or_else(|| anyhow::anyhow!("Field name must be a string"))?
            .to_string();
        
        let value = args[1].as_i64()
            .ok_or_else(|| anyhow::anyhow!("Comparison value must be an integer"))?;
        
        match op {
            "eq" => Ok(Criterion::Eq { field, value }),
            "neq" => Ok(Criterion::Neq { field, value }),
            "lt" => Ok(Criterion::Lt { field, value }),
            "lte" => Ok(Criterion::Lte { field, value }),
            "gt" => Ok(Criterion::Gt { field, value }),
            "gte" => Ok(Criterion::Gte { field, value }),
            _ => unreachable!(),
        }
    }
    
    fn parse_in(args: &[Value]) -> Result<Self> {
        if args.len() != 2 {
            anyhow::bail!("'in' criterion must have exactly 2 arguments [field, values]");
        }
        
        let field = args[0].as_str()
            .ok_or_else(|| anyhow::anyhow!("Field name must be a string"))?
            .to_string();
        
        let values_arr = args[1].as_array()
            .ok_or_else(|| anyhow::anyhow!("'in' values must be an array"))?;
        
        let mut values = Vec::new();
        for val in values_arr {
            let num = val.as_u64()
                .ok_or_else(|| anyhow::anyhow!("'in' values must be integers"))?;
            values.push(num as u32);
        }
        
        Ok(Criterion::In { field, values })
    }
    
    /// Evaluate this criterion against patient features
    ///
    /// # Arguments
    /// * `features` - Patient features to evaluate against
    ///
    /// # Returns
    /// * `Ok(true)` - Criterion passes
    /// * `Ok(false)` - Criterion fails
    /// * `Err` - If field name is unknown
    pub fn evaluate(&self, features: &PatientFeatures) -> Result<bool> {
        match self {
            Criterion::Eq { field, value } => {
                let patient_val = Self::get_field_value(features, field)?;
                Ok(patient_val == *value)
            }
            Criterion::Neq { field, value } => {
                let patient_val = Self::get_field_value(features, field)?;
                Ok(patient_val != *value)
            }
            Criterion::Lt { field, value } => {
                let patient_val = Self::get_field_value(features, field)?;
                Ok(patient_val < *value)
            }
            Criterion::Lte { field, value } => {
                let patient_val = Self::get_field_value(features, field)?;
                Ok(patient_val <= *value)
            }
            Criterion::Gt { field, value } => {
                let patient_val = Self::get_field_value(features, field)?;
                Ok(patient_val > *value)
            }
            Criterion::Gte { field, value } => {
                let patient_val = Self::get_field_value(features, field)?;
                Ok(patient_val >= *value)
            }
            Criterion::In { field, values } => {
                let patient_val = Self::get_field_value(features, field)? as u32;
                Ok(values.contains(&patient_val))
            }
        }
    }
    
    /// Extract the patient's value for the criterion's field
    ///
    /// Returns as a field element for use in ZKP trace
    pub fn extract_patient_value(&self, features: &PatientFeatures) -> Result<F> {
        let field = match self {
            Criterion::Eq { field, .. } | Criterion::Neq { field, .. } |
            Criterion::Lt { field, .. } | Criterion::Lte { field, .. } |
            Criterion::Gt { field, .. } | Criterion::Gte { field, .. } |
            Criterion::In { field, .. } => field,
        };
        
        let val = Self::get_field_value(features, field)?;
        Ok(F::from(val as u64))
    }
    
    /// Extract the policy's threshold/list value
    ///
    /// For comparison operators, returns the threshold value.
    /// For 'in' operator, returns the first value in the list (for trace purposes).
    pub fn extract_policy_value(&self) -> F {
        match self {
            Criterion::Eq { value, .. } | Criterion::Neq { value, .. } |
            Criterion::Lt { value, .. } | Criterion::Lte { value, .. } |
            Criterion::Gt { value, .. } | Criterion::Gte { value, .. } => {
                F::from(*value as u64)
            }
            Criterion::In { values, .. } => {
                if values.is_empty() {
                    F::from(0u64)
                } else {
                    F::from(values[0] as u64)
                }
            }
        }
    }
    
    /// Get the field name for this criterion
    pub fn field_name(&self) -> &str {
        match self {
            Criterion::Eq { field, .. } | Criterion::Neq { field, .. } |
            Criterion::Lt { field, .. } | Criterion::Lte { field, .. } |
            Criterion::Gt { field, .. } | Criterion::Gte { field, .. } |
            Criterion::In { field, .. } => field,
        }
    }
    
    /// Helper: Get a field value from patient features as i64
    fn get_field_value(features: &PatientFeatures, field: &str) -> Result<i64> {
        match field {
            "age_years" => Ok(features.age_years as i64),
            "sex" => Ok(features.sex as i64),
            "primary_icd10" => Ok(features.primary_icd10 as i64),
            "pregnant" => Ok(features.pregnant as i64),
            "pos" => Ok(features.pos as i64),
            "units" => Ok(features.units as i64),
            _ => anyhow::bail!("Unknown field name: {}", field),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn test_features() -> PatientFeatures {
        PatientFeatures {
            age_years: 45,
            sex: 1,  // F
            primary_icd10: 1002,
            pregnant: 0,
            pos: 22,
            units: 1,
        }
    }

    #[test]
    fn test_parse_gte_criterion() {
        let json = json!({"gte": ["age_years", 18]});
        let criterion = Criterion::from_json(&json).unwrap();
        
        match criterion {
            Criterion::Gte { field, value } => {
                assert_eq!(field, "age_years");
                assert_eq!(value, 18);
            }
            _ => panic!("Expected Gte variant"),
        }
    }

    #[test]
    fn test_parse_in_criterion() {
        let json = json!({"in": ["primary_icd10", [1001, 1002, 1003]]});
        let criterion = Criterion::from_json(&json).unwrap();
        
        match criterion {
            Criterion::In { field, values } => {
                assert_eq!(field, "primary_icd10");
                assert_eq!(values, vec![1001, 1002, 1003]);
            }
            _ => panic!("Expected In variant"),
        }
    }

    #[test]
    fn test_parse_eq_criterion() {
        let json = json!({"eq": ["pregnant", 1]});
        let criterion = Criterion::from_json(&json).unwrap();
        
        match criterion {
            Criterion::Eq { field, value } => {
                assert_eq!(field, "pregnant");
                assert_eq!(value, 1);
            }
            _ => panic!("Expected Eq variant"),
        }
    }

    #[test]
    fn test_evaluate_gte_pass() {
        let json = json!({"gte": ["age_years", 18]});
        let criterion = Criterion::from_json(&json).unwrap();
        let features = test_features();
        
        assert_eq!(criterion.evaluate(&features).unwrap(), true);
    }

    #[test]
    fn test_evaluate_gte_fail() {
        let json = json!({"gte": ["age_years", 50]});
        let criterion = Criterion::from_json(&json).unwrap();
        let features = test_features();
        
        assert_eq!(criterion.evaluate(&features).unwrap(), false);
    }

    #[test]
    fn test_evaluate_in_pass() {
        let json = json!({"in": ["primary_icd10", [1001, 1002, 1003]]});
        let criterion = Criterion::from_json(&json).unwrap();
        let features = test_features();
        
        assert_eq!(criterion.evaluate(&features).unwrap(), true);
    }

    #[test]
    fn test_evaluate_in_fail() {
        let json = json!({"in": ["primary_icd10", [2001, 2002, 2003]]});
        let criterion = Criterion::from_json(&json).unwrap();
        let features = test_features();
        
        assert_eq!(criterion.evaluate(&features).unwrap(), false);
    }

    #[test]
    fn test_evaluate_eq_pass() {
        let json = json!({"eq": ["sex", 1]});
        let criterion = Criterion::from_json(&json).unwrap();
        let features = test_features();
        
        assert_eq!(criterion.evaluate(&features).unwrap(), true);
    }

    #[test]
    fn test_evaluate_eq_fail() {
        let json = json!({"eq": ["pregnant", 1]});
        let criterion = Criterion::from_json(&json).unwrap();
        let features = test_features();
        
        assert_eq!(criterion.evaluate(&features).unwrap(), false);
    }

    #[test]
    fn test_extract_patient_value() {
        let json = json!({"gte": ["age_years", 18]});
        let criterion = Criterion::from_json(&json).unwrap();
        let features = test_features();
        
        let value = criterion.extract_patient_value(&features).unwrap();
        assert_eq!(value, F::from(45u64));
    }

    #[test]
    fn test_extract_policy_value() {
        let json = json!({"gte": ["age_years", 18]});
        let criterion = Criterion::from_json(&json).unwrap();
        
        let value = criterion.extract_policy_value();
        assert_eq!(value, F::from(18u64));
    }

    #[test]
    fn test_unknown_field() {
        let json = json!({"eq": ["invalid_field", 1]});
        let criterion = Criterion::from_json(&json).unwrap();
        let features = test_features();
        
        assert!(criterion.evaluate(&features).is_err());
    }
}

