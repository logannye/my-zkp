"""
Agent that processes verified Medicare Coverage Database articles,
extracts CPT codes and ICD-10-CM codes, and generates rules.json files.

This agent:
1. Processes a list of Medicare article IDs (without version numbers)
2. Fetches articles from URLs like: https://www.cms.gov/medicare-coverage-database/view/article.aspx?articleId=57765
3. Parses HTML content to extract CPT codes and ICD-10-CM codes
4. Generates rules.json files (one per CPT code) in the rules folder
"""

import json
import os
import re
import time
import hashlib
from typing import List, Dict, Optional, Any, Set, Tuple

try:
    import requests
    from bs4 import BeautifulSoup
except ImportError:
    print("ERROR: Required packages not installed. Run: pip install requests beautifulsoup4")
    exit(1)

OUT_DIR = "rules"

# Medicare Coverage Database base URL
CMS_BASE_URL = "https://www.cms.gov"
CMS_MCD_BASE = f"{CMS_BASE_URL}/medicare-coverage-database"

# List of article IDs to process (without version numbers)
# These will be fetched as: https://www.cms.gov/medicare-coverage-database/view/article.aspx?articleId={id}
# Expanded list - will try articles until we successfully parse 50
ARTICLE_IDS = [
    57765,  # Major Joint Replacement (Hip and Knee)
    57021, 57022, 57023, 57024, 57025,
    57026, 57027, 57028, 57029, 57030,
    57031, 57032, 57033, 57034, 57035,
    57036, 57037, 57038, 57039, 57040,
    57041, 57042, 57043, 57044, 57045,
    57046, 57047, 57048, 57049, 57050,
    57051, 57052, 57053, 57054, 57055,
    57056, 57057, 57058, 57059, 57060,
    57061, 57062, 57063, 57064, 57065,
    57066, 57067, 57068, 57069, 57070,
    57071, 57072, 57073, 57074, 57075,
    57076, 57077, 57078, 57079, 57080,
    57081, 57082, 57083, 57084, 57085,
    57086, 57087, 57088, 57089, 57090,
    57091, 57092, 57093, 57094, 57095,
    57096, 57097, 57098, 57099, 57100,
]


def normalize_icd(code: str) -> str:
    """Normalize ICD-10 code (remove dots, uppercase)."""
    return code.replace(".", "").upper().strip()

def icd10_to_integer(icd_code: str) -> int:
    """
    Map ICD-10 code to a small integer for ZK processing.
    Uses a deterministic hash function.
    """
    normalized = normalize_icd(icd_code)
    hash_bytes = hashlib.sha256(normalized.encode('utf-8')).digest()[:4]
    integer_id = int.from_bytes(hash_bytes, byteorder='big') % 100000
    return integer_id


def extract_icd10_codes(text: str) -> List[str]:
    """Extract ICD-10-CM codes from text using regex patterns."""
    icd_codes = set()
    
    patterns = [
        r'\b([A-Z]\d{2}\.?\d{0,2})\b',  # Standard ICD-10 format
        r'ICD[- ]?10[- ]?CM[- ]?[Cc]ode[s]?[:\s]+([A-Z]\d{2}\.?\d{0,2})',
        r'ICD[- ]?10[- ]?[Cc]ode[s]?[:\s]+([A-Z]\d{2}\.?\d{0,2})',
        r'([A-Z]\d{2}\.\d{1,2})',  # With decimal
        r'ICD10CM[:]?\s*([A-Z]\d{2}\.?\d{0,2})',
        r'Diagnosis[:\s]+([A-Z]\d{2}\.?\d{0,2})',
    ]
    
    for pattern in patterns:
        matches = re.findall(pattern, text, re.IGNORECASE)
        for match in matches:
            normalized = normalize_icd(match)
            # ICD-10-CM codes are 3-7 characters
            if 3 <= len(normalized) <= 7 and normalized[0].isalpha():
                icd_codes.add(normalized)
    
    return sorted(list(icd_codes))


