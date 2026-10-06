from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[2]
TESTDATA = REPO / "testdata"
SAMPLES = REPO / "samples"

needs_samples = pytest.mark.skipif(not SAMPLES.is_dir(), reason="samples/ not present")
