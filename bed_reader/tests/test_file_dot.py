import logging
from pathlib import Path

import numpy as np
import pytest

from bed_reader import file_b_less_aatbx, file_dot_piece
from bed_reader._open_bed import get_num_threads, open_bed  # noqa


def file_dot(filename, offset, iid_count, sid_count, sid_step):
    ata = np.full((sid_count, sid_count), np.nan)
    for sid_start in range(0, sid_count, sid_step):
        sid_range_len = min(sid_step, sid_count - sid_start)
        ata_piece = np.full((sid_count - sid_start, sid_range_len), np.nan)
        file_dot_piece(
            str(filename),
            offset,
            iid_count,
            sid_start,
            ata_piece,
            num_threads=get_num_threads(None),
            log_frequency=sid_range_len,
        )
        ata[sid_start:, sid_start : sid_start + sid_range_len] = ata_piece
    for sid_index in range(sid_count):
        ata[sid_index, sid_index + 1 :] = ata[sid_index + 1 :, sid_index]
    return ata


def write_read_test_file_dot(iid_count, sid_count, sid_step, tmp_path):
    offset = 640
    file_path = tmp_path / f"{iid_count}x{sid_count}_o{offset}_array.memmap"
    mm = np.memmap(
        file_path,
        dtype="float64",
        mode="w+",
        offset=offset,
        shape=(iid_count, sid_count),
        order="F",
    )
    mm[:] = np.linspace(0, 1, mm.size).reshape(mm.shape)
    mm.flush()

    out_val = file_dot(file_path, offset, iid_count, sid_count, sid_step)
    expected = mm.T.dot(mm)
    assert np.allclose(expected, out_val, equal_nan=True)


def test_file_dot_medium(tmp_path):
    write_read_test_file_dot(100, 1000, 33, tmp_path)


# # Too slow
# def test_file_dot_giant(tmp_path):
#     write_read_test_file_dot(100_000, 10_000, 1000, tmp_path)


def test_file_dot_small(shared_datadir):

    filename = shared_datadir / "small_array.memmap"

    out_val = file_dot(filename, 0, 2, 3, 2)
    print(out_val)

    expected = np.array([[17.0, 22.0, 27.0], [22.0, 29.0, 36.0], [27.0, 36.0, 45.0]])
    print(expected)
    assert np.allclose(expected, out_val, equal_nan=True)


def mmultfile_b_less_aatb(a_snp_mem_map, b, log_frequency=0, force_python_only=False):

    # Without memory efficiency
    #   a=a_snp_mem_map.val
    #   aTb = np.dot(a.T,b)
    #   aaTb = b-np.dot(a,aTb)
    #   return aTb, aaTb

    if force_python_only:
        aTb = np.zeros(
            (a_snp_mem_map.shape[1], b.shape[1])
        )  # b can be destroyed. Is everything is in best order, i.e. F vs C
        aaTb = b.copy()
        b_mem = np.array(b, order="F")
        with open(a_snp_mem_map.filename, "rb") as U_fp:
            U_fp.seek(a_snp_mem_map.offset)
            for i in range(a_snp_mem_map.shape[1]):
                a_mem = np.fromfile(
                    U_fp, dtype=np.float64, count=a_snp_mem_map.shape[0]
                )
                if log_frequency > 0 and i % log_frequency == 0:
                    logging.info("{0}/{1}".format(i, a_snp_mem_map.shape[1]))
                aTb[i, :] = np.dot(a_mem, b_mem)
                aaTb -= np.dot(a_mem.reshape(-1, 1), aTb[i : i + 1, :])
    else:
        b1 = np.array(b, order="F")
        aTb = np.zeros((a_snp_mem_map.shape[1], b.shape[1]))
        aaTb = np.array(b1, order="F")

        file_b_less_aatbx(
            str(a_snp_mem_map.filename),
            a_snp_mem_map.offset,
            a_snp_mem_map.shape[0],  # row count
            b1,  # B copy 1 in "F" order
            aaTb,  # B copy 2 in "F" order
            aTb,  # result
            num_threads=get_num_threads(None),
            log_frequency=log_frequency,
        )

    return aTb, aaTb


