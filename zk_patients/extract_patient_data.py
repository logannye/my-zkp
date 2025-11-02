"""
LLM Agent that extracts patient data from PDF files and generates JSON.
Uses OpenAI API to parse patient medical records and extract structured data.
"""

import os
import json
from pathlib import Path
from typing import Dict, Any, Optional
from dotenv import load_dotenv
from openai import OpenAI
import pypdf

# Load environment variables
load_dotenv()

# Initialize OpenAI client
client = OpenAI(api_key=os.getenv("OPENAI_API_KEY"))

def extract_text_from_pdf(pdf_path: str) -> str:
    """Extract text content from a PDF file."""
    try:
        with open(pdf_path, 'rb') as file:
            reader = pypdf.PdfReader(file)
            text = ""
            for page in reader.pages:
                text += page.extract_text() + "\n"
        return text
    except Exception as e:
        raise Exception(f"Error reading PDF {pdf_path}: {e}")

def extract_patient_data_with_llm(pdf_text: str, pdf_filename: str) -> Dict[str, Any]:
    """
    Use OpenAI API to extract patient data from PDF text.
    Returns JSON with patient information compatible with rules.json processing.
    """
    
    prompt = f"""You are a medical data extraction assistant. Extract patient information from the following medical record text and return it as a JSON object with the exact structure specified below.

Medical Record Text:
{pdf_text}

Extract the following information and return ONLY valid JSON (no markdown, no code blocks):
{{
  "patient_id": "string (extract from Patient ID or generate if not found)",
  "dob": "YYYY-MM-DD (date of birth)",
  "sex": "F or M (gender)",
  "icd10_list": ["array", "of", "ICD-10", "codes", "found", "in", "the", "document"],
  "pregnant": boolean (true/false, only for female patients),
  "place_of_service": integer (HIPAA place of service code, default to 22 if not found),
  "units": integer (units requested, default to 1 if not found)
}}

IMPORTANT:
- Extract all ICD-10-CM diagnosis codes from the document (format: Letter followed by numbers, may include dots like C50.912)
- If sex is Female or F, check for pregnancy status
- If sex is Male or M, pregnant should always be false
- Place of service should be a numeric code (11=office, 22=outpatient hospital, 23=emergency room, etc.)
- Units should be an integer (typically 1)
- If any information is missing, use reasonable defaults but extract what you can find
- Return ONLY the JSON object, no explanation, no markdown formatting

Return the JSON now:"""

    try:
        response = client.chat.completions.create(
            model="gpt-4o-mini",  # Using mini for cost efficiency, can switch to gpt-4o for better accuracy
            messages=[
                {"role": "system", "content": "You are a precise medical data extraction assistant. Return only valid JSON."},
                {"role": "user", "content": prompt}
            ],
            temperature=0.1,  # Low temperature for consistent extraction
            response_format={"type": "json_object"}  # Ensure JSON response
        )
        
        result_text = response.choices[0].message.content.strip()
        
        # Remove markdown code blocks if present
        if result_text.startswith("```json"):
            result_text = result_text[7:]
        if result_text.startswith("```"):
            result_text = result_text[3:]
        if result_text.endswith("```"):
            result_text = result_text[:-3]
        result_text = result_text.strip()
        
        # Parse JSON
        patient_data = json.loads(result_text)
        
        # Validate and normalize
        patient_data = normalize_patient_data(patient_data, pdf_filename)
        
        return patient_data
        
    except json.JSONDecodeError as e:
        raise Exception(f"Failed to parse JSON response from LLM: {e}")
    except Exception as e:
        raise Exception(f"OpenAI API error: {e}")

def normalize_patient_data(data: Dict[str, Any], filename: str) -> Dict[str, Any]:
    """Normalize and validate patient data structure."""
    
    # Ensure required fields exist
    normalized = {
        "patient_id": data.get("patient_id", f"PAT{filename.replace('.pdf', '')}"),
        "dob": data.get("dob", "1970-01-01"),
        "sex": data.get("sex", "F").upper(),
        "icd10_list": data.get("icd10_list", []),
        "pregnant": data.get("pregnant", False),
        "place_of_service": int(data.get("place_of_service", 22)),
        "units": int(data.get("units", 1))
    }
    
    # Normalize ICD-10 codes (uppercase, keep dots for now, will be normalized later)
    normalized["icd10_list"] = [code.upper().strip() for code in normalized["icd10_list"] if code]
    
    # If male, pregnant must be false
    if normalized["sex"] == "M":
        normalized["pregnant"] = False
    
    # Validate date format (YYYY-MM-DD)
    if len(normalized["dob"]) != 10 or normalized["dob"][4] != "-" or normalized["dob"][7] != "-":
        # Try to fix date format
        try:
            from datetime import datetime
            # Try parsing various formats
            for fmt in ["%Y-%m-%d", "%m/%d/%Y", "%m-%d-%Y", "%d/%m/%Y"]:
                try:
                    dt = datetime.strptime(normalized["dob"], fmt)
                    normalized["dob"] = dt.strftime("%Y-%m-%d")
                    break
                except:
                    continue
        except:
            normalized["dob"] = "1970-01-01"
    
    return normalized

