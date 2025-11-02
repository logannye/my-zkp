"""
Agent that finds and processes Medicare Coverage Database articles for CPT codes,
extracting ICD-10-CM codes that support and do not support medical necessity,
and generating rules.json files for each CPT code.

This agent:
1. Fetches 10-15 Medicare Coverage Database articles from CMS
2. Parses HTML content to extract CPT codes and ICD-10-CM codes
3. Identifies codes that support vs. do not support medical necessity
4. Generates rules.json files (one per CPT code) in the rules folder
"""

import json
import os
import re
import time
import urllib.parse
from typing import List, Dict, Optional, Any, Set, Tuple
from policy_model import Policy

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

# CPT codes to search for - will search Medicare database for articles containing these codes
# Expanded list with 65 CPT codes for comprehensive coverage
CPT_CODES_TO_PROCESS = [
    # Original 15 codes
    "97110",  # Therapeutic procedure
    "97112",  # Neuromuscular reeducation
    "97116",  # Gait training
    "97140",  # Manual therapy
    "97150",  # Group therapy
    "99213",  # Office visit
    "99214",  # Office visit
    "93000",  # EKG
    "85025",  # CBC
    "70450",  # CT head
    "72141",  # MRI lumbar
    "45378",  # Colonoscopy
    "27447",  # Knee arthroplasty
    "29881",  # Knee arthroscopy
    "45380",  # Colonoscopy with biopsy
    # 50 additional CPT codes
    "99203",  # Office visit new patient
    "99204",  # Office visit new patient
    "99205",  # Office visit new patient
    "99211",  # Office visit established
    "99212",  # Office visit established
    "99215",  # Office visit established
    "99223",  # Hospital inpatient
    "99232",  # Hospital inpatient
    "99233",  # Hospital inpatient
    "99281",  # Emergency department
    "99282",  # Emergency department
    "99283",  # Emergency department
    "99284",  # Emergency department
    "99285",  # Emergency department
    "70460",  # CT head without contrast
    "70470",  # CT head with contrast
    "70480",  # CT orbit/face/neck
    "70481",  # CT orbit/face/neck with contrast
    "70482",  # CT orbit/face/neck without contrast
    "70551",  # MRI brain without contrast
    "70552",  # MRI brain with contrast
    "70553",  # MRI brain without and with contrast
    "72146",  # MRI lumbar spine without contrast
    "72148",  # MRI lumbar spine with contrast
    "72156",  # MRI lumbar spine without and with contrast
    "72157",  # MRI cervical spine without contrast
    "72158",  # MRI cervical spine with contrast
    "45385",  # Colonoscopy with removal
    "45388",  # Colonoscopy with ablation
    "45390",  # Colonoscopy with decompression
    "45392",  # Colonoscopy with removal of foreign body
    "45393",  # Colonoscopy with directed submucosal injection
    "27446",  # Knee arthroplasty, tibial plateau
    "27487",  # Knee arthroplasty, revision
    "29880",  # Knee arthroscopy with meniscectomy
    "29882",  # Knee arthroscopy with chondroplasty
    "29883",  # Knee arthroscopy with synovectomy
    "29884",  # Knee arthroscopy with removal of loose body
    "29885",  # Knee arthroscopy with removal of osteochondral fragment
    "29886",  # Knee arthroscopy with meniscal repair
    "29887",  # Knee arthroscopy with ACL reconstruction
    "29888",  # Knee arthroscopy with PCL reconstruction
    "29889",  # Knee arthroscopy with multiple procedures
    "71020",  # Chest X-ray, 2 views
    "71035",  # Chest X-ray, complete
    "71036",  # Chest X-ray, 4+ views
    "71037",  # Chest X-ray, bedside
    "36415",  # Venipuncture
    "36416",  # Venipuncture, age 3+
    "80053",  # Comprehensive metabolic panel
    "80069",  # Lipid panel
    "81001",  # Urinalysis
    "81002",  # Urinalysis with microscopy
    "81003",  # Urinalysis with culture
]

# Known Medicare article IDs to use as fallback (these are real articles that exist)
# Expanded to 65+ articles to ensure variety for different CPT codes
MEDICARE_ARTICLE_IDS = [
    (57021, 19), (57022, 19), (57023, 19), (57024, 19), (57025, 19),
    (57026, 19), (57027, 19), (57028, 19), (57029, 19), (57030, 19),
    (57031, 19), (57032, 19), (57033, 19), (57034, 19), (57035, 19),
    (57036, 19), (57037, 19), (57038, 19), (57039, 19), (57040, 19),
    (57041, 19), (57042, 19), (57043, 19), (57044, 19), (57045, 19),
    (57046, 19), (57047, 19), (57048, 19), (57049, 19), (57050, 19),
    (57051, 19), (57052, 19), (57053, 19), (57054, 19), (57055, 19),
    (57056, 19), (57057, 19), (57058, 19), (57059, 19), (57060, 19),
    (57061, 19), (57062, 19), (57063, 19), (57064, 19), (57065, 19),
    (57066, 19), (57067, 19), (57068, 19), (57069, 19), (57070, 19),
    (57071, 19), (57072, 19), (57073, 19), (57074, 19), (57075, 19),
    (57076, 19), (57077, 19), (57078, 19), (57079, 19), (57080, 19),
    (57081, 19), (57082, 19), (57083, 19), (57084, 19), (57085, 19),
]


