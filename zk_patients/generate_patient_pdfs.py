"""
Generate detailed patient PDF files with realistic medical records.
Creates 10 mock patient PDFs with extensive clinical data, notes, lab values,
and other information. The LLM agent will extract only relevant structured data.
"""

from reportlab.lib.pagesizes import letter
from reportlab.pdfgen import canvas
from reportlab.lib.units import inch
import os
from datetime import datetime, timedelta
import random

# Sample patient data with detailed clinical information
PATIENT_DATA = [
    {
        "patient_id": "PAT001",
        "name": "Jane Smith",
        "dob": "1970-04-02",
        "sex": "F",
        "icd10_list": ["C50.912", "E11.9"],
        "pregnant": False,
        "place_of_service": 22,
        "units": 1,
        "clinical_notes": [
            "Patient presents with right breast lump. Mammography shows suspicious mass in upper outer quadrant.",
            "Previous mammogram from 6 months ago showed no abnormalities.",
            "Family history: Mother had breast cancer at age 65.",
            "Patient reports occasional right breast tenderness, denies discharge."
        ],
        "lab_values": [
            ("CBC", "WBC: 6.5", "RBC: 4.2", "Hgb: 12.8", "Platelets: 250"),
            ("Chem-7", "Glucose: 145", "BUN: 18", "Creatinine: 0.9", "eGFR: 78"),
            ("Tumor Markers", "CA 15-3: 28.5", "CA 27.29: 32.1"),
            ("Hormones", "Estradiol: 85", "FSH: 42", "LH: 18")
        ],
        "medications": ["Metformin 500mg BID", "Aspirin 81mg daily", "Vitamin D 1000 IU"],
        "vital_signs": {"BP": "128/78", "HR": "72", "Temp": "98.6", "RR": "16", "O2Sat": "98%"},
        "allergies": "Penicillin (rash)",
        "insurance": "Medicare Original",
    },
    {
        "patient_id": "PAT002",
        "name": "John Doe",
        "dob": "1985-03-15",
        "sex": "M",
        "icd10_list": ["J18.9", "E11.65"],
        "pregnant": False,
        "place_of_service": 22,
        "units": 1,
        "clinical_notes": [
            "Patient presents with 5-day history of productive cough, fever, and shortness of breath.",
            "Chest X-ray shows consolidation in right lower lobe consistent with pneumonia.",
            "Oxygen saturation improved from 92% to 96% with supplemental oxygen.",
            "Patient has type 2 diabetes, HbA1c last checked at 7.2%.",
            "No recent travel or sick contacts. Works as office manager."
        ],
        "lab_values": [
            ("CBC", "WBC: 12.3", "RBC: 4.8", "Hgb: 14.2", "Platelets: 320"),
            ("CMP", "Glucose: 165", "Sodium: 138", "Potassium: 4.2", "Creatinine: 1.1"),
            ("Blood Gas", "pH: 7.38", "pCO2: 42", "pO2: 88", "HCO3: 24"),
            ("Inflammatory", "CRP: 45.2", "ESR: 32", "Procalcitonin: 0.8")
        ],
        "medications": ["Levofloxacin 750mg daily", "Glipizide 5mg BID", "Lisinopril 10mg daily"],
        "vital_signs": {"BP": "135/85", "HR": "88", "Temp": "100.2", "RR": "20", "O2Sat": "96%"},
        "allergies": "Sulfa drugs (GI upset)",
        "insurance": "Medicare Original",
    },
    {
        "patient_id": "PAT003",
        "name": "Mary Johnson",
        "dob": "1995-06-20",
        "sex": "F",
        "icd10_list": ["C50.912"],
        "pregnant": False,
        "place_of_service": 11,
        "units": 1,
        "clinical_notes": [
            "29-year-old female with abnormal screening mammogram.",
            "BI-RADS 4B classification. Recommended diagnostic mammogram and ultrasound.",
            "Patient has no family history of breast cancer.",
            "Regular menses, no hormone replacement therapy.",
            "G2P1, last delivery 3 years ago, breastfed for 6 months."
        ],
        "lab_values": [
            ("CBC", "WBC: 7.2", "RBC: 4.5", "Hgb: 13.5", "Platelets: 280"),
            ("Chem-10", "Glucose: 92", "ALT: 18", "AST: 22", "Bilirubin: 0.6"),
            ("Lipid Panel", "Total Chol: 185", "LDL: 110", "HDL: 65", "Triglycerides: 150")
        ],
        "medications": ["Oral contraceptive", "Calcium supplement"],
        "vital_signs": {"BP": "118/72", "HR": "68", "Temp": "98.4", "RR": "14", "O2Sat": "99%"},
        "allergies": "NKDA",
        "insurance": "Medicare Original",
    },
    {
        "patient_id": "PAT004",
        "name": "Robert Williams",
        "dob": "1965-11-30",
        "sex": "M",
        "icd10_list": ["G43.909", "M54.5"],
        "pregnant": False,
        "place_of_service": 22,
        "units": 1,
        "clinical_notes": [
            "59-year-old male with chronic migraine headaches, worsening over past 3 months.",
            "Headaches are unilateral, pulsating, associated with photophobia and nausea.",
            "Patient reports headaches 3-4 times per week, lasting 4-6 hours.",
            "Also complains of lower back pain, worse in morning.",
            "MRI brain ordered to rule out structural abnormalities.",
            "Previous treatments: Sumatriptan PRN, Topiramate 50mg BID."
        ],
        "lab_values": [
            ("CBC", "WBC: 8.1", "RBC: 5.2", "Hgb: 15.8", "Platelets: 245"),
            ("CMP", "Glucose: 98", "Sodium: 140", "Potassium: 4.5", "Creatinine: 1.0"),
            ("TSH", "TSH: 2.1", "Free T4: 1.2"),
            ("B12/Folate", "B12: 450", "Folate: 12.5")
        ],
        "medications": ["Sumatriptan 100mg PRN", "Topiramate 50mg BID", "Ibuprofen 600mg PRN", "Fish Oil"],
        "vital_signs": {"BP": "142/88", "HR": "76", "Temp": "98.5", "RR": "16", "O2Sat": "97%"},
        "allergies": "Codeine (nausea, vomiting)",
        "insurance": "Medicare Original",
    },
    {
        "patient_id": "PAT005",
        "name": "Sarah Davis",
        "dob": "1992-08-14",
        "sex": "F",
        "icd10_list": ["C50.912", "E11.9"],
        "pregnant": True,
        "place_of_service": 22,
        "units": 1,
        "clinical_notes": [
            "32-year-old pregnant female (28 weeks gestation) with suspicious breast mass.",
            "Ultrasound-guided core biopsy recommended. Modified approach due to pregnancy.",
            "Patient also has gestational diabetes, well-controlled with diet.",
            "Obstetric history: G1P0, singleton pregnancy, due date 12 weeks from now.",
            "Patient is anxious about procedure during pregnancy but understands necessity.",
            "OB/GYN consulted and approved biopsy with modified technique."
        ],
        "lab_values": [
            ("CBC", "WBC: 9.5", "RBC: 3.8", "Hgb: 11.2", "Platelets: 290"),
            ("Chem-7", "Glucose: 140", "BUN: 12", "Creatinine: 0.7", "eGFR: 95"),
            ("Prenatal Labs", "HgbA1c: 5.8", "Glucose Challenge: 145"),
            ("Hormones", "HCG: Positive", "Estradiol: 8500", "Progesterone: 45")
        ],
        "medications": ["Prenatal vitamins", "Folic acid 1mg", "Glyburide 2.5mg daily"],
        "vital_signs": {"BP": "125/75", "HR": "82", "Temp": "98.7", "RR": "18", "O2Sat": "98%"},
        "allergies": "NKDA",
        "insurance": "Medicare Original",
    },
    {
        "patient_id": "PAT006",
        "name": "Michael Brown",
        "dob": "1978-02-28",
        "sex": "M",
        "icd10_list": ["C34.90", "Z87.891"],
        "pregnant": False,
        "place_of_service": 23,
        "units": 1,
        "clinical_notes": [
            "46-year-old male with lung mass found on routine chest CT.",
            "Patient has 40-pack-year smoking history, quit 2 years ago.",
            "CT chest shows 2.5cm nodule in right upper lobe with spiculated margins.",
            "PET scan shows increased FDG uptake, concerning for malignancy.",
            "Patient reports 15-pound weight loss over past 6 months.",
            "Family history: Father died of lung cancer at age 72.",
            "CT-guided biopsy scheduled for tissue diagnosis."
        ],
        "lab_values": [
            ("CBC", "WBC: 10.5", "RBC: 4.9", "Hgb: 14.5", "Platelets: 380"),
            ("CMP", "Glucose: 102", "Sodium: 139", "Potassium: 4.3", "Creatinine: 1.2"),
            ("Tumor Markers", "CEA: 8.5", "CYFRA 21-1: 3.2"),
            ("Coagulation", "PT: 12.5", "INR: 1.1", "PTT: 28")
        ],
        "medications": ["Atorvastatin 20mg daily", "Omeprazole 20mg daily", "Multivitamin"],
        "vital_signs": {"BP": "138/86", "HR": "92", "Temp": "99.1", "RR": "22", "O2Sat": "94%"},
        "allergies": "Aspirin (GI bleeding)",
        "insurance": "Medicare Original",
    },
    {
        "patient_id": "PAT007",
        "name": "Emily Wilson",
        "dob": "1988-09-10",
        "sex": "F",
        "icd10_list": ["K50.90", "E03.9"],
        "pregnant": False,
        "place_of_service": 19,
        "units": 2,
        "clinical_notes": [
            "36-year-old female with Crohn's disease, flaring for past 2 weeks.",
            "Patient reports 6-8 loose stools per day, abdominal cramping, and fatigue.",
            "Colonoscopy shows active inflammation in terminal ileum and ascending colon.",
            "Previous treatment with Mesalamine ineffective, now on Infliximab.",
            "Patient also has hypothyroidism, on levothyroxine replacement.",
            "Routine infusion scheduled. Patient tolerates infusions well.",
            "Last infusion was 8 weeks ago, symptoms returning."
        ],
        "lab_values": [
            ("CBC", "WBC: 11.2", "RBC: 4.0", "Hgb: 11.8", "Platelets: 420"),
            ("CMP", "Glucose: 88", "Albumin: 3.2", "Total Protein: 6.8", "Creatinine: 0.8"),
            ("Inflammatory", "CRP: 28.5", "ESR: 45", "Calprotectin: 450"),
            ("Thyroid", "TSH: 4.5", "Free T4: 0.9", "Free T3: 2.8"),
            ("Nutritional", "Iron: 45", "Ferritin: 25", "B12: 380", "Folate: 8.5")
        ],
        "medications": ["Infliximab 10mg/kg IV q8weeks", "Levothyroxine 75mcg daily", "Iron supplement"],
        "vital_signs": {"BP": "112/68", "HR": "78", "Temp": "98.8", "RR": "16", "O2Sat": "98%"},
        "allergies": "Sulfasalazine (rash, hives)",
        "insurance": "Medicare Original",
    },
    {
        "patient_id": "PAT008",
        "name": "David Miller",
        "dob": "1955-12-05",
        "sex": "M",
        "icd10_list": ["S06.0", "F41.9"],
        "pregnant": False,
        "place_of_service": 22,
        "units": 1,
        "clinical_notes": [
            "69-year-old male with closed head injury from fall 2 days ago.",
            "Patient lost consciousness for approximately 30 seconds.",
            "CT head shows small subdural hematoma, non-expanding.",
            "Neurological exam: Alert, oriented x3, mild confusion at times.",
            "Patient also reports anxiety and sleep disturbance since injury.",
            "GCS on admission was 14, currently 15. No focal deficits.",
            "Follow-up MRI scheduled in 1 week to monitor hematoma."
        ],
        "lab_values": [
            ("CBC", "WBC: 8.5", "RBC: 4.6", "Hgb: 14.0", "Platelets: 265"),
            ("CMP", "Glucose: 108", "Sodium: 138", "Potassium: 4.1", "Creatinine: 1.3"),
            ("Coagulation", "PT: 13.2", "INR: 1.2", "PTT: 30"),
            ("Liver Function", "ALT: 28", "AST: 32", "Alk Phos: 85", "Bilirubin: 0.8")
        ],
        "medications": ["Warfarin 5mg daily", "Atenolol 50mg daily", "Sertraline 50mg daily", "Melatonin 3mg"],
        "vital_signs": {"BP": "145/90", "HR": "68", "Temp": "98.6", "RR": "18", "O2Sat": "97%"},
        "allergies": "Penicillin (anaphylaxis)",
        "insurance": "Medicare Original",
    },
    {
        "patient_id": "PAT009",
        "name": "Lisa Anderson",
        "dob": "1972-07-22",
        "sex": "F",
        "icd10_list": ["M05.9", "M79.3"],
        "pregnant": False,
        "place_of_service": 22,
        "units": 1,
        "clinical_notes": [
            "52-year-old female with rheumatoid arthritis, active joint inflammation.",
            "Patient reports morning stiffness lasting 2-3 hours, affecting hands and wrists.",
            "Physical exam: Swollen MCP and PIP joints bilaterally, warmth and tenderness.",
            "Rheumatoid factor positive, CCP antibodies positive.",
            "Previous DMARDs: Methotrexate caused nausea, now on Etanercept.",
            "Patient also complains of myalgia and fibromyalgia-like symptoms.",
            "X-rays show erosive changes in multiple joints. MRI wrist shows active synovitis."
        ],
        "lab_values": [
            ("CBC", "WBC: 7.8", "RBC: 4.3", "Hgb: 12.5", "Platelets: 310"),
            ("CMP", "Glucose: 95", "ALT: 22", "AST: 25", "Creatinine: 0.9"),
            ("Inflammatory", "CRP: 18.5", "ESR: 32", "RF: 85", "Anti-CCP: 125"),
            ("Autoimmune", "ANA: Positive 1:160", "Anti-dsDNA: Negative")
        ],
        "medications": ["Etanercept 50mg SQ weekly", "Folic acid 1mg daily", "Naproxen 500mg BID", "Gabapentin 300mg TID"],
        "vital_signs": {"BP": "130/82", "HR": "74", "Temp": "98.5", "RR": "16", "O2Sat": "98%"},
        "allergies": "Sulfa drugs (Stevens-Johnson syndrome)",
        "insurance": "Medicare Original",
    },
    {
        "patient_id": "PAT010",
        "name": "James Taylor",
        "dob": "1990-01-18",
        "sex": "M",
        "icd10_list": ["C61", "N40.1"],
        "pregnant": False,
        "place_of_service": 11,
        "units": 1,
        "clinical_notes": [
            "35-year-old male with elevated PSA (8.5) and abnormal digital rectal exam.",
            "Prostate biopsy shows Gleason 3+3 = 6, involving 2 of 12 cores.",
            "MRI prostate shows PI-RADS 4 lesion in peripheral zone.",
            "Patient has BPH with moderate LUTS symptoms.",
            "Family history: Paternal uncle had prostate cancer at age 68.",
            "Genetic testing shows no BRCA mutations.",
            "Active surveillance vs. treatment options discussed."
        ],
        "lab_values": [
            ("CBC", "WBC: 7.2", "RBC: 5.1", "Hgb: 15.2", "Platelets: 295"),
            ("CMP", "Glucose: 92", "Sodium: 141", "Potassium: 4.4", "Creatinine: 1.0"),
            ("Prostate", "PSA: 8.5", "Free PSA: 1.2", "%Free PSA: 14.1"),
            ("Testosterone", "Total Testosterone: 485", "Free Testosterone: 12.5")
        ],
        "medications": ["Tamsulosin 0.4mg daily", "Finasteride 5mg daily", "Multivitamin"],
        "vital_signs": {"BP": "128/80", "HR": "70", "Temp": "98.4", "RR": "14", "O2Sat": "99%"},
        "allergies": "NKDA",
        "insurance": "Medicare Original",
    }
]