def process_patient_pdf(pdf_path: str, output_dir: str = "output") -> Dict[str, Any]:
    """
    Process a single patient PDF and generate JSON output.
    
    Args:
        pdf_path: Path to the PDF file
        output_dir: Directory to save the JSON output
        
    Returns:
        Dictionary with patient data
    """
    print(f"\nProcessing: {pdf_path}")
    
    # Extract text from PDF
    print("  Extracting text from PDF...")
    pdf_text = extract_text_from_pdf(pdf_path)
    
    if not pdf_text.strip():
        raise Exception(f"No text could be extracted from {pdf_path}")
    
    print(f"  Extracted {len(pdf_text)} characters of text")
    
    # Extract patient data using LLM
    print("  Calling OpenAI API to extract patient data...")
    filename = os.path.basename(pdf_path)
    patient_data = extract_patient_data_with_llm(pdf_text, filename)
    
    print(f"  ✓ Extracted patient data:")
    print(f"    Patient ID: {patient_data['patient_id']}")
    print(f"    DOB: {patient_data['dob']}")
    print(f"    Sex: {patient_data['sex']}")
    print(f"    ICD-10 codes: {', '.join(patient_data['icd10_list'])}")
    print(f"    Place of Service: {patient_data['place_of_service']}")
    print(f"    Units: {patient_data['units']}")
    
    # Save JSON output
    os.makedirs(output_dir, exist_ok=True)
    json_filename = os.path.basename(pdf_path).replace(".pdf", ".json")
    json_path = os.path.join(output_dir, json_filename)
    
    with open(json_path, "w") as f:
        json.dump(patient_data, f, indent=2, ensure_ascii=False)
    
    print(f"  ✓ Saved JSON to: {json_path}")
    
    return patient_data

def process_all_patients(patients_dir: str = "patients", output_dir: str = "output") -> list:
    """Process all PDF files in the patients directory."""
    
    patients_path = Path(patients_dir)
    if not patients_path.exists():
        raise Exception(f"Patients directory not found: {patients_dir}")
    
    pdf_files = list(patients_path.glob("*.pdf"))
    
    if not pdf_files:
        raise Exception(f"No PDF files found in {patients_dir}")
    
    print(f"\nFound {len(pdf_files)} PDF file(s) to process\n")
    
    results = []
    for pdf_file in sorted(pdf_files):
        try:
            patient_data = process_patient_pdf(str(pdf_file), output_dir)
            results.append(patient_data)
        except Exception as e:
            print(f"  ✗ Error processing {pdf_file}: {e}")
            continue
    
    print(f"\n✓ Successfully processed {len(results)}/{len(pdf_files)} patient PDFs")
    print(f"  JSON files saved to: {output_dir}/")
    
    return results

def main():
    """Main entry point."""
    import sys
    
    # Check for OpenAI API key
    if not os.getenv("OPENAI_API_KEY"):
        print("ERROR: OPENAI_API_KEY not found in environment variables.")
        print("Please create a .env file with your OpenAI API key:")
        print("  OPENAI_API_KEY=your_key_here")
        sys.exit(1)
    
    # Process all patient PDFs
    try:
        results = process_all_patients()
        
        # Summary
        print("\n" + "="*70)
        print("SUMMARY")
        print("="*70)
        print(f"Total patients processed: {len(results)}")
        total_icd_codes = sum(len(p["icd10_list"]) for p in results)
        print(f"Total ICD-10 codes extracted: {total_icd_codes}")
        print(f"Average ICD-10 codes per patient: {total_icd_codes/len(results):.1f}" if results else "N/A")
        print("="*70)
        
    except Exception as e:
        print(f"\nERROR: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()