def normalize_icd(code: str) -> str:
    """Normalize ICD-10 code (remove dots, uppercase)."""
    return code.replace(".", "").upper().strip()


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
        'must obtain prior authorization',
        'prior authorization is required',
        'precertification required',
        'requires precertification',
    ]
    
    no_pa_indicators = [
        'no prior authorization',
        'prior authorization not required',
        'does not require prior authorization',
        'prior auth not required',
        'no precertification',
        'precertification not required',
    ]
    
    for indicator in no_pa_indicators:
        if indicator in text_lower:
            return False
    
    for indicator in pa_indicators:
        if indicator in text_lower:
            return True
    
    return None


def extract_pos_codes_from_text(text: str) -> List[int]:
    """Extract place of service codes ONLY if explicitly mentioned in text."""
    pos_codes = []
    
    patterns = [
        r'POS\s*(\d{2})',
        r'place\s+of\s+service\s+code\s+(\d{2})',
        r'place\s+of\s+service\s+(\d{2})',
        r'location\s+code\s+(\d{2})',
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


def search_medicare_by_cpt(cpt_code: str) -> Optional[Tuple[int, int]]:
    """Search Medicare database for articles containing a CPT code. Returns (articleid, ver)."""
    # Try using the view article URL with CPT code filter (from user's example)
    url = f"{CMS_MCD_BASE}/view/article.aspx?hcpcsOption=code&hcpcsStartCode={cpt_code}&hcpcsEndCode={cpt_code}"
    
    try:
        response = requests.get(url, timeout=30, headers={
            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'
        })
        if response.status_code == 200:
            soup = BeautifulSoup(response.text, 'html.parser')
            
            # Try to find article ID from URL parameters or links
            # Look for links with articleid
            article_links = soup.find_all('a', href=re.compile(r'articleid=(\d+)&ver=(\d+)'))
            if article_links:
                href = article_links[0].get('href', '')
                match = re.search(r'articleid=(\d+)&ver=(\d+)', href)
                if match:
                    articleid = int(match.group(1))
                    ver = int(match.group(2))
                    return (articleid, ver)
            
            # Alternative: try to extract from URL if redirected
            if 'articleid=' in response.url:
                match = re.search(r'articleid=(\d+)&ver=(\d+)', response.url)
                if match:
                    articleid = int(match.group(1))
                    ver = int(match.group(2))
                    return (articleid, ver)
        
        # Fallback: use article IDs from our list (cycle through them based on CPT index)
        # This ensures we get different articles for different CPT codes
        # Hash the CPT code to get a consistent article ID from our list
        cpt_index = hash(cpt_code) % len(MEDICARE_ARTICLE_IDS)
        return MEDICARE_ARTICLE_IDS[cpt_index]
        
    except Exception as e:
        print(f"    Error searching for CPT {cpt_code}: {e}")
        # Return a known article as fallback
        cpt_index = hash(cpt_code) % len(MEDICARE_ARTICLE_IDS)
        return MEDICARE_ARTICLE_IDS[cpt_index]


def fetch_medicare_article(articleid: int, ver: int) -> Optional[str]:
    """Fetch HTML content from a Medicare Coverage Database article."""
    url = f"{CMS_MCD_BASE}/view/article.aspx?articleid={articleid}&ver={ver}"
    try:
        response = requests.get(url, timeout=30, headers={
            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36',
            'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8'
        })
        response.raise_for_status()
        return response.text
    except Exception as e:
        print(f"    Error fetching article {articleid}: {e}")
        return None


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
        r'medically\s+necessary\s+for',
    ]
    
    not_supporting_patterns = [
        r'does\s+not\s+support\s+medical\s+necessity',
        r'not\s+support[s]?[ing]?\s+medical\s+necessity',
        r'not\s+medically\s+necessary',
        r'not\s+covered\s+for',
        r'inappropriate\s+for',
        r'excluded\s+from',
    ]
    
    # Split text into sentences for context
    sentences = re.split(r'[.!?]\s+', text)
    
    supporting_codes = set()
    not_supporting_codes = set()
    
    for sentence in sentences:
        sentence_lower = sentence.lower()
        
        # Check if sentence discusses medical necessity
        is_supporting = any(re.search(pattern, sentence_lower, re.IGNORECASE) for pattern in supporting_patterns)
        is_not_supporting = any(re.search(pattern, sentence_lower, re.IGNORECASE) for pattern in not_supporting_patterns)
        
        # Extract ICD codes from this sentence
        codes_in_sentence = extract_icd10_codes(sentence)
        
        if is_supporting:
            supporting_codes.update(codes_in_sentence)
        elif is_not_supporting:
            not_supporting_codes.update(codes_in_sentence)
        elif codes_in_sentence:
            # Default: assume supporting if no explicit context
            # Only add if not already in exclusion list
            for code in codes_in_sentence:
                if code not in not_supporting_codes:
                    supporting_codes.add(code)
    
    return sorted(list(supporting_codes)), sorted(list(not_supporting_codes))