def extract_cpt_codes(text: str) -> List[str]:
    """Extract CPT/HCPCS codes from text."""
    cpt_codes = set()
    
    # Look for 5-digit codes that are CPT/HCPCS codes
    patterns = [
        r'\b(9\d{4})\b',  # CPT codes typically start with 9
        r'\b([2345678]\d{4})\b',  # Other CPT codes
        r'CPT[:\s]+(\d{5})',
        r'HCPCS[:\s]+(\d{5})',
        r'code[s]?[:\s]+(\d{5})',
    ]
    
    for pattern in patterns:
        matches = re.findall(pattern, text, re.IGNORECASE)
        for match in matches:
            if isinstance(match, tuple):
                match = match[0] if match else ""
            if match and len(match) == 5:
                cpt_codes.add(match)
    
    return sorted(list(cpt_codes))


def extract_age_requirements(text: str) -> Optional[int]:
    """Extract age requirements from text."""
    patterns = [
        r'age\s*[>=]\s*(\d+)',
        r'(\d+)\s*years?\s*or\s*older',
        r'minimum\s*age[:\s]+(\d+)',
        r'at\s*least\s*(\d+)\s*years?\s*old',
        r'age\s*(\d+)\s*or\s*older',
    ]
    
    ages = []
    for pattern in patterns:
        matches = re.findall(pattern, text, re.IGNORECASE)
        for match in matches:
            try:
                age = int(match)
                if 0 <= age <= 120:
                    ages.append(age)
            except:
                pass
    
    return min(ages) if ages else None


def extract_prior_auth(text: str) -> Optional[bool]:
    """Determine if prior authorization is required from text."""
    text_lower = text.lower()
    
    pa_indicators = [
        'prior authorization required',
        'prior auth required',
        'requires prior authorization',
        'requires prior auth',
        'preauthorization required',
        'pa required',
    ]
    
    no_pa_indicators = [
        'no prior authorization',
        'prior authorization not required',
        'does not require prior authorization',
    ]
    
    for indicator in no_pa_indicators:
        if indicator in text_lower:
            return False
    
    for indicator in pa_indicators:
        if indicator in text_lower:
            return True
    
    return None


def extract_pos_codes_from_text(text: str) -> List[int]:
    """Extract place of service codes if explicitly mentioned in text."""
    pos_codes = []
    
    patterns = [
        r'POS\s*(\d{2})',
        r'place\s+of\s+service\s+code\s+(\d{2})',
        r'place\s+of\s+service\s+(\d{2})',
    ]
    
    for pattern in patterns:
        matches = re.findall(pattern, text, re.IGNORECASE)
        for match in matches:
            try:
                pos = int(match)
                if 11 <= pos <= 99:
                    if pos not in pos_codes:
                        pos_codes.append(pos)
            except:
                pass
    
    return sorted(pos_codes)


def extract_medical_necessity_sections(text: str) -> Tuple[List[str], List[str]]:
    """
    Extract ICD-10-CM codes that support vs. do not support medical necessity.
    Returns: (supporting_codes, not_supporting_codes)
    """
    supporting_patterns = [
        r'support[s]?[ing]?\s+medical\s+necessity',
        r'medical\s+necessity\s+support[s]?[ing]?',
        r'indicated\s+for',
        r'covered\s+for',
        r'appropriate\s+for',
    ]
    
    not_supporting_patterns = [
        r'does\s+not\s+support\s+medical\s+necessity',
        r'not\s+support[s]?[ing]?\s+medical\s+necessity',
        r'not\s+medically\s+necessary',
        r'not\s+covered\s+for',
    ]
    
    sentences = re.split(r'[.!?]\s+', text)
    
    supporting_codes = set()
    not_supporting_codes = set()
    
    for sentence in sentences:
        sentence_lower = sentence.lower()
        
        is_supporting = any(re.search(pattern, sentence_lower, re.IGNORECASE) for pattern in supporting_patterns)
        is_not_supporting = any(re.search(pattern, sentence_lower, re.IGNORECASE) for pattern in not_supporting_patterns)
        
        codes_in_sentence = extract_icd10_codes(sentence)
        
        if is_supporting:
            supporting_codes.update(codes_in_sentence)
        elif is_not_supporting:
            not_supporting_codes.update(codes_in_sentence)
        elif codes_in_sentence:
            # Default: assume supporting if no explicit context
            for code in codes_in_sentence:
                if code not in not_supporting_codes:
                    supporting_codes.add(code)
    
    return sorted(list(supporting_codes)), sorted(list(not_supporting_codes))