def create_patient_pdf(patient_data, output_path):
    """Create a detailed PDF file with patient medical record data."""
    c = canvas.Canvas(output_path, pagesize=letter)
    width, height = letter
    
    # Title
    c.setFont("Helvetica-Bold", 18)
    c.drawString(1*inch, height - 0.75*inch, "MEDICAL RECORD")
    
    # Date
    c.setFont("Helvetica", 10)
    c.drawString(width - 2*inch, height - 0.75*inch, f"Date: {datetime.now().strftime('%Y-%m-%d')}")
    
    # Patient Information Section
    y = height - 1.25*inch
    c.setFont("Helvetica-Bold", 14)
    c.drawString(1*inch, y, "PATIENT INFORMATION")
    
    y -= 0.35*inch
    c.setFont("Helvetica", 10)
    
    # Patient demographics in columns
    left_col = 1*inch
    right_col = 4*inch
    
    fields_left = [
        ("Patient ID:", patient_data["patient_id"]),
        ("Name:", patient_data["name"]),
        ("Date of Birth:", patient_data["dob"]),
        ("Sex:", patient_data["sex"]),
    ]
    
    fields_right = [
        ("Insurance:", patient_data.get("insurance", "Medicare Original")),
        ("Place of Service:", f"{patient_data['place_of_service']}"),
        ("Units:", str(patient_data["units"])),
        ("Allergies:", patient_data.get("allergies", "NKDA")),
    ]
    
    for i, (label, value) in enumerate(fields_left):
        c.drawString(left_col, y, f"{label:15} {value}")
        y -= 0.25*inch
    
    y = height - 1.25*inch - 0.35*inch
    for i, (label, value) in enumerate(fields_right):
        c.drawString(right_col, y, f"{label:15} {value}")
        y -= 0.25*inch
    
    # Vital Signs
    y = height - 3*inch
    c.setFont("Helvetica-Bold", 12)
    c.drawString(1*inch, y, "VITAL SIGNS")
    y -= 0.3*inch
    c.setFont("Helvetica", 10)
    
    if "vital_signs" in patient_data:
        vs = patient_data["vital_signs"]
        vital_text = f"BP: {vs.get('BP', 'N/A')}  HR: {vs.get('HR', 'N/A')} bpm  Temp: {vs.get('Temp', 'N/A')}°F  RR: {vs.get('RR', 'N/A')}  O2 Sat: {vs.get('O2Sat', 'N/A')}"
        c.drawString(1*inch, y, vital_text)
        y -= 0.3*inch
    
    # Diagnosis Codes Section
    c.setFont("Helvetica-Bold", 12)
    c.drawString(1*inch, y, "DIAGNOSIS CODES (ICD-10-CM)")
    y -= 0.3*inch
    c.setFont("Helvetica", 10)
    
    for icd in patient_data["icd10_list"]:
        c.drawString(1.2*inch, y, f"• {icd}")
        y -= 0.2*inch
    
    # Clinical Notes Section
    y -= 0.3*inch
    c.setFont("Helvetica-Bold", 12)
    c.drawString(1*inch, y, "CLINICAL NOTES")
    y -= 0.3*inch
    c.setFont("Helvetica", 9)
    
    if "clinical_notes" in patient_data:
        for i, note in enumerate(patient_data["clinical_notes"], 1):
            # Wrap long text
            words = note.split()
            lines = []
            current_line = ""
            for word in words:
                if len(current_line + word) < 80:
                    current_line += word + " "
                else:
                    lines.append(current_line.strip())
                    current_line = word + " "
            if current_line:
                lines.append(current_line.strip())
            
            for line in lines:
                if y < 2*inch:  # New page if needed
                    c.showPage()
                    y = height - 1*inch
                    c.setFont("Helvetica", 9)
                c.drawString(1.2*inch, y, line)
                y -= 0.18*inch
    
    # Lab Values Section
    y -= 0.3*inch
    if y < 3*inch:  # New page if needed
        c.showPage()
        y = height - 1*inch
    
    c.setFont("Helvetica-Bold", 12)
    c.drawString(1*inch, y, "LABORATORY VALUES")
    y -= 0.3*inch
    c.setFont("Helvetica", 9)
    
    if "lab_values" in patient_data:
        for lab_group in patient_data["lab_values"]:
            if y < 1.5*inch:  # New page if needed
                c.showPage()
                y = height - 1*inch
                c.setFont("Helvetica", 9)
            
            group_name = lab_group[0]
            values = " | ".join(lab_group[1:])
            c.setFont("Helvetica-Bold", 10)
            c.drawString(1.2*inch, y, f"{group_name}:")
            y -= 0.22*inch
            c.setFont("Helvetica", 9)
            c.drawString(1.4*inch, y, values)
            y -= 0.25*inch
    
    # Medications Section
    y -= 0.3*inch
    if y < 2*inch:  # New page if needed
        c.showPage()
        y = height - 1*inch
    
    c.setFont("Helvetica-Bold", 12)
    c.drawString(1*inch, y, "CURRENT MEDICATIONS")
    y -= 0.3*inch
    c.setFont("Helvetica", 10)
    
    if "medications" in patient_data:
        for med in patient_data["medications"]:
            if y < 1.5*inch:  # New page if needed
                c.showPage()
                y = height - 1*inch
                c.setFont("Helvetica", 10)
            c.drawString(1.2*inch, y, f"• {med}")
            y -= 0.22*inch
    
    # Additional Information
    y -= 0.3*inch
    if y < 2*inch:  # New page if needed
        c.showPage()
        y = height - 1*inch
    
    c.setFont("Helvetica-Bold", 12)
    c.drawString(1*inch, y, "ADDITIONAL INFORMATION")
    y -= 0.3*inch
    c.setFont("Helvetica", 10)
    
    if patient_data.get("pregnant"):
        c.drawString(1*inch, y, "Pregnancy Status: YES")
        c.drawString(1*inch, y - 0.25*inch, f"Estimated Due Date: {datetime.now() + timedelta(days=84):%Y-%m-%d}")
        y -= 0.5*inch
    else:
        c.drawString(1*inch, y, "Pregnancy Status: NO")
        y -= 0.3*inch
    
    # Random additional data to add noise
    random_data = [
        f"Last visit: {(datetime.now() - timedelta(days=random.randint(1, 90))).strftime('%Y-%m-%d')}",
        f"Provider: Dr. {'Smith' if random.random() > 0.5 else 'Jones'}",
        f"Referring physician: Dr. {'Williams' if random.random() > 0.5 else 'Brown'}",
        f"Appointment duration: {random.randint(15, 60)} minutes",
        f"Visit type: {'Follow-up' if random.random() > 0.5 else 'Consultation'}",
    ]
    
    c.setFont("Helvetica", 9)
    for data in random_data[:2]:  # Add 2 random fields
        if y < 1.5*inch:
            break
        c.drawString(1*inch, y, data)
        y -= 0.22*inch
    
    # Footer on last page
    if y < 2*inch:  # New page if needed
        c.showPage()
        y = height - 1*inch
    
    c.setFont("Helvetica", 8)
    c.drawString(1*inch, 0.5*inch, f"Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')} | Record ID: {patient_data['patient_id']}-{random.randint(1000, 9999)}")
    
    c.save()

def main():
    """Generate 10 detailed patient PDF files."""
    patients_dir = "patients"
    os.makedirs(patients_dir, exist_ok=True)
    
    print(f"Generating {len(PATIENT_DATA)} detailed patient PDF files...")
    
    for patient in PATIENT_DATA:
        pdf_path = os.path.join(patients_dir, f"{patient['patient_id']}.pdf")
        create_patient_pdf(patient, pdf_path)
        print(f"  Created: {pdf_path}")
    
    print(f"\n✓ Successfully generated {len(PATIENT_DATA)} detailed patient PDF files in '{patients_dir}/'")
    print("  Each PDF contains: clinical notes, lab values, medications, vital signs, and random data")

if __name__ == "__main__":
    main()
