import os

errors = [
    "ARC-002-V1", "AUD-001-V1", "AUD-002-V1", "AUD-002-V2", "AUD-003-V1", "AUD-003-V2", "AUD-004-V1",
    "CAL-001-V1", "CAL-002-V2", "CAL-003-V1", "CAL-004-V1", "CLK-001-V1", "COST-001-V1", "DAT-001-V1",
    "DEM-001-V1", "DEM-002-V1", "DEM-003-V1", "DEM-004-V1", "DEM-005-V1", "DEM-006-V1", "DOC-001-V1",
    "EXP-002-V1", "FCP-001-V1", "FCP-002-V1", "IPC-001-V1", "IPC-002-V1", "IPC-003-V1", "IPC-004-V1",
    "IPC-004-V2", "IPC-005-V1", "MET-001-V1", "MET-002-V1", "MET-003-V1", "MET-004-V1", "MET-005-V1",
    "PLT-001-V1", "REP-001-V1", "REP-002-V1", "ROB-001-V1", "SEC-001-V1", "SEC-002-V1", "SEC-003-V1",
    "SEC-004-V1", "SIM-001-V1", "SIM-002-V1", "SIM-003-V1", "SIM-004-V1", "SIM-005-V1", "STA-001-V1",
    "STA-002-V1", "STA-003-V1", "TEL-001-V1", "TEL-002-V1", "TIM-001-V1", "TIM-002-V1", "TYP-001-V1",
    "TYP-002-V1", "TYP-003-V1"
]

base_dir = r"c:\Users\Jean\Desktop\binary_event_forecasting\verification\tests"
os.makedirs(base_dir, exist_ok=True)

for err in errors:
    d = os.path.join(base_dir, err)
    os.makedirs(d, exist_ok=True)
    with open(os.path.join(d, "test_stub.py"), "w") as f:
        f.write("def test_stub():\n    pass\n")

print("Created test directories.")