def write_read_test_file_b_less_aatbx(
    iid_count, a_sid_count, b_sid_count, log_frequency, tmp_path, do_both=True
):
    offset = 640
    file_path = tmp_path / f"{iid_count}x{a_sid_count}_o{offset}_array.memmap"
    mm = np.memmap(
        file_path,
        dtype="float64",
        mode="w+",
        offset=offset,
        shape=(iid_count, a_sid_count),
        order="F",
    )
    mm[:] = np.linspace(0, 1, mm.size).reshape(mm.shape)
    mm.flush()

    b = np.array(
        np.linspace(0, 1, iid_count * b_sid_count).reshape(
            (iid_count, b_sid_count), order="F"
        )
    )
    b_again = b.copy()

    logging.info("Calling Rust")
    aTb, aaTb = mmultfile_b_less_aatb(
        mm, b_again, log_frequency, force_python_only=False
    )

    if do_both:
        logging.info("Calling Python")
        aTb_python, aaTb_python = mmultfile_b_less_aatb(
            mm, b, log_frequency, force_python_only=True
        )

        if (
            not np.abs(aTb_python - aTb).max() < 1e-8
            or not np.abs(aaTb_python - aaTb).max() < 1e-8
        ):
            raise AssertionError(
                "Expect Python and Rust to get the same mmultfile_b_less_aatb answer"
            )


def test_file_b_less_aatbx_medium(tmp_path):
    write_read_test_file_b_less_aatbx(500, 400, 100, 10, tmp_path, do_both=True)


def test_file_b_less_aatbx_medium2(tmp_path):
    write_read_test_file_b_less_aatbx(5_000, 400, 100, 100, tmp_path, do_both=True)


# Slow and doesn't check answer
# def test_file_b_less_aatbx_2(tmp_path):
#     write_read_test_file_b_less_aatbx(50_000, 4000, 1000, 100, tmp_path, do_both=False)


if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)
    shared_datadir = Path(r"D:\OneDrive\programs\bed-reader\bed_reader\tests\data")
    tmp_path = Path(r"m:/deldir/tests")

    if False:
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

    if False:
        mm = np.memmap(
            tmp_path / "100x1000_o640_array.memmap",
            dtype="float64",
            mode="w+",
            offset=640,
            shape=(100, 1000),
            order="F",
        )
        total = mm.shape[0] * mm.shape[1]
        lin = np.linspace(0, 1, total).reshape(mm.shape)

        mm[:] = lin[:]
        print(mm.offset)
        mm.flush()

    if False:
        mm = np.memmap(
            tmp_path / "1000x10000_o640_array.memmap",
            dtype="float64",
            mode="w+",
            offset=640,
            shape=(1000, 10_000),
            order="F",
        )
        total = mm.shape[0] * mm.shape[1]
        lin = np.linspace(0, 1, total).reshape(mm.shape)

        mm[:] = lin[:]
        print(mm.offset)
        mm.flush()

    if False:
        mm = np.memmap(
            tmp_path / "10_000x100_000_o6400_array.memmap",
            dtype="float64",
            mode="w+",
            offset=640,
            shape=(10_000, 100_000),
            order="F",
        )
        total = mm.shape[0] * mm.shape[1]
        lin = np.linspace(0, 1, total).reshape(mm.shape)

        mm[:] = lin[:]
        print(mm.offset)
        mm.flush()

    if False:
        mm = np.memmap(
            tmp_path / "100_000x10_000_o6400_array.memmap",
            dtype="float64",
            mode="w+",
            offset=640,
            shape=(100_000, 10_000),
            order="F",
        )
        total = mm.shape[0] * mm.shape[1]
        lin = np.linspace(0, 1, total).reshape(mm.shape)

        mm[:] = lin[:]
        print(mm.offset)
        mm.flush()

    # test_file_b_less_aatbx_2(tmp_path)
    # test_zero_files(tmp_path)
    # test_index(shared_datadir)
    # test_c_reader_bed(shared_datadir)
    # test_read1(shared_datadir)
    pytest.main([__file__])
