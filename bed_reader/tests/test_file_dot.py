import logging
import os
import platform
from pathlib import Path

import numpy as np
import pytest

# Be sure any tests in here are actually read


if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)

    shared_datadir = Path(r"D:\OneDrive\programs\bed-reader\bed_reader\tests\data")
    tmp_path = Path(r"m:/deldir/tests")

    small_array = np.array([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]], order="F")
    mm = np.memmap(
        tmp_path / "small_array.memmap",
        dtype="float64",
        mode="w+",
        shape=(2, 3),
        order="F",
    )
    mm[:] = small_array[:]
    print(mm.offset)
    mm.flush()

    # test_zero_files(tmp_path)
    # test_index(shared_datadir)
    # test_c_reader_bed(shared_datadir)
    # test_read1(shared_datadir)
    pytest.main([__file__])