def fetch_medicare_article(article_id: int) -> Optional[Tuple[str, str]]:
    """
    Fetch HTML content from a Medicare Coverage Database article.
    Handles license agreement acceptance if needed.
    Returns: (html_content, url) if successful, None otherwise.
    """
    url = f"{CMS_MCD_BASE}/view/article.aspx?articleId={article_id}"
    
    try:
        session = requests.Session()
        
        # Initial request
        response = session.get(url, timeout=30, headers={
            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',
            'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8'
        })
        response.raise_for_status()
        
        soup = BeautifulSoup(response.text, 'html.parser')
        
        # Check if license agreement page is present
        # Look for common license agreement indicators
        license_indicators = [
            'license agreement',
            'ama license',
            'accept',
            'i agree',
            'terms and conditions'
        ]
        
        page_text_lower = response.text.lower()
        is_license_page = any(indicator in page_text_lower for indicator in license_indicators)
        
        if is_license_page:
            # Look for accept/agree buttons or forms
            # Common patterns: buttons with "Accept", "I Agree", "Continue", or forms with acceptance checkboxes
            accept_button = soup.find('button', string=re.compile(r'accept|agree|continue', re.I))
            accept_link = soup.find('a', string=re.compile(r'accept|agree|continue', re.I))
            accept_input = soup.find('input', {'type': 'submit', 'value': re.compile(r'accept|agree|continue', re.I)})
            
            # Also check for forms with __VIEWSTATE (ASP.NET forms)
            form = soup.find('form')
            if form:
                # Extract form data
                form_data = {}
                
                # Get all hidden inputs (like __VIEWSTATE)
                hidden_inputs = form.find_all('input', type='hidden')
                for input_field in hidden_inputs:
                    name = input_field.get('name', '')
                    value = input_field.get('value', '')
                    if name:
                        form_data[name] = value
                
                # Look for license acceptance checkbox or button
                checkbox = form.find('input', {'type': 'checkbox', 'name': re.compile(r'accept|agree|license', re.I)})
                if checkbox:
                    form_data[checkbox.get('name', '')] = 'on'
                
                # Try to submit the form to accept license
                form_action = form.get('action', '')
                if form_action:
                    if form_action.startswith('/'):
                        submit_url = f"{CMS_BASE_URL}{form_action}"
                    elif form_action.startswith('http'):
                        submit_url = form_action
                    else:
                        submit_url = url
                    
                    # Submit the form
                    response = session.post(submit_url, data=form_data, timeout=30, headers={
                        'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',
                        'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8',
                        'Referer': url
                    })
                    response.raise_for_status()
                    print(f"  ✓ License agreement accepted")
            
            elif accept_button or accept_link:
                # If there's an accept link/button, try clicking it
                if accept_link:
                    href = accept_link.get('href', '')
                    if href:
                        if href.startswith('/'):
                            accept_url = f"{CMS_BASE_URL}{href}"
                        elif href.startswith('http'):
                            accept_url = href
                        else:
                            accept_url = url
                        
                        response = session.get(accept_url, timeout=30, headers={
                            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',
                            'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8',
                            'Referer': url
                        })
                        response.raise_for_status()
                        print(f"  ✓ License agreement accepted")
            
            # After accepting, fetch the article again
            response = session.get(url, timeout=30, headers={
                'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',
                'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8'
            })
            response.raise_for_status()
        
        # Check if article is invalid
        if 'invalid article' in response.text.lower()[:2000] or len(response.text) < 5000:
            return None
        
        return (response.text, url)
    except Exception as e:
        print(f"    Error fetching article {article_id}: {e}")
        return None