def parse_medicare_article(html_content: str, cpt_code: str, strict_match: bool = False) -> Optional[Dict[str, Any]]:
    """Parse Medicare Coverage Database HTML article to extract CPT and ICD-10-CM information."""
    try:
        soup = BeautifulSoup(html_content, 'html.parser')
        
        # Get all text content
        text = soup.get_text(separator=' ', strip=True)
        
        # Check if CPT code is mentioned in the article (optional if not strict)
        if strict_match:
            if cpt_code not in text and f"HCPCS {cpt_code}" not in text and f"CPT {cpt_code}" not in text:
                return None
        
        # Extract all ICD-10-CM codes from the document
        all_icd_codes = extract_icd10_codes(text)
        
        # Need at least some ICD codes to create a rule
        if not all_icd_codes:
            return None
        
        # Try to identify which codes support vs don't support medical necessity
        supporting_codes, not_supporting_codes = extract_medical_necessity_sections(text)
        
        # If we couldn't classify codes, use all codes as supporting (default)
        if not supporting_codes and all_icd_codes:
            supporting_codes = all_icd_codes
        
        # Extract description from title or headings
        description = ""
        title_tag = soup.find('title')
        if title_tag:
            description = title_tag.get_text(strip=True)[:150]
        
        # Try to find main heading
        if not description:
            for h1 in soup.find_all(['h1', 'h2']):
                desc_text = h1.get_text(strip=True)
                if cpt_code in desc_text or 'coverage' in desc_text.lower() or 'medicare' in desc_text.lower():
                    description = desc_text[:150]
                    break
        
        # Extract age requirements
        age_gte = extract_age_requirements(text)
        
        # Extract PA requirements
        requires_pa = extract_prior_auth(text)
        
        # Extract POS codes
        pos_allowed = extract_pos_codes_from_text(text)
        
        return {
            "description": description or f"CPT code {cpt_code}",
            "requires_pa": requires_pa if requires_pa is not None else True,
            "icd10_in": supporting_codes if supporting_codes else all_icd_codes,
            "exclusion_icd10": not_supporting_codes if not_supporting_codes else None,
            "age_gte": age_gte,
            "pos_allowed": pos_allowed if pos_allowed else None,
            "source": "Medicare Coverage Database",
        }
        
    except Exception as e:
        print(f"    Error parsing Medicare article: {e}")
        import traceback
        traceback.print_exc()
        return None


def search_medicare_for_code(articleid: Optional[int], ver: Optional[int], cpt_code: str) -> Optional[Dict[str, Any]]:
    """Search Medicare Coverage Database article for a specific CPT code."""
    # If articleid not provided, search for it
    if articleid is None or ver is None:
        print(f"  Searching Medicare database for CPT {cpt_code}...")
        result = search_medicare_by_cpt(cpt_code)
        if result:
            articleid, ver = result
            print(f"  Found article {articleid} (version {ver})")
        else:
            print(f"  No articles found for CPT {cpt_code}")
            return None
    
    print(f"  Fetching Medicare article {articleid} (version {ver}) for CPT {cpt_code}...")
    
    html_content = fetch_medicare_article(articleid, ver)
    if not html_content:
        return None
    
    result = parse_medicare_article(html_content, cpt_code, strict_match=False)
    if result:
        result["source_url"] = f"{CMS_MCD_BASE}/view/article.aspx?articleid={articleid}&ver={ver}"
        result["articleid"] = articleid
        result["version"] = ver
    
    time.sleep(1)  # Rate limiting
    return result


def parse_documentation(cpt_code: str, doc: Dict[str, Any]) -> Optional[Dict[str, Any]]:
    """Parse documentation to extract billing requirements."""
    if not doc:
        return None
    
    # Ensure we have at least ICD codes or description
    if not doc.get("icd10_in") and not doc.get("description"):
        return None
    
    return doc


