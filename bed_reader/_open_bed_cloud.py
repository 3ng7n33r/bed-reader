import logging
# cmk import multiprocessing
import os
# cmk from dataclasses import dataclass
# cmk from itertools import repeat, takewhile
from pathlib import Path
from typing import Any, List, Mapping, Optional, Union

import numpy as np

try:
    from scipy import sparse
except ImportError:
    sparse = None

from .bed_reader import read_cloud_i8  # type: ignore
from bed_reader._open_bed import _meta_meta, _read_csv, _rawincount, _delimiters, get_num_threads, _count_name


class open_bed_cloud:
    """
    Open a PLINK .bed file for reading.

    Parameters
    ----------
    filepath: pathlib.Path or str
        File path to the .bed file.
    iid_count: None or int, optional
        Number of individuals (samples) in the .bed file.
        The default (``iid_count=None``) finds the number
        automatically by quickly scanning the .fam file.
    sid_count: None or int, optional
        Number of SNPs (variants) in the .bed file.
        The default (``sid_count=None``) finds the number
        automatically by quickly scanning the .bim file.
    properties: dict, optional
        A dictionary of any replacement properties. The default is an empty dictionary.
        The keys of the dictionary are the names of the properties to replace.
        The possible keys are:

             "fid" (family id), "iid" (individual or sample id), "father" (father id),
             "mother" (mother id), "sex", "pheno" (phenotype), "chromosome", "sid"
             (SNP or variant id), "cm_position" (centimorgan position), "bp_position"
             (base-pair position), "allele_1", "allele_2".

          The values are replacement lists or arrays. A value can also be `None`,
          meaning do not read or offer this property. See examples, below.

          The list or array will be converted to a :class:`numpy.ndarray`
          of the appropriate dtype, if necessary. Any :data:`numpy.nan` values
          will converted to the appropriate missing value. The PLINK `.fam specification
          <https://www.cog-genomics.org/plink2/formats#fam>`_
          and `.bim specification <https://www.cog-genomics.org/plink2/formats#bim>`_
          lists the dtypes and missing values for each property.

    count_A1: bool, optional
        True (default) to count the number of A1 alleles (the PLINK standard).
        False to count the number of A2 alleles.
    num_threads: None or int, optional
        The number of threads with which to read data. Defaults to all available
        processors.
        Can also be set with these environment variables (listed in priority order):
        'PST_NUM_THREADS', 'NUM_THREADS', 'MKL_NUM_THREADS'.
    skip_format_check: bool, optional
        False (default) to immediately check for expected starting bytes in
        the .bed file. True to delay the check until (and if) data is read.
    fam_filepath: pathlib.Path or str, optional
        Path to the file containing information about each individual (sample).
        Defaults to replacing the .bed file’s suffix with .fam.
    bim_filepath: pathlib.Path or str, optional
        Path to the file containing information about each SNP (variant).
        Defaults to replacing the .bed file’s suffix with .bim.

    Returns
    -------
    open_bed
        an open_bed object

    Examples
    --------

    List individual (sample) :attr:`iid` and SNP (variant) :attr:`sid`, then :meth:`read`
    the whole file.

    .. doctest::

        >>> from bed_reader import open_bed, sample_file
        >>>
        >>> file_name = sample_file("small.bed")
        >>> bed = open_bed(file_name)
        >>> print(bed.iid)
        ['iid1' 'iid2' 'iid3']
        >>> print(bed.sid)
        ['sid1' 'sid2' 'sid3' 'sid4']
        >>> print(bed.read())
        [[ 1.  0. nan  0.]
         [ 2.  0. nan  2.]
         [ 0.  1.  2.  0.]]
        >>> del bed  # optional: delete bed object

    Open the file and read data for one SNP (variant)
    at index position 2.

    .. doctest::

        >>> import numpy as np
        >>> with open_bed(file_name) as bed:
        ...     print(bed.read(np.s_[:,2]))
        [[nan]
         [nan]
         [ 2.]]

    Replace :attr:`iid`.


        >>> bed = open_bed(file_name, properties={"iid":["sample1","sample2","sample3"]})
        >>> print(bed.iid) # replaced
        ['sample1' 'sample2' 'sample3']
        >>> print(bed.sid) # same as before
        ['sid1' 'sid2' 'sid3' 'sid4']

    Give the number of individuals (samples) and SNPs (variants) so that the .fam and
    .bim files need never be opened.

        >>> with open_bed(file_name, iid_count=3, sid_count=4) as bed:
        ...     print(bed.read())
        [[ 1.  0. nan  0.]
         [ 2.  0. nan  2.]
         [ 0.  1.  2.  0.]]

    Mark some properties as "don’t read or offer".

        >>> bed = open_bed(file_name, properties={
        ...    "father" : None, "mother" : None, "sex" : None, "pheno" : None,
        ...    "allele_1" : None, "allele_2":None })
        >>> print(bed.iid)        # read from file
        ['iid1' 'iid2' 'iid3']
        >>> print(bed.allele_2)   # not read and not offered
        None

    See the :meth:`read` for details of reading batches via slicing and fancy indexing.

    """

    async def __init__(
        self,
        filepath: Union[str, Path],
        iid_count: Optional[int] = None,
        sid_count: Optional[int] = None,
        properties: Mapping[str, List[Any]] = {},
        count_A1: bool = True,
        num_threads: Optional[int] = None,
        skip_format_check: bool = False,
        fam_filepath: Union[str, Path] = None,
        bim_filepath: Union[str, Path] = None,
    ):
        self.filepath = Path(filepath)
        self.count_A1 = count_A1
        self._num_threads = num_threads
        self.skip_format_check = skip_format_check
        self._fam_filepath = (
            Path(fam_filepath)
            if fam_filepath is not None
            else self.filepath.parent / (self.filepath.stem + ".fam")
        )
        self._bim_filepath = (
            Path(bim_filepath)
            if bim_filepath is not None
            else self.filepath.parent / (self.filepath.stem + ".bim")
        )

        self.properties_dict, self._counts = open_bed_cloud._fix_up_properties(
            properties, iid_count, sid_count, use_fill_sequence=False
        )
        self._iid_range = None
        self._sid_range = None

        if not self.skip_format_check:
            with open(self.filepath, "rb") as filepointer:
                self._check_file(filepointer)

    async def read(
        self,
        index: Optional[Any] = None,
        dtype: Optional[Union[type, str]] = "float32",
        order: Optional[str] = "F",
        max_concurrent_requests=None,
        max_chunk_size=None,
        num_threads=None,
    ) -> np.ndarray:
        """
        Read genotype information from a cloud location. // cmk update docs

        Parameters
        ----------
        index:
            An optional expression specifying the individuals (samples) and SNPs
            (variants) to read. (See examples, below).
            Defaults to ``None``, meaning read all.

            (If index is a tuple, the first component indexes the individuals and the
            second indexes
            the SNPs. If it is not a tuple and not None, it indexes SNPs.)

        dtype: {'float32' (default), 'float64', 'int8'}, optional
            The desired data-type for the returned array.
        order : {'F','C'}, optional
            The desired memory layout for the returned array.
            Defaults to ``F`` (Fortran order, which is SNP-major).

        num_threads: None or int, optional
            The number of threads with which to read data. Defaults to all available
            processors.
            Can also be set with :class:`open_bed` or these
            environment variables (listed in priority order):
            'PST_NUM_THREADS', 'NUM_THREADS', 'MKL_NUM_THREADS'.


        Returns
        -------
        numpy.ndarray
            2-D array containing values of 0, 1, 2, or missing


        Rows represent individuals (samples). Columns represent SNPs (variants).

        For ``dtype`` 'float32' and 'float64', NaN indicates missing values.
        For 'int8', -127 indicates missing values.

        Examples
        --------

        To read all data in a .bed file, set ``index`` to ``None``. This is the default.

        .. doctest::

            >>> from bed_reader import open_bed, sample_file
            >>>
            >>> file_name = sample_file("small.bed")
            >>> with open_bed(file_name) as bed:
            ...     print(bed.read())
            [[ 1.  0. nan  0.]
             [ 2.  0. nan  2.]
             [ 0.  1.  2.  0.]]

        To read selected individuals (samples) and/or SNPs (variants), set each part of
        a :data:`numpy.s_` to an `int`, a list of `int`, a slice expression, or
        a list of `bool`.
        Negative integers count from the end of the list.


        .. doctest::

            >>> import numpy as np
            >>> bed = open_bed(file_name)
            >>> print(bed.read(np.s_[:,2]))  # read the SNPs indexed by 2.
            [[nan]
             [nan]
             [ 2.]]
            >>> print(bed.read(np.s_[:,[2,3,0]]))  # read the SNPs indexed by 2, 3, and 0
            [[nan  0.  1.]
             [nan  2.  2.]
             [ 2.  0.  0.]]
            >>> # read SNPs from 1 (inclusive) to 4 (exclusive)
            >>> print(bed.read(np.s_[:,1:4]))
            [[ 0. nan  0.]
             [ 0. nan  2.]
             [ 1.  2.  0.]]
            >>> print(np.unique(bed.chromosome)) # print unique chrom values
            ['1' '5' 'Y']
            >>> print(bed.read(np.s_[:,bed.chromosome=='5'])) # read all SNPs in chrom 5
            [[nan]
             [nan]
             [ 2.]]
            >>> print(bed.read(np.s_[0,:])) # Read 1st individual (across all SNPs)
            [[ 1.  0. nan  0.]]
            >>> print(bed.read(np.s_[::2,:])) # Read every 2nd individual
            [[ 1.  0. nan  0.]
             [ 0.  1.  2.  0.]]
            >>> #read last and 2nd-to-last individuals and the last SNPs
            >>> print(bed.read(np.s_[[-1,-2],-1]))
            [[0.]
             [2.]]


        You can give a dtype for the output.

        .. doctest::

            >>> print(bed.read(dtype='int8'))
            [[   1    0 -127    0]
             [   2    0 -127    2]
             [   0    1    2    0]]
            >>> del bed  # optional: delete bed object

        """

        iid_index_or_slice_etc, sid_index_or_slice_etc = self._split_index(index)

        dtype = np.dtype(dtype)
        if order not in {"F", "C"}:
            raise ValueError(f"order '{order}' not known, only 'F', 'C'")

        # Later happy with _iid_range and _sid_range or could it be done with
        # allocation them?
        if self._iid_range is None:
            self._iid_range = np.arange(self.iid_count, dtype="intp")
        if self._sid_range is None:
            self._sid_range = np.arange(self.sid_count, dtype="intp")

        iid_index = np.ascontiguousarray(
            self._iid_range[iid_index_or_slice_etc],
            dtype="intp",
        )
        sid_index = np.ascontiguousarray(
            self._sid_range[sid_index_or_slice_etc], dtype="intp"
        )

        num_threads = get_num_threads(
            self._num_threads if num_threads is None else num_threads
        )

        val = np.zeros((len(iid_index), len(sid_index)), order=order, dtype=dtype)

        if self.iid_count > 0 and self.sid_count > 0:
            if dtype == np.int8:
                reader = read_cloud_i8
            # elif dtype == np.float64:
            #     reader = read_f64
            # elif dtype == np.float32:
            #     reader = read_f32
            else:
                raise ValueError(
                    f"dtype '{val.dtype}' not known, only "
                    + "'int8', 'float32', and 'float64' are allowed."
                )

            await reader(
                str(self.filepath),
                iid_count=self.iid_count,
                sid_count=self.sid_count,
                is_a1_counted=self.count_A1,
                iid_index=iid_index,
                sid_index=sid_index,
                val=val,
                num_threads=num_threads,
            )

        return val

    def __str__(self) -> str:
        return f"{self.__class__.__name__}('{self.filepath}',...)"

    @property
    async def fid(self) -> np.ndarray:
        """
        Family id of each individual (sample).

        Returns
        -------
        numpy.ndarray
            array of str


        '0' represents a missing value.

        If needed, will cause a one-time read of the .fam file.

        Example
        -------

        .. doctest::

            >>> from bed_reader import open_bed, sample_file
            >>>
            >>> file_name = sample_file("small.bed")
            >>> with open_bed(file_name) as bed:
            ...     print(bed.fid)
            ['fid1' 'fid1' 'fid2']

        """

        return self.property_item("fid")

    @property
    async def iid(self) -> np.ndarray:
        """
        Individual id of each individual (sample).

        Returns
        -------
        numpy.ndarray
            array of str


        If needed, will cause a one-time read of the .fam file.

        Example
        -------

        .. doctest::

            >>> from bed_reader import open_bed, sample_file
            >>>
            >>> file_name = sample_file("small.bed")
            >>> with open_bed(file_name) as bed:
            ...     print(bed.iid)
            ['iid1' 'iid2' 'iid3']

        """
        return self.property_item("iid")

    @property
    async def father(self) -> np.ndarray:
        """
        Father id of each individual (sample).

        Returns
        -------
        numpy.ndarray
            array of str


        '0' represents a missing value.

        If needed, will cause a one-time read of the .fam file.

        Example
        -------

        .. doctest::

            >>> from bed_reader import open_bed, sample_file
            >>>
            >>> file_name = sample_file("small.bed")
            >>> with open_bed(file_name) as bed:
            ...     print(bed.father)
            ['iid23' 'iid23' 'iid22']

        """
        return self.property_item("father")

    @property
    async def mother(self) -> np.ndarray:
        """
        Mother id of each individual (sample).

        Returns
        -------
        numpy.ndarray
            array of str


        '0' represents a missing value.

        If needed, will cause a one-time read of the .fam file.

        Example
        -------

        .. doctest::

            >>> from bed_reader import open_bed, sample_file
            >>>
            >>> file_name = sample_file("small.bed")
            >>> with open_bed(file_name) as bed:
            ...     print(bed.mother)
            ['iid34' 'iid34' 'iid33']

        """
        return self.property_item("mother")

    @property
    async def sex(self) -> np.ndarray:
        """
        Sex of each individual (sample).

        Returns
        -------
        numpy.ndarray
            array of 0, 1, or 2


        0 is unknown, 1 is male, 2 is female

        If needed, will cause a one-time read of the .fam file.

        Example
        -------
        .. doctest::

            >>> from bed_reader import open_bed, sample_file
            >>>
            >>> file_name = sample_file("small.bed")
            >>> with open_bed(file_name) as bed:
            ...     print(bed.sex)
            [1 2 0]

        """
        return self.property_item("sex")

    @property
    async def pheno(self) -> np.ndarray:
        """
        A phenotype for each individual (sample)
        (seldom used).

        Returns
        -------
        numpy.ndarray
            array of str


        '0' may represent a missing value.

        If needed, will cause a one-time read of the .fam file.

        Example
        -------

        .. doctest::

            >>> from bed_reader import open_bed, sample_file
            >>>
            >>> file_name = sample_file("small.bed")
            >>> with open_bed(file_name) as bed:
            ...     print(bed.pheno)
            ['red' 'red' 'blue']

        """
        return self.property_item("pheno")

    @property
    async def properties(self) -> Mapping[str, np.array]:
        """
        All the properties returned as a dictionary.

        Returns
        -------
        dict
            all the properties


        The keys of the dictionary are the names of the properties, namely:

             "fid" (family id), "iid" (individual or sample id), "father" (father id),
             "mother" (mother id), "sex", "pheno" (phenotype), "chromosome", "sid"
             (SNP or variant id), "cm_position" (centimorgan position), "bp_position"
             (base-pair position), "allele_1", "allele_2".

        The values are :class:`numpy.ndarray`.

        If needed, will cause a one-time read of the .fam and .bim file.

        Example
        -------

        .. doctest::

            >>> from bed_reader import open_bed, sample_file
            >>>
            >>> file_name = sample_file("small.bed")
            >>> with open_bed(file_name) as bed:
            ...     print(len(bed.properties)) #length of dict
            12

        """
        for key in _meta_meta:
            self.property_item(key)
        return self.properties_dict

    async def property_item(self, name: str) -> np.ndarray:
        """
        Retrieve one property by name.

        Returns
        -------
        numpy.ndarray
            a property value


        The name is one of these:

             "fid" (family id), "iid" (individual or sample id), "father" (father id),
             "mother" (mother id), "sex", "pheno" (phenotype), "chromosome", "sid"
             (SNP or variant id), "cm_position" (centimorgan position), "bp_position"
             (base-pair position), "allele_1", "allele_2".

        If needed, will cause a one-time read of the .fam or .bim file.

        Example
        -------

        .. doctest::

            >>> from bed_reader import open_bed, sample_file
            >>>
            >>> file_name = sample_file("small.bed")
            >>> with open_bed(file_name) as bed:
            ...     print(bed.property_item('chromosome'))
            ['1' '1' '5' 'Y']

        """
        if name not in self.properties_dict:
            mm = _meta_meta[name]
            self._read_fam_or_bim(suffix=mm.suffix)
        return self.properties_dict[name]

    @property
    async def chromosome(self) -> np.ndarray:
        """
        Chromosome of each SNP (variant)

        Returns
        -------
        numpy.ndarray
            array of str


        '0' represents a missing value.

        If needed, will cause a one-time read of the .bim file.

        Example
        -------

        .. doctest::

            >>> from bed_reader import open_bed, sample_file
            >>>
            >>> file_name = sample_file("small.bed")
            >>> with open_bed(file_name) as bed:
            ...     print(bed.chromosome)
            ['1' '1' '5' 'Y']

        """
        return self.property_item("chromosome")

    @property
    async def sid(self) -> np.ndarray:
        """
        SNP id of each SNP (variant).

        Returns
        -------
        numpy.ndarray
            array of str


        If needed, will cause a one-time read of the .bim file.

        Example
        -------

        .. doctest::

            >>> from bed_reader import open_bed, sample_file
            >>>
            >>> file_name = sample_file("small.bed")
            >>> with open_bed(file_name) as bed:
            ...     print(bed.sid)
            ['sid1' 'sid2' 'sid3' 'sid4']

        """
        return self.property_item("sid")

    @property
    async def cm_position(self) -> np.ndarray:
        """
        Centimorgan position of each SNP (variant).

        Returns
        -------
        numpy.ndarray
            array of float


        0.0 represents a missing value.

        If needed, will cause a one-time read of the .bim file.

        Example
        -------

        .. doctest::

            >>> from bed_reader import open_bed, sample_file
            >>>
            >>> file_name = sample_file("small.bed")
            >>> with open_bed(file_name) as bed:
            ...     print(bed.cm_position)
            [ 100.4 2000.5 4000.7 7000.9]

        """
        return self.property_item("cm_position")

    @property
    async def bp_position(self) -> np.ndarray:
        """
        Base-pair position of each SNP (variant).

        Returns
        -------
        numpy.ndarray
            array of int


        0 represents a missing value.

        If needed, will cause a one-time read of the .bim file.

        Example
        -------

        .. doctest::

            >>> from bed_reader import open_bed, sample_file
            >>>
            >>> file_name = sample_file("small.bed")
            >>> with open_bed(file_name) as bed:
            ...     print(bed.bp_position)
            [   1  100 1000 1004]

        """
        return self.property_item("bp_position")

    @property
    async def allele_1(self) -> np.ndarray:
        """
        First allele of each SNP (variant).

        Returns
        -------
        numpy.ndarray
            array of str


        If needed, will cause a one-time read of the .bim file.

        Example
        -------

        .. doctest::

            >>> from bed_reader import open_bed, sample_file
            >>>
            >>> file_name = sample_file("small.bed")
            >>> with open_bed(file_name) as bed:
            ...     print(bed.allele_1)
            ['A' 'T' 'A' 'T']

        """
        return self.property_item("allele_1")

    @property
    async def allele_2(self) -> np.ndarray:
        """
        Second allele of each SNP (variant),

        Returns
        -------
        numpy.ndarray
            array of str


        If needed, will cause a one-time read of the .bim file.

        Example
        -------

        .. doctest::

            >>> from bed_reader import open_bed, sample_file
            >>>
            >>> file_name = sample_file("small.bed")
            >>> with open_bed(file_name) as bed:
            ...     print(bed.allele_2)
            ['A' 'C' 'C' 'G']

        """
        return self.property_item("allele_2")

    @property
    async def iid_count(self) -> np.ndarray:
        """
        Number of individuals (samples).

        Returns
        -------
        int
            number of individuals


        If needed, will cause a fast line-count of the .fam file.

        Example
        -------

        .. doctest::

            >>> from bed_reader import open_bed, sample_file
            >>>
            >>> file_name = sample_file("small.bed")
            >>> with open_bed(file_name) as bed:
            ...     print(bed.iid_count)
            3

        """
        return self._count("fam")

    @property
    async def sid_count(self) -> np.ndarray:
        """
        Number of SNPs (variants).

        Returns
        -------
        int
            number of SNPs


        If needed, will cause a fast line-count of the .bim file.

        Example
        -------

        .. doctest::

            >>> from bed_reader import open_bed, sample_file
            >>>
            >>> file_name = sample_file("small.bed")
            >>> with open_bed(file_name) as bed:
            ...     print(bed.sid_count)
            4

        """
        return self._count("bim")

    def _property_filepath(self, suffix):
        if suffix == "fam":
            return self._fam_filepath
        else:
            assert suffix == "bim"  # real assert
            return self._bim_filepath

    async def _count(self, suffix):
        count = self._counts[suffix]
        if count is None:
            count = _rawincount(self._property_filepath(suffix))
            self._counts[suffix] = count
        return count

    @staticmethod
    async def _check_file(filepointer):
        mode = filepointer.read(2)
        if mode != b"l\x1b":
            raise ValueError("Not a valid .bed file")
        mode = filepointer.read(1)  # \x01 = SNP major \x00 = individual major
        if mode != b"\x01":
            raise ValueError("only SNP-major is implemented")

    def __del__(self):
        self.__exit__()

    def __enter__(self):
        return self

    def __exit__(self, *_):
        pass

    @staticmethod
    def _array_properties_are_ok(val, order):
        if order == "F":
            return val.flags["F_CONTIGUOUS"]
        else:
            assert order == "C"  # real assert
            return val.flags["C_CONTIGUOUS"]

    @property
    async def shape(self):
        """
        Number of individuals (samples) and SNPs (variants).

        Returns
        -------
        (int, int)
            number of individuals, number of SNPs


        If needed, will cause a fast line-count of the .fam and .bim files.

        Example
        -------

        .. doctest::

            >>> from bed_reader import open_bed, sample_file
            >>>
            >>> file_name = sample_file("small.bed")
            >>> with open_bed(file_name) as bed:
            ...     print(bed.shape)
            (3, 4)

        """
        return (len(self.iid), len(self.sid))

    @staticmethod
    def _split_index(index):
        if not isinstance(index, tuple):
            index = (None, index)
        iid_index = open_bed_cloud._fix_up_index(index[0])
        sid_index = open_bed_cloud._fix_up_index(index[1])
        return iid_index, sid_index

    @staticmethod
    def _fix_up_index(index):
        if index is None:  # make a shortcut for None
            return slice(None)
        try:  # If index is an int, return it in an array
            index = index.__index__()  # (see
            # https://stackoverflow.com/questions/3501382/checking-whether-a-variable-is-an-integer-or-not)
            return [index]
        except Exception:
            pass
        return index

    # @staticmethod
    # def _write_fam_or_bim(base_filepath, properties, suffix, property_filepath):
    #     assert suffix in {"fam", "bim"}, "real assert"

    #     filepath = (
    #         Path(property_filepath)
    #         if property_filepath is not None
    #         else base_filepath.parent / (base_filepath.stem + "." + suffix)
    #     )

    #     fam_bim_list = []
    #     for key, mm in _meta_meta.items():
    #         if mm.suffix == suffix:
    #             assert len(fam_bim_list) == mm.column, "real assert"
    #             fam_bim_list.append(properties[key])

    #     sep = " " if suffix == "fam" else "\t"

    #     with open(filepath, "w") as filepointer:
    #         for index in range(len(fam_bim_list[0])):
    #             filepointer.write(
    #                 sep.join(str(seq[index]) for seq in fam_bim_list) + "\n"
    #             )

    @staticmethod
    def _fix_up_properties_array(input, dtype, missing_value, key):
        if input is None:
            return None
        if len(input) == 0:
            return np.zeros([0], dtype=dtype)

        if not isinstance(input, np.ndarray):
            return open_bed_cloud._fix_up_properties_array(
                np.array(input), dtype, missing_value, key
            )

        if len(input.shape) != 1:
            raise ValueError(f"{key} should be one dimensional")

        if not np.issubdtype(input.dtype, dtype):
            # This will convert, for example, numerical sids to string sids or
            # floats that happen to be integers into ints,
            # but there will be a warning generated.
            output = np.array(input, dtype=dtype)
        else:
            output = input

        # Change NaN in input to correct missing value
        if np.issubdtype(input.dtype, np.floating):
            output[input != input] = missing_value

        return output

    @staticmethod
    def _fix_up_properties(properties, iid_count, sid_count, use_fill_sequence):
        for key in properties:
            if key not in _meta_meta:
                raise KeyError(f"properties key '{key}' not known")

        count_dict = {"fam": iid_count, "bim": sid_count}
        properties_dict = {}
        for key, mm in _meta_meta.items():
            count = count_dict[mm.suffix]

            if key not in properties or (use_fill_sequence and properties[key] is None):
                if use_fill_sequence:
                    output = mm.fill_sequence(key, count, mm.missing_value, mm.dtype)
                else:
                    continue  # Test coverage reaches this, but doesn't report it.
            else:
                output = open_bed_cloud._fix_up_properties_array(
                    properties[key], mm.dtype, mm.missing_value, key
                )

            if output is not None:
                if count is None:
                    count_dict[mm.suffix] = len(output)
                else:
                    if count != len(output):
                        raise ValueError(
                            f"The length of override {key}, {len(output)}, should not "
                            + "be different from the current "
                            + f"{_count_name[mm.suffix]}, {count}"
                        )
            properties_dict[key] = output
        return properties_dict, count_dict

    async def _read_fam_or_bim(self, suffix):
        property_filepath = self._property_filepath(suffix)

        logging.info("Loading {0} file {1}".format(suffix, property_filepath))

        count = self._counts[suffix]

        delimiter = _delimiters[suffix]
        if delimiter in {r"\s+"}:
            delimiter = None

        usecolsdict = {}
        dtype_dict = {}
        for key, mm in _meta_meta.items():
            if mm.suffix is suffix and key not in self.properties_dict:
                usecolsdict[key] = mm.column
                dtype_dict[mm.column] = mm.dtype
        assert list(usecolsdict.values()) == sorted(usecolsdict.values())  # real assert
        assert len(usecolsdict) > 0  # real assert

        if os.path.getsize(property_filepath) == 0:
            columns, row_count = [], 0
        else:
            columns, row_count = _read_csv(
                property_filepath,
                delimiter=delimiter,
                dtype=dtype_dict,
                usecols=usecolsdict.values(),
            )

        if count is None:
            self._counts[suffix] = row_count
        else:
            if count != row_count:
                raise ValueError(
                    f"The number of lines in the *.{suffix} file, {row_count}, "
                    + "should not be different from the current "
                    + "f{_count_name[suffix]}, {count}"
                )
        for i, key in enumerate(usecolsdict.keys()):
            mm = _meta_meta[key]
            if row_count == 0:
                output = np.array([], dtype=mm.dtype)
            else:
                output = columns[i]
                if not np.issubdtype(output.dtype, mm.dtype):
                    output = np.array(output, dtype=mm.dtype)
            self.properties_dict[key] = output

    # def read_sparse(
    #     self,
    #     index: Optional[Any] = None,
    #     dtype: Optional[Union[type, str]] = "float32",
    #     batch_size: Optional[int] = None,
    #     format: Optional[str] = "csc",
    #     num_threads=None,
    # ) -> (Union[sparse.csc_matrix, sparse.csr_matrix]) if sparse is not None else None:
    #     """
    #     Read genotype information into a :mod:`scipy.sparse` matrix. Sparse matrices
    #     may be useful when the data is mostly zeros.

    #     .. note::
    #         This method requires :mod:`scipy`. Install `scipy` with:

    #         .. code-block:: bash

    #             pip install --upgrade bed-reader[sparse]

    #     Parameters
    #     ----------
    #     index:
    #         An optional expression specifying the individuals (samples) and SNPs
    #         (variants) to read. (See examples, below).
    #         Defaults to ``None``, meaning read all.

    #         (If index is a tuple, the first component indexes the individuals and the
    #         second indexes
    #         the SNPs. If it is not a tuple and not None, it indexes SNPs.)

    #     dtype: {'float32' (default), 'float64', 'int8'}, optional
    #         The desired data-type for the returned array.
    #     batch_size: None or int, optional
    #         Number of dense columns or rows to read at a time, internally.
    #         Defaults to round(sqrt(total-number-of-columns-or-rows-to-read)).
    #     format : {'csc','csr'}, optional
    #         The desired format of the sparse matrix.
    #         Defaults to ``csc`` (Compressed Sparse Column, which is SNP-major).
    #     num_threads: None or int, optional
    #         The number of threads with which to read data. Defaults to all available
    #         processors.
    #         Can also be set with :class:`open_bed` or these
    #         environment variables (listed in priority order):
    #         'PST_NUM_THREADS', 'NUM_THREADS', 'MKL_NUM_THREADS'.

    #     Returns
    #     -------
    #     a :class:`scipy.sparse.csc_matrix` (default) or :class:`scipy.sparse.csr_matrix`
    #     Rows represent individuals (samples). Columns represent SNPs (variants).
    #     For ``dtype`` 'float32' and 'float64', NaN indicates missing values.
    #     For 'int8', -127 indicates missing values.
    #     The memory used by the final sparse matrix is approximately:
    #        # of non-zero values * (4 bytes + 1 byte (for int8))
    #     For example, consider reading 1000 individuals (samples) x 50,000 SNPs (variants)
    #     into csc format where the data is 97% sparse.
    #     The memory used will be about 7.5 MB (1000 x 50,000 x 3% x 5 bytes).
    #     This is 15% of the 50 MB needed by a dense matrix.

    #     Internally, the function reads the data via small dense matrices.
    #     For this example, by default, the function will read 1000 individuals x 224 SNPs
    #     (because 224 * 224 is about 50,000).
    #     The memory used by the small dense matrix is 1000 x 244 x 1 byte (for int8) = 0.224 MB.

    #     You can set `batch_size`. Larger values will be faster.
    #     Smaller values will use less memory.

    #     For this example, we might want to set the `batch_size` to 5000. Then,
    #     the memory used by the small dense matrix
    #     would be 1000 x 5000 x 1 byte (for int8) = 5 MB,
    #     similar to the 7.5 MB needed for the final sparse matrix.

    #     Examples
    #     --------

    #     Read all data in a .bed file into a :class:`scipy.sparse.csc_matrix`.
    #     The file has 10 individuals (samples) by 20 SNPs (variants).
    #     All but eight values are 0.

    #     .. doctest::

    #         >>> # pip install bed-reader[samples,sparse]  # if needed
    #         >>> from bed_reader import open_bed, sample_file
    #         >>>
    #         >>> file_name = sample_file("sparse.bed")
    #         >>> with open_bed(file_name) as bed:
    #         ...     print(bed.shape)
    #         ...     val_sparse = bed.read_sparse(dtype="int8")
    #         ...     print(val_sparse) # doctest:+NORMALIZE_WHITESPACE
    #         (10, 20)
    #             (8, 4)  1
    #             (8, 5)	2
    #             (0, 8)	2
    #             (4, 9)	1
    #             (7, 9)	1
    #             (5, 11)	1
    #             (2, 12)	1
    #             (3, 12)	1

    #     To read selected individuals (samples) and/or SNPs (variants), set each part of
    #     a :data:`numpy.s_` to an `int`, a list of `int`, a slice expression, or
    #     a list of `bool`.
    #     Negative integers count from the end of the list.

    #     .. doctest::

    #         >>> import numpy as np
    #         >>> bed = open_bed(file_name)
    #         >>> print(bed.read_sparse(np.s_[:,5], dtype="int8"))  # read the SNPs indexed by 5. # doctest:+NORMALIZE_WHITESPACE
    #         (8, 0)    2
    #         >>> # read the SNPs indexed by 5, 4, and 0
    #         >>> print(bed.read_sparse(np.s_[:,[5,4,0]], dtype="int8")) # doctest:+NORMALIZE_WHITESPACE
    #         (8, 0)	2
    #         (8, 1)	1
    #         >>> # read SNPs from 1 (inclusive) to 11 (exclusive)
    #         >>> print(bed.read_sparse(np.s_[:,1:11], dtype="int8")) # doctest:+NORMALIZE_WHITESPACE
    #         (8, 3)	1
    #         (8, 4)	2
    #         (0, 7)	2
    #         (4, 8)	1
    #         (7, 8)	1
    #         >>> print(np.unique(bed.chromosome)) # print unique chrom values
    #         ['1' '5' 'Y']
    #         >>> # read all SNPs in chrom 5
    #         >>> print(bed.read_sparse(np.s_[:,bed.chromosome=='5'], dtype="int8")) # doctest:+NORMALIZE_WHITESPACE
    #         (8, 0)	1
    #         (8, 1)	2
    #         (0, 4)	2
    #         (4, 5)	1
    #         (7, 5)	1
    #         (5, 7)	1
    #         (2, 8)	1
    #         (3, 8)	1
    #         >>> # Read 1st individual (across all SNPs)
    #         >>> print(bed.read_sparse(np.s_[0,:], dtype="int8")) # doctest:+NORMALIZE_WHITESPACE
    #         (0, 8)	2
    #         >>> print(bed.read_sparse(np.s_[::2,:], dtype="int8")) # Read every 2nd individual # doctest:+NORMALIZE_WHITESPACE
    #         (4, 4)    1
    #         (4, 5)    2
    #         (0, 8)    2
    #         (2, 9)    1
    #         (1, 12)   1
    #         >>> # read last and 2nd-to-last individuals and the 15th-from-the-last SNP
    #         >>> print(bed.read_sparse(np.s_[[-1,-2],-15], dtype="int8")) # doctest:+NORMALIZE_WHITESPACE
    #         (1, 0)	2

    #     """
    #     if sparse is None:
    #         raise ImportError(
    #             "The function read_sparse() requires scipy. "
    #             + "Install it with 'pip install --upgrade bed-reader[sparse]'."
    #         )
    #     iid_index_or_slice_etc, sid_index_or_slice_etc = self._split_index(index)

    #     dtype = np.dtype(dtype)

    #     # Similar code in read().
    #     # Later happy with _iid_range and _sid_range or could it be done with
    #     # allocation them?
    #     if self._iid_range is None:
    #         self._iid_range = np.arange(self.iid_count, dtype="intp")
    #     if self._sid_range is None:
    #         self._sid_range = np.arange(self.sid_count, dtype="intp")

    #     iid_index = np.ascontiguousarray(
    #         self._iid_range[iid_index_or_slice_etc],
    #         dtype="intp",
    #     )
    #     sid_index = np.ascontiguousarray(
    #         self._sid_range[sid_index_or_slice_etc], dtype="intp"
    #     )

    #     if (
    #         len(iid_index) > np.iinfo(np.int32).max
    #         or len(sid_index) > np.iinfo(np.int32).max
    #     ):
    #         raise ValueError(
    #             "Too (many Individuals or SNPs (variants) requested. Maximum is {np.iinfo(np.int32).max}."
    #         )

    #     if batch_size is None:
    #         batch_size = round(np.sqrt(len(sid_index)))

    #     num_threads = get_num_threads(
    #         self._num_threads if num_threads is None else num_threads
    #     )

    #     if format == "csc":
    #         order = "F"
    #         indptr = np.zeros(len(sid_index) + 1, dtype=np.int32)
    #     elif format == "csr":
    #         order = "C"
    #         indptr = np.zeros(len(iid_index) + 1, dtype=np.int32)
    #     else:
    #         raise ValueError(f"format '{format}' not known. Expected 'csc' or 'csr'.")

    #     # We init data and indices with zero element arrays to set their dtype.
    #     data = [np.empty(0, dtype=dtype)]
    #     indices = [np.empty(0, dtype=np.int32)]

    #     if self.iid_count > 0 and self.sid_count > 0:
    #         if dtype == np.int8:
    #             reader = read_i8
    #         elif dtype == np.float64:
    #             reader = read_f64
    #         elif dtype == np.float32:
    #             reader = read_f32
    #         else:
    #             raise ValueError(
    #                 f"dtype '{dtype}' not known, only "
    #                 + "'int8', 'float32', and 'float64' are allowed."
    #             )

    #         if format == "csc":
    #             val = np.zeros((len(iid_index), batch_size), order=order, dtype=dtype)
    #             for batch_start in range(0, len(sid_index), batch_size):
    #                 batch_end = batch_start + batch_size
    #                 if batch_end > len(sid_index):
    #                     batch_end = len(sid_index)
    #                     del val
    #                     val = np.zeros(
    #                         (len(iid_index), batch_end - batch_start),
    #                         order=order,
    #                         dtype=dtype,
    #                     )
    #                 batch_slice = np.s_[batch_start:batch_end]
    #                 batch_index = sid_index[batch_slice]

    #                 reader(
    #                     str(self.filepath),
    #                     iid_count=self.iid_count,
    #                     sid_count=self.sid_count,
    #                     is_a1_counted=self.count_A1,
    #                     iid_index=iid_index,
    #                     sid_index=batch_index,
    #                     val=val,
    #                     num_threads=num_threads,
    #                 )

    #                 self.sparsify(
    #                     val, order, iid_index, batch_slice, data, indices, indptr
    #                 )
    #         else:
    #             assert format == "csr"  # real assert
    #             val = np.zeros((batch_size, len(sid_index)), order=order, dtype=dtype)
    #             for batch_start in range(0, len(iid_index), batch_size):
    #                 batch_end = batch_start + batch_size
    #                 if batch_end > len(iid_index):
    #                     batch_end = len(iid_index)
    #                     del val
    #                     val = np.zeros(
    #                         (batch_end - batch_start, len(sid_index)),
    #                         order=order,
    #                         dtype=dtype,
    #                     )

    #                 batch_slice = np.s_[batch_start:batch_end]
    #                 batch_index = iid_index[batch_slice]

    #                 reader(
    #                     str(self.filepath),
    #                     iid_count=self.iid_count,
    #                     sid_count=self.sid_count,
    #                     is_a1_counted=self.count_A1,
    #                     iid_index=batch_index,
    #                     sid_index=sid_index,
    #                     val=val,
    #                     num_threads=num_threads,
    #                 )

    #                 self.sparsify(
    #                     val, order, sid_index, batch_slice, data, indices, indptr
    #                 )

    #     data = np.concatenate(data)
    #     indices = np.concatenate(indices)

    #     if format == "csc":
    #         return sparse.csc_matrix(
    #             (data, indices, indptr), (len(iid_index), len(sid_index))
    #         )
    #     else:
    #         assert format == "csr"  # real assert
    #         return sparse.csr_matrix(
    #             (data, indices, indptr), (len(iid_index), len(sid_index))
    #         )

    # def sparsify(self, val, order, minor_index, batch_slice, data, indices, indptr):
    #     flatten = np.ravel(val, order=order)
    #     nz_indices = np.flatnonzero(flatten).astype(np.int32)
    #     column_indexes = nz_indices // len(minor_index)
    #     counts = np.bincount(
    #         column_indexes, minlength=batch_slice.stop - batch_slice.start
    #     ).astype(np.int32)
    #     counts_with_initial = np.r_[
    #         indptr[batch_slice.start : batch_slice.start + 1], counts
    #     ]

    #     data.append(flatten[nz_indices])
    #     indices.append(np.mod(nz_indices, len(minor_index)))
    #     indptr[1:][batch_slice] = np.cumsum(counts_with_initial)[1:]