def parse_medicare_article(html_content: str, source_url: str) -> Optional[Dict[str, Any]]:
    """Parse Medicare Coverage Database HTML article to extract CPT and ICD-10-CM information."""
    try:
        soup = BeautifulSoup(html_content, 'html.parser')
        text = soup.get_text(separator=' ', strip=True)
        
        # Extract CPT codes from the article
        cpt_codes = extract_cpt_codes(text)
        if not cpt_codes:
            return None  # No CPT codes found, skip this article
        
        # Extract ICD-10-CM codes
        all_icd_codes = extract_icd10_codes(text)
        if not all_icd_codes:
            return None  # No ICD codes found, skip
        
        # Identify which codes support vs don't support medical necessity
        supporting_codes, not_supporting_codes = extract_medical_necessity_sections(text)
        
        # If we couldn't classify, use all codes as supporting
        if not supporting_codes and all_icd_codes:
            supporting_codes = all_icd_codes
        
        # Extract other requirements
        age_gte = extract_age_requirements(text)
        requires_pa = extract_prior_auth(text)
        pos_allowed = extract_pos_codes_from_text(text)
        
        return {
            "cpt_codes": cpt_codes,
            "source_url": source_url,
            "icd10_in": supporting_codes if supporting_codes else all_icd_codes,
            "exclusion_icd10": not_supporting_codes if not_supporting_codes else [],
            "age_gte": age_gte,
            "requires_pa": requires_pa if requires_pa is not None else True,
            "pos_allowed": pos_allowed if pos_allowed else None,
        }
        
    except Exception as e:
        return None


def create_rule_json(cpt_code: str, parsed_data: Dict[str, Any]) -> Dict[str, Any]:
    """
    Create a rule JSON in the exact format specified.
    Format:
    {
      "policy_id": "<URL>",
      "version": "2025-10-01",
      "lob": "commercial",
      "codes": ["<CPT>"],
      "requires_pa": true/false,
      "inclusion": [{"gte": ["age_years", 18]}, {"lte": ["age_years", 80]}, {"in": ["primary_icd10", [integers]]}],
      "exclusion": [{"eq": ["pregnant", 1]}],
      "admin_rules": {"pos_allowed": [...], "max_units_per_day": 1}
    }
    """
    source_url = parsed_data.get("source_url", "")
    
    # Extract ICD-10 codes and map to integers
    inclusion_icd_codes = parsed_data.get("icd10_in", [])
    exclusion_icd_codes = parsed_data.get("exclusion_icd10", [])
    
    # Map ICD-10 codes to integers
    inclusion_icd_integers = [icd10_to_integer(code) for code in inclusion_icd_codes] if inclusion_icd_codes else []
    exclusion_icd_integers = [icd10_to_integer(code) for code in exclusion_icd_codes] if exclusion_icd_codes else []
    
    # Build inclusion array
    inclusion = []
    
    # Add age lower bound (gte)
    age_gte = parsed_data.get("age_gte")
    if age_gte:
        inclusion.append({"gte": ["age_years", age_gte]})
    else:
        inclusion.append({"gte": ["age_years", 18]})  # Default minimum age
    
    # Add age upper bound (lte)
    inclusion.append({"lte": ["age_years", 80]})  # Default maximum age
    
    # Add ICD-10 inclusion with integer array
    if inclusion_icd_integers:
        inclusion.append({"in": ["primary_icd10", inclusion_icd_integers]})
    
    # Build exclusion array
    exclusion = []
    
    # Add exclusion ICD-10 codes if any
    if exclusion_icd_integers:
        exclusion.append({"in": ["primary_icd10", exclusion_icd_integers]})
    
    # Build admin_rules
    admin_rules = {
        "max_units_per_day": 1
    }
    
    pos_allowed = parsed_data.get("pos_allowed")
    if pos_allowed:
        admin_rules["pos_allowed"] = pos_allowed
    else:
        admin_rules["pos_allowed"] = [22, 24]  # Default: outpatient hospital, ASC
    
    rule = {
        "policy_id": source_url,  # URL as policy_id
        "version": "2025-10-01",
        "lob": "commercial",
        "codes": [cpt_code],
        "requires_pa": parsed_data.get("requires_pa", True),
        "inclusion": inclusion,
        "exclusion": exclusion,
        "admin_rules": admin_rules
    }
    
    return rule