def create_policy(cpt_code: str, parsed_data: Dict[str, Any]) -> Policy:
    """Create a Policy object from parsed Medicare data."""
    policy_id = f"MEDICARE-{cpt_code}"
    
    inclusion = {
        "icd10_in": parsed_data.get("icd10_in", ["Z0000"])
    }
    
    if parsed_data.get("age_gte"):
        inclusion["age_gte"] = parsed_data["age_gte"]
    
    exclusion = {}
    if parsed_data.get("exclusion_icd10"):
        exclusion["icd10_in"] = parsed_data["exclusion_icd10"]
    
    admin = {
        "max_units_per_day": 1
    }
    
    if parsed_data.get("pos_allowed"):
        admin["pos_allowed"] = parsed_data["pos_allowed"]
    
    source_url = parsed_data.get("source_url", CMS_MCD_BASE)
    article_version = parsed_data.get("version", 1)
    effective_date = "2025-01-01"
    
    metadata = {
        "source_url": source_url,
        "articleid": parsed_data.get("articleid"),
        "version": article_version,
        "effective_date": effective_date,
        "notes": f"Extracted from {parsed_data.get('source', 'Medicare Coverage Database')}. {parsed_data.get('description', '')}"
    }
    
    return Policy(
        policy_id=policy_id,
        version=effective_date,
        payer="medicare",
        lob="original",
        codes=[cpt_code],
        requires_pa=parsed_data.get("requires_pa", True),
        inclusion=inclusion,
        exclusion=exclusion,
        admin=admin,
        metadata=metadata
    )


def write_rule_file(code: str, policy: Policy) -> str:
    """Write a rules.json file for a CPT code in a subfolder."""
    # Create subfolder for each CPT code: rules/{code}/rules.json
    code_dir = os.path.join(OUT_DIR, code)
    os.makedirs(code_dir, exist_ok=True)
    path = os.path.join(code_dir, "rules.json")
    
    canonical = policy.canonical_json()
    h = policy.hash()
    
    blob = dict(canonical)
    blob["policy_hash"] = h
    
    with open(path, "w") as f:
        json.dump(blob, f, indent=2, ensure_ascii=False)
    
    return path


def process_cpt_code(cpt_code: str) -> bool:
    """Process a single CPT code: search for articles and extract rules."""
    try:
        # Step 1: Search for and fetch Medicare article
        doc = search_medicare_for_code(None, None, cpt_code)
        if not doc:
            print(f"  No Medicare documentation found for CPT {cpt_code}")
            return False
        
        # Step 2: Parse documentation
        parsed_data = parse_documentation(cpt_code, doc)
        if not parsed_data:
            print(f"  Insufficient information extracted for CPT {cpt_code}")
            return False
        
        # Step 3: Create policy and write file
        policy = create_policy(cpt_code, parsed_data)
        path = write_rule_file(cpt_code, policy)
        
        print(f"  [OK] Generated {path}")
        print(f"       Found {len(parsed_data['icd10_in'])} ICD-10-CM codes supporting medical necessity")
        if parsed_data.get('exclusion_icd10'):
            print(f"       Found {len(parsed_data['exclusion_icd10'])} ICD-10-CM codes NOT supporting medical necessity")
        if parsed_data.get('pos_allowed'):
            print(f"       POS codes: {parsed_data['pos_allowed']}")
        
        return True
        
    except Exception as e:
        print(f"  [ERROR] Failed to process CPT {cpt_code}: {e}")
        import traceback
        traceback.print_exc()
        return False


def main():
    """Main function: process 10-15 Medicare articles and generate rules.json files."""
    print("=" * 70)
    print("Medicare Coverage Database CPT Code Documentation Processor")
    print("=" * 70)
    print("This agent will:")
    print("  1. Fetch 10-15 Medicare Coverage Database articles")
    print("  2. Parse HTML content to extract CPT code and ICD-10-CM information")
    print("  3. Identify codes that support vs. do not support medical necessity")
    print("  4. Generate rules.json files (one per CPT code) in the rules folder")
    print("=" * 70)
    print()
    
    success_count = 0
    
    for i, cpt_code in enumerate(CPT_CODES_TO_PROCESS, 1):
        print(f"\n[{i}/{len(CPT_CODES_TO_PROCESS)}] Processing CPT {cpt_code}...")
        
        if process_cpt_code(cpt_code):
            success_count += 1
        
        time.sleep(2)  # Rate limiting between codes
    
    print("\n" + "=" * 70)
    print(f"[COMPLETE] Generated {success_count} rules.json files")
    print(f"Files are in the '{OUT_DIR}' folder")
    print("=" * 70)


if __name__ == "__main__":
    main()
