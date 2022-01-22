if __name__ == "__main__":

    import numpy as np
    from pathlib import Path
    from bed_reader import to_bed
    import time
    import pandas as pd
    import matplotlib.pyplot as plt

    if True:
        ssd_path = Path("m:/deldir/bench")
        hdd_path = Path("e:/deldir/bench")
    else:
        ssd_path = Path("/mnt/m/deldir/bench")
        hdd_path = Path("/mnt/e/deldir/bench")

    def test_writes(
        iid_count, sid_count, num_threads, drive, include_error, version_list
    ):
        if drive == "ssd":
            path = ssd_path
        elif drive == "hdd":
            path = hdd_path
        else:
            raise ValueError(f"drive must be 'ssd' or 'hdd', not '{drive}'")

        output_file = path / f"{iid_count}x{sid_count}.bed"

        val = np.full((iid_count, sid_count), 1, dtype=np.int8)
        if include_error:
            val[iid_count // 2, sid_count // 2] = 22

        val_size = float(iid_count) * sid_count
        if val_size > 9_200_000_000:
            raise ValueError(f"val_size {val_size} is too large")

        result_list = []
        for version in version_list:
            start = time.time()
            to_bed(output_file, val, num_threads=num_threads, version=version)
            delta = time.time() - start
            result = [
                iid_count,
                sid_count,
                num_threads,
                drive,
                include_error,
                version,
                True,
                "i8",
                "benchmark.py",
                val_size,
                round(delta, 4),
                round(val_size / delta),
                round(val_size / delta / num_threads),
            ]
            print(result)
            result_list.append(result)

        result_df = pd.DataFrame(
            result_list,
            columns=[
                "iid_count",
                "sid_count",
                "num_threads",
                "drive",
                "include_error",
                "version",
                "release",
                "dtype",
                "source",
                "val size",
                "time",
                "val per second",
                "per thread",
            ],
        )
        return result_df

    # 5K vs 50K
    # 50K vs 50K
    # 500K vs 5K
    result = []
    for sid_count in np.logspace(np.log10(5), np.log10(50_000), 30, base=10, dtype=int):
        iid_count = 50_000
        for drive in ["hdd"]:  # , "hdd"]:
            for num_threads in [1, 20]:
                result.append(
                    test_writes(iid_count, sid_count, num_threads, drive, False, [3, 4])
                )
    df = pd.concat(result)
    df2 = df.pivot(
        index="sid_count",
        columns=["iid_count", "drive", "num_threads", "version"],
        values="val per second",
    )
    df2.plot(marker=".", logx=True)
    plt.show()