def write_rule_file(code: str, rule_json: Dict[str, Any]) -> str:
    """Write a rules.json file directly to rules/{code}.json."""
    os.makedirs(OUT_DIR, exist_ok=True)
    path = os.path.join(OUT_DIR, f"{code}.json")
    
    with open(path, "w") as f:
        json.dump(rule_json, f, indent=2, ensure_ascii=False)
    
    return path


def process_article(article_id: int) -> int:
    """Process a single Medicare article and generate rules for all CPT codes found."""
    print(f"\nProcessing article {article_id}...")
    
    # Fetch article
    result = fetch_medicare_article(article_id)
    if not result:
        print(f"  ✗ Article not accessible or invalid - skipping")
        return 0
    
    html_content, url = result
    print(f"  URL: {url}")
    
    # Parse article
    parsed_data = parse_medicare_article(html_content, url)
    if not parsed_data:
        print(f"  ✗ Could not extract CPT/ICD codes - skipping")
        return 0
    
    cpt_codes = parsed_data.get("cpt_codes", [])
    if not cpt_codes:
        print(f"  ✗ No CPT codes found - skipping")
        return 0
    
    print(f"  ✓ Found {len(cpt_codes)} CPT code(s): {', '.join(cpt_codes)}")
    print(f"  ✓ Found {len(parsed_data['icd10_in'])} ICD-10-CM codes supporting medical necessity")
    if parsed_data.get('exclusion_icd10'):
        print(f"  ✓ Found {len(parsed_data['exclusion_icd10'])} ICD-10-CM codes NOT supporting medical necessity")
    
    # Generate rules.json for each CPT code found
    generated_count = 0
    for cpt_code in cpt_codes:
        rule_json = create_rule_json(cpt_code, parsed_data)
        path = write_rule_file(cpt_code, rule_json)
        print(f"    [OK] Generated {path}")
        generated_count += 1
    
    return generated_count


def main():
    """Main function: process Medicare articles and generate rules.json files."""
    print("=" * 70)
    print("Medicare Coverage Database Article Processor")
    print("=" * 70)
    print(f"This will try articles until we successfully parse 50 articles")
    print("and generate rules.json files for each CPT code found")
    print("=" * 70)
    print()
    
    # Remove existing rules folder
    import shutil
    if os.path.exists(OUT_DIR):
        shutil.rmtree(OUT_DIR)
        print(f"Removed existing '{OUT_DIR}' folder\n")
    
    total_rules = 0
    successfully_parsed = 0
    target_count = 50
    
    for article_id in ARTICLE_IDS:
        if successfully_parsed >= target_count:
            print(f"\n✓ Successfully parsed {target_count} articles. Stopping.")
            break
        
        count = process_article(article_id)
        if count > 0:  # Successfully parsed and found CPT codes
            successfully_parsed += 1
            total_rules += count
        
        time.sleep(1)  # Rate limiting
    
    print("\n" + "=" * 70)
    print(f"[COMPLETE] Successfully parsed {successfully_parsed} articles")
    print(f"Generated {total_rules} rules.json files")
    print(f"Files are in the '{OUT_DIR}' folder")
    print("=" * 70)


if __name__ == "__main__":
    main()
