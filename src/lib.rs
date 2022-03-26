// Inspired by C++ version by Chris Widmer and Carl Kadie

// See: https://towardsdatascience.com/nine-rules-for-writing-python-extensions-in-rust-d35ea3a4ec29?sk=f8d808d5f414154fdb811e4137011437
// for an article on how this project uses Rust to create a Python extension.

//! # bed-reader
//!
//! Read and write the PLINK BED format, simply and efficiently.
//!
//! Features:
//!   * Fast multi-threaded engine.
//!   * Supports many indexing methods. Slice data by individuals (samples) and/or SNPs (variants).
//!   * Used by Python packages [PySnpTools], [FaST-LMM], and [PyStatGen].
//!   * Supports [PLINK 1.9].
//!
//! [PySnpTools]: https://github.com/fastlmm/PySnpTools
//! [FaST-LMM]: https://github.com/fastlmm/FaST-LMM
//! [PyStatGen]: https://github.com/pystatgen
//! [PLINK 1.9]: https://www.cog-genomics.org/plink2/formats
//!
//! ## Usage
//!
//! Read genotype data from a .bed file.
//!
//! ```
//! use ndarray as nd;
//! use bed_reader::Bed;
//! use bed_reader::assert_eq_nan;
//!
//! let file_name = "bed_reader/tests/data/small.bed";
//! let mut bed = Bed::new(file_name)?;
//! let val = bed.read::<f64>()?;
//!
//! assert_eq_nan(
//!     &val,
//!     &nd::array![
//!         [1.0, 0.0, f64::NAN, 0.0],
//!         [2.0, 0.0, f64::NAN, 2.0],
//!         [0.0, 1.0, 2.0, 0.0]
//!     ],
//! );
//! # use bed_reader::BedErrorPlus;
//! # Ok::<(), BedErrorPlus>(())
//! ```
//!
//! Read every second individual and SNPs (variants) from 20 to 30.
//!
//! ```
//! # use ndarray as nd;
//! # use bed_reader::Bed;
//! # use bed_reader::assert_eq_nan;
//! use bed_reader::ReadOptions;
//! use ndarray::s;
//!
//! let file_name2 = "bed_reader/tests/data/some_missing.bed";
//! let mut bed2 = Bed::new(file_name2)?;
//! let val2 = ReadOptions::builder()
//!     .f64()
//!     .iid_index(s![..;2])
//!     .sid_index(20..30)
//!     .read(&mut bed2)?;
//!
//! assert!(val2.dim() == (50, 10));
//! # use bed_reader::BedErrorPlus;
//! # Ok::<(), BedErrorPlus>(())
//! ```
//!
//! List the first 5 individual (sample) ids, the first 5 SNP (variant) ids,
//! and every unique chromosome. Then, read every value in chromosome 5.
//!
//! ```
//! # use ndarray as nd;
//! # use bed_reader::Bed;
//! # use bed_reader::assert_eq_nan;
//! # use bed_reader::ReadOptions;
//! # use ndarray::s;
//! # let file_name2 = "bed_reader/tests/data/some_missing.bed";
//! use std::collections::HashSet;
//!
//! let mut bed3 = Bed::new(file_name2)?;
//! println!("{:?}", bed3.iid()?.slice(s![..5]));
//! println!("{:?}", bed3.sid()?.slice(s![..5]));
//! println!("{:?}", bed3.chromosome()?.iter().collect::<HashSet<_>>());
//! let val3 = ReadOptions::builder()
//!     .sid_index(bed3.chromosome()?.map(|elem| elem == "5"))
//!     .f64()
//!     .read(&mut bed3)?;
//! assert!(val3.dim() == (100, 6));
//! # use bed_reader::BedErrorPlus;
//! # Ok::<(), BedErrorPlus>(())
//! ```
//! *This outputs:*
//! ```text
//! ["iid_0", "iid_1", "iid_2", "iid_3", "iid_4"], shape=[5], strides=[1], layout=CFcf (0xf), const ndim=1
//! ["sid_0", "sid_1", "sid_2", "sid_3", "sid_4"], shape=[5], strides=[1], layout=CFcf (0xf), const ndim=1
//! {"12", "10", "4", "8", "19", "21", "9", "15", "6", "16", "13", "7", "17", "18", "1", "22", "11", "2", "20", "3", "5", "14"}
//! ```
//!
//!  ## Project Links
//!  * cmkDocumentation
//!  * Questions to [fastlmm-dev@python.org](mailto://fastlmm-dev@python.org)
//!  * [Source code](https://github.com/fastlmm/bed-reader)
//!  * [Bug Reports](https://github.com/fastlmm/bed-reader/issues)
//!  * [Mailing list](https://mail.python.org/mailman3/lists/fastlmm-user.python.org)
//!  * [Project Website](https://fastlmm.github.io/)
//!
//! ## Summary
//! ### Open for Reading Genotype Data and Metadata
//! * [`Bed::new`](struct.Bed.html#method.new) - Open a PLINK .bed file for reading. Does not support options.
//! * [`Bed::builder`](struct.Bed.html#method.builder) - Open a PLINK .bed file for reading. Supports options.
//! ### Read Genotype Data
//! * [`Bed::read`](struct.Bed.html#method.read) - Read all genotype data.
//! * [`ReadOptions::builder`](struct.ReadOptions.html#method.builder) - Read genotype data. Supports selection and options.
//! * cmk[`ReadOptionsBuilder::read_and_fill`](struct.ReadOptionsBuilder.html#method.read_and_fill) - Fill an existing array with genotype data. Supports selection and options.
//!
//! *Alternatives:*
//! * [`Bed::read_with_options`](struct.Bed.html#method.read_with_options) - Read genotype data. Supports selection and options.
//! * [`Bed::read_and_fill`](struct.Bed.html#method.read_and_fill) - Fill an existing array with genotype data.
//! * [`Bed::read_and_fill_with_options`](struct.Bed.html#method.read_and_fill_with_options) - Fill an existing array with genotype data. Supports selection and options.
//! ### Write Genotype Data
//! * [`WriteOptions::builder`](struct.WriteOptions.html#method.builder) - Write values to a file in PLINK .bed format. Supports metadata and options.
//!
//! *Alternatives:*
//! * [`Bed::write_with_options`](struct.Bed.html#method.write_with_options) - Write genotype data with options.
//! * [`Bed::write`](struct.Bed.html#method.write) - Write genotype data with default metadata.
//! ### Read Metadata
//! * [`Bed::iid_count`](struct.Bed.html#method.iid_count) - Number of individuals (samples)
//! * [`Bed::sid_count`](struct.Bed.html#method.sid_count) - Number of SNPs (variants)
//! * [`Bed::shape`](struct.Bed.html#method.shape) - Number of individuals (samples) and SNPs (variants)
//! * [`Bed::fid`](struct.Bed.html#method.fid) - Family id of each individual (sample)
//! * [`Bed::iid`](struct.Bed.html#method.iid) - Individual id of each individual (sample)
//! * [`Bed::father`](struct.Bed.html#method.father) - Father id of each individual (sample)
//! * [`Bed::mother`](struct.Bed.html#method.mother) - Mother id of each individual (sample)
//! * [`Bed::sex`](struct.Bed.html#method.sex) - Sex of each individual (sample)
//! * [`Bed::pheno`](struct.Bed.html#method.pheno) - A phenotype for each individual (sample) (seldom used)
//! * [`Bed::chromosome`](struct.Bed.html#method.chromosome) - Chromosome of each SNP (variant)
//! * [`Bed::sid`](struct.Bed.html#method.sid) - SNP Id of each SNP (variant)
//! * [`Bed::cm_position`](struct.Bed.html#method.cm_position) - Centimorgan position of each SNP (variant)
//! * [`Bed::bp_position`](struct.Bed.html#method.bp_position) - Base-pair position of each SNP (variant)
//! * [`Bed::allele1`](struct.Bed.html#method.allele1) - First allele of each SNP (variant)
//! * [`Bed::allele2`](struct.Bed.html#method.allele2) - Second allele of each SNP (variant)
//! * [`Bed::metadata`](struct.Bed.html#method.metadata) - All the metadata returned as a [`struct.Metadata`](struct.Metadata.html)
//!
//! ## Utilities cmk 0 may not need this listed below
//! * [`assert_eq_nan`](fn.assert_eq_nan.html) - Assert that two arrays are equal, ignoring NaN values.
//! * [`allclose`](fn.allclose.html) - Assert that two array views are nearly equal, optionally ignoring NaN values.

// !!!cmk later Environment  variables

mod python_module;
mod tests;
mod tests_api;

use core::fmt::Debug;
use derive_builder::Builder;
use nd::ShapeBuilder;
use ndarray as nd;
use std::fs::{self};
use std::io::Write;
use std::ops::{Bound, Range, RangeBounds, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive};
use std::{
    env,
    fs::File,
    io::{BufRead, BufReader, BufWriter},
    ops::RangeFull,
    path::{Path, PathBuf},
};
use temp_testdir::TempDir;
// !!! might want to use this instead use typed_builder::TypedBuilder;

use byteorder::{LittleEndian, ReadBytesExt};
use dpc_pariter::{scope, IteratorExt};
use num_traits::{abs, Float, FromPrimitive, Signed, ToPrimitive};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rayon::{iter::ParallelBridge, ThreadPoolBuildError};
use statrs::distribution::{Beta, Continuous};
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::num::{ParseFloatError, ParseIntError};
use std::ops::AddAssign;
use std::ops::{Div, Sub};
use thiserror::Error;

const BED_FILE_MAGIC1: u8 = 0x6C; // 0b01101100 or 'l' (lowercase 'L')
const BED_FILE_MAGIC2: u8 = 0x1B; // 0b00011011 or <esc>
const CB_HEADER_U64: u64 = 3;
const CB_HEADER_USIZE: usize = 3;

// About ndarray
//  https://docs.rs/ndarray/0.14.0/ndarray/parallel/index.html
//  https://rust-lang-nursery.github.io/rust-cookbook/concurrency/parallel.html
//  https://github.com/rust-ndarray/ndarray/blob/master/README-quick-start.md
//  https://datacrayon.com/posts/programming/rust-notebooks/multidimensional-arrays-and-operations-with-ndarray
//  https://docs.rs/ndarray/0.14.0/ndarray/doc/ndarray_for_numpy_users/index.html
//  https://docs.rs/ndarray-npy
//  https://rust-lang-nursery.github.io/rust-cookbook/science/mathematics/linear_algebra.html

/// BedError enumerates all possible errors returned by this library.
// Based on `<https://nick.groenen.me/posts/rust-error-handling/#the-library-error-type>`
#[derive(Error, Debug)]
pub enum BedErrorPlus {
    #[error(transparent)]
    BedError(#[from] BedError),

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    ThreadPoolError(#[from] ThreadPoolBuildError),

    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),

    #[error(transparent)]
    ReadOptionsBuilderError(#[from] ReadOptionsBuilderError),

    #[error(transparent)]
    UninitializedFieldError(#[from] ::derive_builder::UninitializedFieldError),

    #[error(transparent)]
    ParseFloatError(#[from] ParseFloatError),
}
// https://docs.rs/thiserror/1.0.23/thiserror/
#[derive(Error, Debug, Clone)]
pub enum BedError {
    #[error("Ill-formed BED file. BED file header is incorrect or length is wrong. '{0}'")]
    IllFormed(String),

    #[error(
        "Ill-formed BED file. BED file header is incorrect. Expected mode to be 0 or 1. '{0}'"
    )]
    BadMode(String),

    #[error("Attempt to write illegal value to BED file. Only 0,1,2,missing allowed. '{0}'")]
    BadValue(String),

    #[error("Multithreading resulted in panic(s)")]
    PanickedThread(),

    #[error("No individual observed for the SNP.")]
    NoIndividuals,

    #[error("Illegal SNP mean.")]
    IllegalSnpMean,

    #[error("Index to individual larger than the number of individuals. (Index value {0})")]
    IidIndexTooBig(usize),

    #[error("Index to SNP larger than the number of SNPs. (Index value {0})")]
    SidIndexTooBig(usize),

    #[error("Length of iid_index ({0}) and sid_index ({1}) must match dimensions of output array ({2},{3}).")]
    IndexMismatch(usize, usize, usize, usize),

    #[error("Indexes ({0},{1}) too big for files")]
    IndexesTooBigForFiles(usize, usize),

    #[error("Subset: length of iid_index ({0}) and sid_index ({1}) must match dimensions of output array ({2},{3}).")]
    SubsetMismatch(usize, usize, usize, usize),

    #[error("Cannot convert beta values to/from float 64")]
    CannotConvertBetaToFromF64,

    #[error("Cannot create Beta Dist with given parameters ({0},{1})")]
    CannotCreateBetaDist(f64, f64),

    #[error("Cannot use skipped metadata '{0}'")]
    CannotUseSkippedMetadata(String),

    #[error("Index starts at {0} but ends at {1}")]
    StartGreaterThanEnd(usize, usize),

    #[error("Step of zero not allowed")]
    StepZero,

    #[error("Index starts at {0} but count is {1}")]
    StartGreaterThanCount(usize, usize),

    #[error("Index ends at {0} but count is {1}")]
    EndGreaterThanCount(usize, usize),

    #[error("Adding new axis not allowed")]
    NewAxis,

    #[error("Expect 1-D NDArray SliceInfo")]
    NdSliceInfoNot1D,

    #[error("Expect {0} fields but find only {1} in '{2}'")]
    MetadataFieldCount(usize, usize, String),

    #[error("{0}_count values of {1} and {2} are inconsistent")]
    InconsistentCount(String, usize, usize),

    #[error("Expect bool arrays and vectors to be length {0}, not {1}")]
    BoolArrayVectorWrongLength(usize, usize),

    #[error("Expect ndarray of shape ({0}, {1}), but found shape ({2}, {3})")]
    InvalidShape(usize, usize, usize, usize),
}

// Trait alias
pub trait BedVal: Copy + Default + From<i8> + Debug + Sync + Send + Missing + PartialEq {}
impl<T> BedVal for T where T: Copy + Default + From<i8> + Debug + Sync + Send + Missing + PartialEq {}

fn create_pool(num_threads: usize) -> Result<rayon::ThreadPool, BedErrorPlus> {
    match rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
    {
        Err(e) => Err(e.into()),
        Ok(pool) => Ok(pool),
    }
}

//#!!!cmk later hide this from the docs
#[allow(clippy::too_many_arguments)]
fn read_no_alloc<TVal: BedVal, P: AsRef<Path>>(
    path: P,
    iid_count: usize,
    sid_count: usize,
    is_a1_counted: bool,
    iid_index: &[usize],
    sid_index: &[usize],
    missing_value: TVal,
    num_threads: usize,
    val: &mut nd::ArrayViewMut2<'_, TVal>, //mutable slices additionally allow to modify elements. But slices cannot grow - they are just a view into some vector.
) -> Result<(), BedErrorPlus> {
    let path_buf = PathBuf::from(path.as_ref());

    create_pool(num_threads)?.install(|| {
        let (buf_reader, bytes_vector) = open_and_check(&path_buf)?;

        match bytes_vector[2] {
            0 => {
                // We swap 'iid' and 'sid' and then reverse the axes.
                let mut val_t = val.view_mut().reversed_axes();
                internal_read_no_alloc(
                    buf_reader,
                    path_buf,
                    sid_count,
                    iid_count,
                    is_a1_counted,
                    sid_index,
                    iid_index,
                    missing_value,
                    &mut val_t,
                )
            }
            1 => internal_read_no_alloc(
                buf_reader,
                path_buf,
                iid_count,
                sid_count,
                is_a1_counted,
                iid_index,
                sid_index,
                missing_value,
                val,
            ),
            _ => Err(BedError::BadMode(path_buf.display().to_string()).into()),
        }
    })?;
    Ok(())
}

fn open_and_check(path_buf: &PathBuf) -> Result<(BufReader<File>, Vec<u8>), BedErrorPlus> {
    let mut buf_reader = BufReader::new(File::open(path_buf)?);
    let mut bytes_vector: Vec<u8> = vec![0; CB_HEADER_USIZE];
    buf_reader.read_exact(&mut bytes_vector)?;
    if (BED_FILE_MAGIC1 != bytes_vector[0]) || (BED_FILE_MAGIC2 != bytes_vector[1]) {
        return Err(BedError::IllFormed(path_buf.display().to_string()).into());
    }
    Ok((buf_reader, bytes_vector))
}

trait Max {
    fn max() -> Self;
}

impl Max for u8 {
    fn max() -> u8 {
        std::u8::MAX
    }
}

impl Max for u64 {
    fn max() -> u64 {
        std::u64::MAX
    }
}

pub trait Missing {
    fn missing() -> Self;
}

impl Missing for f64 {
    fn missing() -> Self {
        f64::NAN
    }
}

impl Missing for f32 {
    fn missing() -> Self {
        f32::NAN
    }
}

impl Missing for i8 {
    fn missing() -> Self {
        -127i8
    }
}

// We make this generic instead of u64, so that we can test it via u8
fn try_div_4<T: Max + TryFrom<usize> + Sub<Output = T> + Div<Output = T> + Ord>(
    in_iid_count: usize,
    in_sid_count: usize,
    cb_header: T,
) -> Result<(usize, T), BedErrorPlus> {
    // 4 genotypes per byte so round up without overflow
    let in_iid_count_div4 = if in_iid_count > 0 {
        (in_iid_count - 1) / 4 + 1
    } else {
        0
    };
    let in_iid_count_div4_t = match T::try_from(in_iid_count_div4) {
        Ok(v) => v,
        Err(_) => return Err(BedError::IndexesTooBigForFiles(in_iid_count, in_sid_count).into()),
    };
    let in_sid_count_t = match T::try_from(in_sid_count) {
        Ok(v) => v,
        Err(_) => return Err(BedError::IndexesTooBigForFiles(in_iid_count, in_sid_count).into()),
    };

    let m: T = Max::max(); // Don't know how to move this into the next line.
    if in_sid_count > 0 && (m - cb_header) / in_sid_count_t < in_iid_count_div4_t {
        return Err(BedError::IndexesTooBigForFiles(in_iid_count, in_sid_count).into());
    }

    Ok((in_iid_count_div4, in_iid_count_div4_t))
}

// !!!cmk later could iid_index and sid_index be ExpectSizeIterator<Item=usize>?
#[allow(clippy::too_many_arguments)]
fn internal_read_no_alloc<TVal: BedVal, P: AsRef<Path>>(
    mut buf_reader: BufReader<File>,
    path: P,
    in_iid_count: usize,
    in_sid_count: usize,
    is_a1_counted: bool,
    iid_index: &[usize],
    sid_index: &[usize],
    missing_value: TVal,
    out_val: &mut nd::ArrayViewMut2<'_, TVal>, //mutable slices additionally allow to modify elements. But slices cannot grow - they are just a view into some vector.
) -> Result<(), BedErrorPlus> {
    // Find the largest in_iid_i (if any) and check its size.
    if let Some(in_max_iid_i) = iid_index.iter().max() {
        if *in_max_iid_i >= in_iid_count {
            return Err(BedError::IidIndexTooBig(*in_max_iid_i).into());
        }
    }

    let (in_iid_count_div4, in_iid_count_div4_u64) =
        try_div_4(in_iid_count, in_sid_count, CB_HEADER_U64)?;

    let from_two_bits_to_value = set_up_two_bits_to_value(is_a1_counted, missing_value);

    // "as" and math is safe because of early checks
    let file_len = buf_reader.seek(SeekFrom::End(0))?;
    let file_len2 = in_iid_count_div4_u64 * (in_sid_count as u64) + CB_HEADER_U64;
    if file_len != file_len2 {
        return Err(BedError::IllFormed(PathBuf::from(path.as_ref()).display().to_string()).into());
    }

    // See https://morestina.net/blog/1432/parallel-stream-processing-with-rayon
    // Possible optimization: We could try to read only the iid info needed
    // Possible optimization: We could read snp in their input order instead of their output order
    sid_index
        .iter()
        // Read all the iid info for one snp from the disk
        .map(|in_sid_i| {
            if in_sid_i >= &in_sid_count {
                return Err(BedErrorPlus::BedError(BedError::SidIndexTooBig(*in_sid_i)));
            }
            let mut bytes_vector: Vec<u8> = vec![0; in_iid_count_div4];
            let pos: u64 = (*in_sid_i as u64) * in_iid_count_div4_u64 + CB_HEADER_U64; // "as" and math is safe because of early checks
            buf_reader.seek(SeekFrom::Start(pos))?;
            buf_reader.read_exact(&mut bytes_vector)?;
            Ok(bytes_vector)
        })
        // Zip in the column of the output array
        .zip(out_val.axis_iter_mut(nd::Axis(1)))
        // In parallel, decompress the iid info and put it in its column
        .par_bridge() // This seems faster that parallel zip
        .try_for_each(|(bytes_vector_result, mut col)| {
            match bytes_vector_result {
                Err(e) => Err(e),
                Ok(bytes_vector) => {
                    for (out_iid_i, in_iid_i) in iid_index.iter().enumerate() {
                        // Possible optimization: We could pre-compute the conversion, the division, the mod, and the multiply*2
                        let i_div_4 = in_iid_i / 4;
                        let i_mod_4 = in_iid_i % 4;
                        let genotype_byte: u8 = (bytes_vector[i_div_4] >> (i_mod_4 * 2)) & 0x03;
                        col[out_iid_i] = from_two_bits_to_value[genotype_byte as usize];
                    }
                    Ok(())
                }
            }
        })?;

    Ok(())
}

fn set_up_two_bits_to_value<TVal: From<i8>>(count_a1: bool, missing_value: TVal) -> [TVal; 4] {
    let homozygous_primary_allele = TVal::from(0); // Major Allele
    let heterozygous_allele = TVal::from(1);
    let homozygous_secondary_allele = TVal::from(2); // Minor Allele

    let from_two_bits_to_value;
    if count_a1 {
        from_two_bits_to_value = [
            homozygous_secondary_allele, // look-up 0
            missing_value,               // look-up 1
            heterozygous_allele,         // look-up 2
            homozygous_primary_allele,   // look-up 3
        ];
    } else {
        from_two_bits_to_value = [
            homozygous_primary_allele,   // look-up 0
            missing_value,               // look-up 1
            heterozygous_allele,         // look-up 2
            homozygous_secondary_allele, // look-up 3
        ];
    }
    from_two_bits_to_value
}

// Thanks to Dawid for his dpc-pariter library that makes this function scale.
// https://dpc.pw/adding-parallelism-to-your-rust-iterators
fn write_val<S, TVal, P>(
    path: P,
    val: &nd::ArrayBase<S, nd::Ix2>,
    is_a1_counted: bool,
    missing: TVal,
    num_threads: usize,
) -> Result<(), BedErrorPlus>
where
    S: nd::Data<Elem = TVal>,
    TVal: BedVal,
    P: AsRef<Path>,
{
    let (iid_count, sid_count) = val.dim();

    // 4 genotypes per byte so round up
    let (iid_count_div4, _) = try_div_4(iid_count, sid_count, CB_HEADER_U64)?;

    // We create and write to a file.
    // If there is an error, we will delete it.
    if let Err(e) = write_internal(
        &path,
        iid_count_div4,
        val,
        is_a1_counted,
        missing,
        num_threads,
    ) {
        // Clean up the file
        let _ = fs::remove_file(path);
        Err(e)
    } else {
        Ok(())
    }
}

// https://www.reddit.com/r/rust/comments/mo4s8e/difference_between_reference_and_view_in_ndarray/
fn write_internal<S, TVal, P>(
    path: P,
    iid_count_div4: usize,
    //val: &nd::ArrayView2<'_, TVal>,
    val: &nd::ArrayBase<S, nd::Ix2>,
    is_a1_counted: bool,
    missing: TVal,
    num_threads: usize,
) -> Result<(), BedErrorPlus>
where
    S: nd::Data<Elem = TVal>,
    TVal: BedVal,
    P: AsRef<Path>,
{
    let mut writer = BufWriter::new(File::create(&path)?);
    writer.write_all(&[BED_FILE_MAGIC1, BED_FILE_MAGIC2, 0x01])?;

    #[allow(clippy::eq_op)]
    let use_nan = missing != missing; // generic NAN test
    let zero_code = if is_a1_counted { 3u8 } else { 0u8 };
    let two_code = if is_a1_counted { 0u8 } else { 3u8 };

    let homozygous_primary_allele = TVal::from(0); // Major Allele
    let heterozygous_allele = TVal::from(1);
    let homozygous_secondary_allele = TVal::from(2); // Minor Allele

    let path_buf = PathBuf::from(path.as_ref());
    scope(|scope| {
        val.axis_iter(nd::Axis(1))
            .parallel_map_scoped(scope, {
                move |column| {
                    // Convert each column into a bytes_vector
                    let mut bytes_vector: Vec<u8> = vec![0; iid_count_div4]; // inits to 0
                    for (iid_i, &v0) in column.iter().enumerate() {
                        #[allow(clippy::eq_op)]
                        let genotype_byte = if v0 == homozygous_primary_allele {
                            zero_code
                        } else if v0 == heterozygous_allele {
                            2
                        } else if v0 == homozygous_secondary_allele {
                            two_code
                        //                    v0 !=v0 is generic NAN check
                        } else if (use_nan && v0 != v0) || (!use_nan && v0 == missing) {
                            1
                        } else {
                            return Err(BedError::BadValue(path_buf.display().to_string()));
                        };
                        // Possible optimization: We could pre-compute the conversion, the division, the mod, and the multiply*2
                        let i_div_4 = iid_i / 4;
                        let i_mod_4 = iid_i % 4;
                        bytes_vector[i_div_4] |= genotype_byte << (i_mod_4 * 2);
                    }
                    Ok(bytes_vector)
                }
            })
            .threads(num_threads)
            .try_for_each(|bytes_vector| {
                // Write the bytes vector, they must be in order.
                writer.write_all(&bytes_vector?)?;
                Ok(())
            })
    })
    .map_err(|_e| BedError::PanickedThread())?
}

fn count_lines<P: AsRef<Path>>(path: P) -> Result<usize, BedErrorPlus> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let count = reader.lines().count();
    Ok(count)
}

fn matrix_subset_no_alloc<
    TIn: Copy + Default + Debug + Sync + Send + Sized,
    TOut: Copy + Default + Debug + Sync + Send + From<TIn>,
>(
    in_val: &nd::ArrayView3<'_, TIn>,
    iid_index: &[usize],
    sid_index: &[usize],
    out_val: &mut nd::ArrayViewMut3<'_, TOut>,
) -> Result<(), BedErrorPlus> {
    let out_iid_count = iid_index.len();
    let out_sid_count = sid_index.len();
    let did_count = in_val.dim().2;

    if (out_iid_count, out_sid_count, did_count) != out_val.dim() {
        return Err(BedError::SubsetMismatch(
            out_iid_count,
            out_sid_count,
            out_val.dim().0,
            out_val.dim().1,
        )
        .into());
    }

    // If output is F-order (or in general if iid stride is no more than sid_stride)
    if out_val.stride_of(nd::Axis(0)) <= out_val.stride_of(nd::Axis(1)) {
        // (No error are possible in the par_azip, so don't have to collect and check them)
        nd::par_azip!((mut out_col in out_val.axis_iter_mut(nd::Axis(1)),
                    in_sid_i_pr in sid_index) {
            let in_col = in_val.index_axis(nd::Axis(1), *in_sid_i_pr);
            for did_i in 0..did_count
            {
                for (out_iid_i, in_iid_i_ptr) in iid_index.iter().enumerate() {
                    out_col[(out_iid_i,did_i)] = in_col[(*in_iid_i_ptr,did_i)].into();
                }
            }
        });
        Ok(())
    } else {
        //If output is C-order, transpose input and output and recurse
        let in_val_t = in_val.view().permuted_axes([1, 0, 2]);
        let mut out_val_t = out_val.view_mut().permuted_axes([1, 0, 2]);
        matrix_subset_no_alloc(&in_val_t, sid_index, iid_index, &mut out_val_t)
    }
}

enum Dist {
    Unit,
    Beta { a: f64, b: f64 },
}

fn impute_and_zero_mean_snps<
    T: Default + Copy + Debug + Sync + Send + Float + ToPrimitive + FromPrimitive,
>(
    val: &mut nd::ArrayViewMut2<'_, T>,
    dist: Dist,
    apply_in_place: bool,
    use_stats: bool,
    stats: &mut nd::ArrayViewMut2<'_, T>,
) -> Result<(), BedErrorPlus> {
    let two = T::one() + T::one();

    // If output is F-order (or in general if iid stride is no more than sid_stride)
    if val.stride_of(nd::Axis(0)) <= val.stride_of(nd::Axis(1)) {
        let result_list = nd::Zip::from(val.axis_iter_mut(nd::Axis(1)))
            .and(stats.axis_iter_mut(nd::Axis(0)))
            .par_map_collect(|mut col, mut stats_row| {
                _process_sid(
                    &mut col,
                    apply_in_place,
                    use_stats,
                    &mut stats_row,
                    &dist,
                    two,
                )
            });

        // Check the result list for errors
        result_list
            .iter()
            .par_bridge()
            .try_for_each(|x| (*x).clone())?;

        Ok(())
    } else {
        //If C-order
        _process_all_iids(val, apply_in_place, use_stats, stats, dist, two)
    }
}

// !!!cmk later move the other fast-lmm functions into their own package
fn find_factor<T: Default + Copy + Debug + Sync + Send + Float + ToPrimitive + FromPrimitive>(
    dist: &Dist,
    mean_s: T,
    std: T,
) -> Result<T, BedError> {
    if let Dist::Beta { a, b } = dist {
        // Try to create a beta dist
        let beta_dist = if let Ok(beta_dist) = Beta::new(*a, *b) {
            beta_dist
        } else {
            return Err(BedError::CannotCreateBetaDist(*a, *b));
        };

        // Try to an f64 maf
        let mut maf = if let Some(mean_u64) = mean_s.to_f64() {
            mean_u64 / 2.0
        } else {
            return Err(BedError::CannotConvertBetaToFromF64);
        };
        if maf > 0.5 {
            maf = 1.0 - maf;
        }

        // Try to put the maf in the beta dist
        if let Some(b) = T::from_f64(beta_dist.pdf(maf)) {
            Ok(b)
        } else {
            Err(BedError::CannotConvertBetaToFromF64)
        }
    } else {
        Ok(T::one() / std)
    }
}

fn _process_sid<T: Default + Copy + Debug + Sync + Send + Float + ToPrimitive + FromPrimitive>(
    col: &mut nd::ArrayViewMut1<'_, T>,
    apply_in_place: bool,
    use_stats: bool,
    stats_row: &mut nd::ArrayViewMut1<'_, T>,
    dist: &Dist,
    two: T,
) -> Result<(), BedError> {
    if !use_stats {
        let mut n_observed = T::zero();
        let mut sum_s = T::zero(); // the sum of a SNP over all observed individuals
        let mut sum2_s = T::zero(); // the sum of the squares of the SNP over all observed individuals

        for iid_i in 0..col.len() {
            let v = col[iid_i];
            if !v.is_nan() {
                sum_s = sum_s + v;
                sum2_s = sum2_s + v * v;
                n_observed = n_observed + T::one();
            }
        }
        if n_observed < T::one() {
            //LATER make it work (in some form) for n of 0
            return Err(BedError::NoIndividuals);
        }
        let mean_s = sum_s / n_observed; //compute the mean over observed individuals for the current SNP
        let mean2_s: T = sum2_s / n_observed; //compute the mean of the squared SNP

        if mean_s.is_nan()
            || (matches!(dist, Dist::Beta { a: _, b: _ })
                && ((mean_s > two) || (mean_s < T::zero())))
        {
            return Err(BedError::IllegalSnpMean);
        }

        let variance: T = mean2_s - mean_s * mean_s; //By the Cauchy Schwartz inequality this should always be positive

        let mut std = variance.sqrt();
        if std.is_nan() || std <= T::zero() {
            // All "SNPs" have the same value (aka SNC)
            std = T::infinity(); //SNCs are still meaning full in QQ plots because they should be thought of as SNPs without enough data.
        }

        stats_row[0] = mean_s;
        stats_row[1] = std;
    }

    if apply_in_place {
        {
            let mean_s = stats_row[0];
            let std = stats_row[1];
            let is_snc = std.is_infinite();

            let factor = find_factor(dist, mean_s, std)?;

            for iid_i in 0..col.len() {
                //check for Missing (NAN) or SNC
                if col[iid_i].is_nan() || is_snc {
                    col[iid_i] = T::zero();
                } else {
                    col[iid_i] = (col[iid_i] - mean_s) * factor;
                }
            }
        }
    }
    Ok(())
}

fn _process_all_iids<
    T: Default + Copy + Debug + Sync + Send + Float + ToPrimitive + FromPrimitive,
>(
    val: &mut nd::ArrayViewMut2<'_, T>,
    apply_in_place: bool,
    use_stats: bool,
    stats: &mut nd::ArrayViewMut2<'_, T>,
    dist: Dist,
    two: T,
) -> Result<(), BedErrorPlus> {
    let sid_count = val.dim().1;

    if !use_stats {
        // O(iid_count * sid_count)
        // Serial that respects C-order is 3-times faster than parallel that doesn't
        // So we parallelize the inner loop instead of the outer loop
        let mut n_observed_array = nd::Array1::<T>::zeros(sid_count);
        let mut sum_s_array = nd::Array1::<T>::zeros(sid_count); //the sum of a SNP over all observed individuals
        let mut sum2_s_array = nd::Array1::<T>::zeros(sid_count); //the sum of the squares of the SNP over all observed individuals
        for row in val.axis_iter(nd::Axis(0)) {
            nd::par_azip!((&v in row,
                n_observed_ptr in &mut n_observed_array,
                sum_s_ptr in &mut sum_s_array,
                sum2_s_ptr in &mut sum2_s_array
            )
                if !v.is_nan() {
                    *n_observed_ptr = *n_observed_ptr + T::one();
                    *sum_s_ptr = *sum_s_ptr + v;
                    *sum2_s_ptr = *sum2_s_ptr + v * v;
                }
            );
        }

        // O(sid_count)
        let mut result_list: Vec<Result<(), BedError>> = vec![Ok(()); sid_count];
        nd::par_azip!((mut stats_row in stats.axis_iter_mut(nd::Axis(0)),
                &n_observed in &n_observed_array,
                &sum_s in &sum_s_array,
                &sum2_s in &sum2_s_array,
                result_ptr in &mut result_list)
        {
            if n_observed < T::one() {
                *result_ptr = Err(BedError::NoIndividuals);
                return;
            }
            let mean_s = sum_s / n_observed; //compute the mean over observed individuals for the current SNP
            let mean2_s: T = sum2_s / n_observed; //compute the mean of the squared SNP

            if mean_s.is_nan()
                || (matches!(dist, Dist::Beta { a:_, b:_ }) && ((mean_s > two) || (mean_s < T::zero())))
            {
                *result_ptr = Err(BedError::IllegalSnpMean);
                return;
            }

            let variance: T = mean2_s - mean_s * mean_s; //By the Cauchy Schwartz inequality this should always be positive
            let mut std = variance.sqrt();
            if std.is_nan() || std <= T::zero() {
                // All "SNPs" have the same value (aka SNC)
                std = T::infinity(); //SNCs are still meaning full in QQ plots because they should be thought of as SNPs without enough data.
            }
            stats_row[0] = mean_s;
            stats_row[1] = std;
        });
        // Check the result list for errors
        result_list.par_iter().try_for_each(|x| (*x).clone())?;
    }

    if apply_in_place {
        // O(sid_count)
        let mut factor_array = nd::Array1::<T>::zeros(stats.dim().0);

        stats
            .axis_iter_mut(nd::Axis(0))
            .zip(&mut factor_array)
            .par_bridge()
            .try_for_each(|(stats_row, factor_ptr)| {
                match find_factor(&dist, stats_row[0], stats_row[1]) {
                    Err(e) => Err(e),
                    Ok(factor) => {
                        *factor_ptr = factor;
                        Ok(())
                    }
                }
            })?;

        // O(iid_count * sid_count)
        nd::par_azip!((mut row in val.axis_iter_mut(nd::Axis(0)))
        {
            for sid_i in 0..row.len() {
                //check for Missing (NAN) or SNC
                if row[sid_i].is_nan() || stats[(sid_i, 1)].is_infinite() {
                    row[sid_i] = T::zero();
                } else {
                    row[sid_i] = (row[sid_i] - stats[(sid_i, 0)]) * factor_array[sid_i];
                }
            }
        });
    }
    Ok(())
}

fn file_b_less_aatbx<P: AsRef<Path>>(
    a_filename: P,
    offset: u64,
    iid_count: usize,
    b1: &mut nd::ArrayViewMut2<'_, f64>,
    aatb: &mut nd::ArrayViewMut2<'_, f64>,
    atb: &mut nd::ArrayViewMut2<'_, f64>,
    log_frequency: usize,
) -> Result<(), BedErrorPlus> {
    //speed idea from C++:
    //Are copies really needed?
    //is F, vc C order the best?
    //would bigger snp blocks be better

    let (a_sid_count, b_sid_count) = atb.dim();
    if log_frequency > 0 {
        println!(
            "file_b_less_aatbx: iid_count={}, {}x{} output",
            iid_count, a_sid_count, b_sid_count
        );
    };

    // Open the file and move to the starting sid
    let mut buf_reader = BufReader::new(File::open(a_filename)?);
    buf_reader.seek(SeekFrom::Start(offset))?;

    let mut sid_reuse = vec![f64::NAN; iid_count];
    for (a_sid_index, mut atb_row) in atb.axis_iter_mut(nd::Axis(0)).enumerate() {
        if log_frequency > 0 && a_sid_index % log_frequency == 0 {
            println!(
                "   working on train_sid_index={} of {} (iid_count={}, b_sid_count={})",
                a_sid_index, a_sid_count, iid_count, b_sid_count
            );
        }

        buf_reader.read_f64_into::<LittleEndian>(&mut sid_reuse)?;

        nd::par_azip!(
            (mut atb_element in atb_row.axis_iter_mut(nd::Axis(0)),
            b1_col in b1.axis_iter(nd::Axis(1)),
            mut aatb_col in aatb.axis_iter_mut(nd::Axis(1)))
        {
            let mut atbi = 0.0;
            for iid_index in 0..iid_count {
                atbi += sid_reuse[iid_index] * b1_col[iid_index];
            }
            atb_element[()] = atbi;
            for iid_index in 0..iid_count {
                aatb_col[iid_index] -= sid_reuse[iid_index] * atbi;
            }
        });
    }
    Ok(())
}

fn read_into_f64(src: &mut BufReader<File>, dst: &mut [f64]) -> std::io::Result<()> {
    src.read_f64_into::<LittleEndian>(dst)
}

fn read_into_f32(src: &mut BufReader<File>, dst: &mut [f32]) -> std::io::Result<()> {
    src.read_f32_into::<LittleEndian>(dst)
}

/* Here are Python algorithms that shows how to do a low-memory multiply A (or A.T) x B (or B.T)
   They are used by file_ata_piece and file_aat_piece with some optimizations for A and B being the same.

output_list = [np.zeros((4,4)) for i in range(4)]

# a.T.dot(b)
for a_col2 in range(0,4,2): # 1 pass through A, returning output chunk about the same size writing in one pass
    buffer_a2 = a[:,a_col2:a_col2+2]
    for b_col in range(4): # A1/a1 passes through B
        buffer_b = b[:,b_col]
        for i in range(4):
            b_val = buffer_b[i]
            a_slice = buffer_a2[i,:]
            for k in range(2): # A1/a1 * A0 passes through the output
                output_list[0][a_col2+k,b_col] += a_slice[k]*b_val

# a.dot(b.T)
for out_col2 in range(0,4,2): # 1 pass through output, returning chunk on each pass
    for col in range(4): # O1/o1 passes through A and B
        buffer_a = a[:,col]
        buffer_b = b[:,col]
        for k in range(2):
            for i in range(4):
                output_list[1][i,out_col2+k] += buffer_a[i]*buffer_b[out_col2+k]

# a.T.dot(b.T)
for a_col2 in range(0,4,2): # 1 pass through A, returning an output chunk on each pass
    buffer_a2 = a[:,a_col2:a_col2+2]
    for b_col in range(4):
        buffer_b = b[:,b_col]
        for i in range(4):
            b_val = buffer_b[i]
            for k in range(2):
                output_list[2][a_col2+k,i] += buffer_a2[b_col,k]*b_val

# a.dot(b)  - but should instead do  (b.T.dot(a.T)).T
for b_col2 in range(0,4,2): #Transpose of preceding one
    buffer_b2 = b[:,b_col2:b_col2+2]
    for a_col in range(4):
        buffer_a = a[:,a_col]
        for i in range(4):
            a_val = buffer_a[i]
            for k in range(2):
                output_list[3][i,b_col2+k] += buffer_b2[a_col,k]*a_val


for output in output_list:
    print(output)
 */

// Given A, a matrix in Fortran order in a file
// with row_count rows and col_count columns,
// and given a starting column,
// returns part of A.T x A, the column vs column product.
// The piece piece returned has dimensions
// (col_count-col_start) x ncols
// where ncols <= (col_count-col_start)
// Makes only one pass through the file.
#[allow(clippy::too_many_arguments)]
fn file_ata_piece<T: Float + Send + Sync + AddAssign, P: AsRef<Path>>(
    path: P,
    offset: u64,
    row_count: usize,
    col_count: usize,
    col_start: usize,
    ata_piece: &mut nd::ArrayViewMut2<'_, T>,
    log_frequency: usize,
    read_into: fn(&mut BufReader<File>, &mut [T]) -> std::io::Result<()>,
) -> Result<(), BedErrorPlus> {
    let (nrows, ncols) = ata_piece.dim();
    if (col_start >= col_count)
        || (col_start + nrows != col_count)
        || (col_start + ncols > col_count)
    {
        return Err(BedErrorPlus::BedError(BedError::CannotConvertBetaToFromF64));
    }

    _file_ata_piece_internal(
        path,
        offset,
        row_count,
        col_start,
        ata_piece,
        log_frequency,
        read_into,
    )
}

fn _file_ata_piece_internal<T: Float + Send + Sync + AddAssign, P: AsRef<Path>>(
    path: P,
    offset: u64,
    row_count: usize,
    col_start: usize,
    ata_piece: &mut nd::ArrayViewMut2<'_, T>,
    log_frequency: usize,
    read_into: fn(&mut BufReader<File>, &mut [T]) -> std::io::Result<()>,
) -> Result<(), BedErrorPlus> {
    let (nrows, ncols) = ata_piece.dim();
    if log_frequency > 0 {
        println!(
            "file_ata_piece: col_start={}, {}x{} output",
            col_start, nrows, ncols
        );
    };

    // Open the file and move to the starting col
    let mut buf_reader = BufReader::new(File::open(path)?);
    buf_reader.seek(SeekFrom::Start(
        offset + col_start as u64 * row_count as u64 * std::mem::size_of::<T>() as u64,
    ))?;

    let mut col_save_list: Vec<Vec<T>> = vec![];
    let mut col_reuse = vec![T::nan(); row_count];

    for (col_rel_index, mut ata_row) in ata_piece.axis_iter_mut(nd::Axis(0)).enumerate() {
        if log_frequency > 0 && col_rel_index % log_frequency == 0 {
            println!("   working on {} of {}", col_rel_index, nrows);
        }

        // Read next col and save if in range
        let col = if col_save_list.len() < ncols {
            let mut col_save = vec![T::nan(); row_count];
            read_into(&mut buf_reader, &mut col_save)?;
            col_save_list.push(col_save);
            &col_save_list.last().unwrap() // unwrap is OK here
        } else {
            read_into(&mut buf_reader, &mut col_reuse)?;
            &col_reuse
        };

        // Multiple saved sids with new sid
        let mut ata_row_trimmed = ata_row.slice_mut(nd::s![..col_save_list.len()]);
        nd::par_azip!((
            col_in_range in &col_save_list,
            mut ata_val in ata_row_trimmed.axis_iter_mut(nd::Axis(0))
        )
        {
            ata_val[()] = col_product(col_in_range, col);
        });
    }

    // Reflect the new product values
    for row_index in 0usize..ncols - 1 {
        for col_index in row_index..ncols {
            ata_piece[(row_index, col_index)] = ata_piece[(col_index, row_index)];
        }
    }
    Ok(())
}

fn col_product<T: Float + AddAssign>(col_i: &[T], col_j: &[T]) -> T {
    assert!(col_i.len() == col_j.len()); // real assert
    let mut product = T::zero();
    for row_index in 0..col_i.len() {
        product += col_i[row_index] * col_j[row_index];
    }
    product
}

// Given A, a matrix in Fortran order in a file
// with row_count rows and col_count columns,
// and given a starting column,
// returns part of A x A.T, the row vs row product.
// The piece piece returned has dimensions
// (row_count-row_start) x ncols
// where ncols <= (row_count-row_start)
// Makes only one pass through the file.
#[allow(clippy::too_many_arguments)]
fn file_aat_piece<T: Float + Sync + Send + AddAssign, P: AsRef<Path>>(
    path: P,
    offset: u64,
    row_count: usize,
    col_count: usize,
    row_start: usize,
    aat_piece: &mut nd::ArrayViewMut2<'_, T>,
    log_frequency: usize,
    read_into: fn(&mut BufReader<File>, &mut [T]) -> std::io::Result<()>,
) -> Result<(), BedErrorPlus> {
    let (nrows, ncols) = aat_piece.dim();

    if log_frequency > 0 {
        println!(
            "file_aat_piece: row_start={}, {}x{} output",
            row_start, nrows, ncols
        );
    };

    if (row_start >= row_count)
        || (row_start + nrows != row_count)
        || (row_start + ncols > row_count)
    {
        return Err(BedErrorPlus::BedError(BedError::CannotConvertBetaToFromF64));
    }

    aat_piece.fill(T::zero());

    // Open the file and move to the starting col
    let mut buf_reader = BufReader::new(File::open(path)?);

    let mut col = vec![T::nan(); row_count - row_start];

    for col_index in 0..col_count {
        if log_frequency > 0 && col_index % log_frequency == 0 {
            println!("   working on {} of {}", col_index, col_count);
        }

        // Read next col
        buf_reader.seek(SeekFrom::Start(
            offset + (col_index * row_count + row_start) as u64 * std::mem::size_of::<T>() as u64,
        ))?;
        read_into(&mut buf_reader, &mut col)?;

        nd::par_azip!(
            (index row_index1,
            mut aat_col in aat_piece.axis_iter_mut(nd::Axis(1))
        )
        {
            let val1 = col[row_index1];
            for row_index0 in row_index1..nrows {
                aat_col[row_index0] += val1 * col[row_index0];
            }
        });
    }

    // Notice that ata reflects and aat doesn't. They don't need
    // to be the same, but they could be.
    Ok(())
}

// References: https://www.youtube.com/watch?v=0zOg8_B71gE&t=22s
// https://deterministic.space/elegant-apis-in-rust.html
// https://rust-lang.github.io/api-guidelines/
// https://ricardomartins.cc/2016/08/03/convenient_and_idiomatic_conversions_in_rust

// !!!cmk later write doc tests (see https://deterministic.space/elegant-apis-in-rust.html#what-makes-an-api-elegant)
// !!!cmk later To enforce that every public API item is documented, use #![deny(missing_docs)].
// !!!cmk later conventions for formatting Rust documentation https://deterministic.space/machine-readable-inline-markdown-code-cocumentation.html

// !!!cmk later document and add issue that File(s) are not held, incorrectly allowing for the file to be changed between reads.

#[derive(Debug, Clone)]
enum LazyOrSkip<T> {
    Lazy,
    Skip,
    Some(T),
}

impl<T> LazyOrSkip<T> {
    pub fn is_lazy(&self) -> bool {
        match self {
            LazyOrSkip::Lazy => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Skippable<T> {
    Some(T),
    Skip,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Metadata<'a> {
    pub fid: Skippable<&'a nd::Array1<String>>,
    pub iid: Skippable<&'a nd::Array1<String>>,
    pub father: Skippable<&'a nd::Array1<String>>,
    pub mother: Skippable<&'a nd::Array1<String>>,
    pub sex: Skippable<&'a nd::Array1<i32>>,
    pub pheno: Skippable<&'a nd::Array1<String>>,

    pub chromosome: Skippable<&'a nd::Array1<String>>,
    pub sid: Skippable<&'a nd::Array1<String>>,
    pub cm_position: Skippable<&'a nd::Array1<f32>>,
    pub bp_position: Skippable<&'a nd::Array1<i32>>,
    pub allele_1: Skippable<&'a nd::Array1<String>>,
    pub allele_2: Skippable<&'a nd::Array1<String>>,
}

fn option_count<T>(array: &LazyOrSkip<nd::Array1<T>>) -> Option<usize> {
    match array {
        LazyOrSkip::Some(array) => Some(array.len()),
        LazyOrSkip::Skip => None,
        LazyOrSkip::Lazy => None,
    }
}

// https://crates.io/crates/typed-builder
// (or https://docs.rs/derive_builder/latest/derive_builder/)
// Somehow ndarray can do this: 	Array::zeros((3, 4, 5).f())
//       see https://docs.rs/ndarray/latest/ndarray/doc/ndarray_for_numpy_users/index.html
#[derive(Clone, Debug, Builder)]
#[builder(build_fn(private, name = "build_no_file_check", error = "BedErrorPlus"))]
pub struct Bed {
    // https://stackoverflow.com/questions/32730714/what-is-the-right-way-to-store-an-immutable-path-in-a-struct
    // don't emit a setter, but keep the field declaration on the builder
    #[builder(setter(custom))]
    pub path: PathBuf, // !!!cmk later always clone?

    #[builder(setter(custom))]
    #[builder(default = "None")]
    fam_path: Option<PathBuf>,

    #[builder(setter(custom))]
    #[builder(default = "None")]
    bim_path: Option<PathBuf>,

    #[builder(setter(custom))]
    #[builder(default = "true")]
    is_checked_early: bool,

    #[builder(setter(custom))]
    #[builder(default = "None")]
    iid_count: Option<usize>,

    #[builder(setter(custom))]
    #[builder(default = "None")]
    sid_count: Option<usize>,

    #[builder(setter(custom))]
    #[builder(default = "LazyOrSkip::Lazy")]
    fid: LazyOrSkip<nd::Array1<String>>,

    #[builder(setter(custom))]
    #[builder(default = "LazyOrSkip::Lazy")]
    iid: LazyOrSkip<nd::Array1<String>>,

    #[builder(setter(custom))]
    #[builder(default = "LazyOrSkip::Lazy")]
    father: LazyOrSkip<nd::Array1<String>>,

    #[builder(setter(custom))]
    #[builder(default = "LazyOrSkip::Lazy")]
    mother: LazyOrSkip<nd::Array1<String>>,

    #[builder(setter(custom))]
    #[builder(default = "LazyOrSkip::Lazy")]
    sex: LazyOrSkip<nd::Array1<i32>>,

    #[builder(setter(custom))]
    #[builder(default = "LazyOrSkip::Lazy")]
    pheno: LazyOrSkip<nd::Array1<String>>,

    #[builder(setter(custom))]
    #[builder(default = "LazyOrSkip::Lazy")]
    chromosome: LazyOrSkip<nd::Array1<String>>,

    #[builder(setter(custom))]
    #[builder(default = "LazyOrSkip::Lazy")]
    sid: LazyOrSkip<nd::Array1<String>>,

    #[builder(setter(custom))]
    #[builder(default = "LazyOrSkip::Lazy")]
    cm_position: LazyOrSkip<nd::Array1<f32>>,

    // i32 based on https://www.cog-genomics.org/plink2/formats#bim
    #[builder(setter(custom))]
    #[builder(default = "LazyOrSkip::Lazy")]
    bp_position: LazyOrSkip<nd::Array1<i32>>,

    #[builder(setter(custom))]
    #[builder(default = "LazyOrSkip::Lazy")]
    allele_1: LazyOrSkip<nd::Array1<String>>,

    #[builder(setter(custom))]
    #[builder(default = "LazyOrSkip::Lazy")]
    allele_2: LazyOrSkip<nd::Array1<String>>,
}

impl BedBuilder {
    fn new<P: AsRef<Path>>(path: P) -> Self {
        let path: PathBuf = path.as_ref().into();

        Self {
            path: Some(path),
            fam_path: None,
            bim_path: None,

            is_checked_early: None,
            iid_count: None,
            sid_count: None,

            fid: None,
            iid: None,
            father: None,
            mother: None,
            sex: None,
            pheno: None,

            chromosome: None,
            sid: None,
            cm_position: None,
            bp_position: None,
            allele_1: None,
            allele_2: None,
        }
    }

    fn check_counts(
        count_vec: Vec<Option<usize>>,
        option_xid_count: &mut Option<usize>,
        prefix: &str,
    ) -> Result<(), BedErrorPlus> {
        for option_count in count_vec {
            if let Some(count) = option_count {
                match option_xid_count {
                    Some(xid_count) => {
                        if *xid_count != count {
                            return Err(BedError::InconsistentCount(
                                prefix.to_string(),
                                *xid_count,
                                count,
                            )
                            .into());
                        }
                    }
                    None => {
                        *option_xid_count = Some(count);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn build(&self) -> Result<Bed, BedErrorPlus> {
        let mut bed = self.build_no_file_check()?;

        if bed.is_checked_early {
            open_and_check(&bed.path)?;
        }

        BedBuilder::check_counts(
            vec![
                option_count(&bed.fid),
                option_count(&bed.iid),
                option_count(&bed.father),
                option_count(&bed.mother),
                option_count(&bed.sex),
                option_count(&bed.pheno),
            ],
            &mut bed.iid_count,
            "iid",
        )?;

        BedBuilder::check_counts(
            vec![
                option_count(&bed.chromosome),
                option_count(&bed.sid),
                option_count(&bed.cm_position),
                option_count(&bed.bp_position),
                option_count(&bed.allele_1),
                option_count(&bed.allele_2),
            ],
            &mut bed.sid_count,
            "sid",
        )?;

        Ok(bed)
    }

    pub fn fam_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.fam_path = Some(Some(path.as_ref().into()));
        self
    }

    pub fn bim_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.bim_path = Some(Some(path.as_ref().into()));
        self
    }

    pub fn skip_early_check(mut self) -> Self {
        self.is_checked_early = Some(false);
        self
    }

    pub fn skip_iid(mut self) -> Self {
        self.iid = Some(LazyOrSkip::Skip);
        self
    }

    pub fn skip_fid(mut self) -> Self {
        self.fid = Some(LazyOrSkip::Skip);
        self
    }
    pub fn skip_father(mut self) -> Self {
        self.father = Some(LazyOrSkip::Skip);
        self
    }

    pub fn skip_mother(mut self) -> Self {
        self.mother = Some(LazyOrSkip::Skip);
        self
    }

    pub fn skip_sex(mut self) -> Self {
        self.sex = Some(LazyOrSkip::Skip);
        self
    }

    pub fn skip_pheno(mut self) -> Self {
        self.pheno = Some(LazyOrSkip::Skip);
        self
    }

    pub fn skip_chromosome(mut self) -> Self {
        self.chromosome = Some(LazyOrSkip::Skip);
        self
    }

    pub fn skip_sid(mut self) -> Self {
        self.sid = Some(LazyOrSkip::Skip);
        self
    }

    pub fn skip_cm_position(mut self) -> Self {
        self.cm_position = Some(LazyOrSkip::Skip);
        self
    }

    pub fn skip_bp_position(mut self) -> Self {
        self.bp_position = Some(LazyOrSkip::Skip);
        self
    }
    pub fn skip_allele_1(mut self) -> Self {
        self.allele_1 = Some(LazyOrSkip::Skip);
        self
    }

    pub fn skip_allele_2(mut self) -> Self {
        self.allele_2 = Some(LazyOrSkip::Skip);
        self
    }

    pub fn iid_count(mut self, count: usize) -> Self {
        self.iid_count = Some(Some(count));
        self
    }

    pub fn sid_count(mut self, count: usize) -> Self {
        self.sid_count = Some(Some(count));
        self
    }

    // https://stackoverflow.com/questions/38183551/concisely-initializing-a-vector-of-strings
    // https://stackoverflow.com/questions/65250496/how-to-convert-intoiteratoritem-asrefstr-to-iteratoritem-str-in-rust
    pub fn fid<I: IntoIterator<Item = T>, T: AsRef<str>>(mut self, fid: I) -> Self {
        self.fid = Some(LazyOrSkip::Some(
            fid.into_iter().map(|s| s.as_ref().to_string()).collect(),
        ));
        self
    }

    pub fn iid<I: IntoIterator<Item = T>, T: AsRef<str>>(mut self, iid: I) -> Self {
        self.iid = Some(LazyOrSkip::Some(
            iid.into_iter().map(|s| s.as_ref().to_string()).collect(),
        ));

        self
    }
    pub fn father<I: IntoIterator<Item = T>, T: AsRef<str>>(mut self, father: I) -> Self {
        self.father = Some(LazyOrSkip::Some(
            father.into_iter().map(|s| s.as_ref().to_string()).collect(),
        ));
        self
    }
    pub fn mother<I: IntoIterator<Item = T>, T: AsRef<str>>(mut self, mother: I) -> Self {
        self.mother = Some(LazyOrSkip::Some(
            mother.into_iter().map(|s| s.as_ref().to_string()).collect(),
        ));
        self
    }

    pub fn sex<I: IntoIterator<Item = i32>>(mut self, sex: I) -> Self {
        self.sex = Some(LazyOrSkip::Some(sex.into_iter().map(|s| s).collect()));
        self
    }

    pub fn pheno<I: IntoIterator<Item = T>, T: AsRef<str>>(mut self, pheno: I) -> Self {
        self.pheno = Some(LazyOrSkip::Some(
            pheno.into_iter().map(|s| s.as_ref().to_string()).collect(),
        ));
        self
    }

    pub fn chromosome<I: IntoIterator<Item = T>, T: AsRef<str>>(mut self, chromosome: I) -> Self {
        self.chromosome = Some(LazyOrSkip::Some(
            chromosome
                .into_iter()
                .map(|s| s.as_ref().to_string())
                .collect(),
        ));
        self
    }

    pub fn sid<I: IntoIterator<Item = T>, T: AsRef<str>>(mut self, sid: I) -> Self {
        self.sid = Some(LazyOrSkip::Some(
            sid.into_iter().map(|s| s.as_ref().to_string()).collect(),
        ));
        self
    }

    pub fn cm_position<I: IntoIterator<Item = f32>>(mut self, cm_position: I) -> Self {
        self.cm_position = Some(LazyOrSkip::Some(
            cm_position.into_iter().map(|s| s).collect(),
        ));
        self
    }

    pub fn bp_position<I: IntoIterator<Item = i32>>(mut self, bp_position: I) -> Self {
        self.bp_position = Some(LazyOrSkip::Some(
            bp_position.into_iter().map(|s| s).collect(),
        ));
        self
    }

    pub fn allele_1<I: IntoIterator<Item = T>, T: AsRef<str>>(mut self, allele_1: I) -> Self {
        self.allele_1 = Some(LazyOrSkip::Some(
            allele_1
                .into_iter()
                .map(|s| s.as_ref().to_string())
                .collect(),
        ));
        self
    }

    pub fn allele_2<I: IntoIterator<Item = T>, T: AsRef<str>>(mut self, allele_2: I) -> Self {
        self.allele_2 = Some(LazyOrSkip::Some(
            allele_2
                .into_iter()
                .map(|s| s.as_ref().to_string())
                .collect(),
        ));
        self
    }
}

fn to_metadata_path(
    bed_path: &PathBuf,
    metadata_path: &Option<PathBuf>,
    extension: &str,
) -> PathBuf {
    if let Some(metadata_path) = metadata_path {
        metadata_path.clone()
    } else {
        bed_path.with_extension(extension)
    }
}

fn to_skippable<'a, T>(lazy_or_skip: &'a LazyOrSkip<T>) -> Skippable<&'a T> {
    match lazy_or_skip {
        LazyOrSkip::Lazy => panic!("assert: impossible"),
        LazyOrSkip::Skip => Skippable::Skip,
        LazyOrSkip::Some(some) => Skippable::Some(some),
    }
}

// !!!cmk later should bed builder be able to accept a metadata struct?

impl Bed {
    /// Attempts to open a PLINK .bed file for reading. Supports options.
    ///
    /// > Also see [`Bed::new`](struct.Bed.html#method.new), which does not support options.
    ///
    /// The options, [listed here](struct.BedBuilder.html#implementations), can:
    ///  * set the path of the .fam and/or .bim file
    ///  * override selected metadata, for example, replacing the individual ids.
    ///  * give the number of individuals (samples) or SNPs (variants)
    ///  * control checking the validity of the .bed file's header
    ///  * skip reading selected metadata
    ///
    /// # Errors
    /// By default, this method will return an error if the file is missing or its header
    /// is ill-formed. It will also return an error if the options contradict each other.
    /// See [`BedError`](enum.BedError.html) and [`BedErrorPlus`](enum.BedErrorPlus.html)
    /// for all possible errors.
    ///
    /// # Examples
    /// List individual (sample) [`iid`](struct.Bed.html#method.iid) and
    /// SNP (variant) [`sid`](struct.Bed.html#method.sid),
    /// then [`read`](struct.Bed.html#method.read) the whole file.
    ///
    /// ```
    /// use ndarray as nd;
    /// use bed_reader::Bed;
    /// use bed_reader::assert_eq_nan;
    ///
    /// let file_name = "bed_reader/tests/data/small.bed";
    /// let mut bed = Bed::builder(file_name).build()?;
    /// println!("{:?}", bed.iid()?);
    /// println!("{:?}", bed.sid()?);
    /// let val = bed.read::<f64>()?;
    ///
    /// assert_eq_nan(
    ///     &val,
    ///     &nd::array![
    ///         [1.0, 0.0, f64::NAN, 0.0],
    ///         [2.0, 0.0, f64::NAN, 2.0],
    ///         [0.0, 1.0, 2.0, 0.0]
    ///     ],
    /// );
    /// # use bed_reader::BedErrorPlus;
    /// # Ok::<(), BedErrorPlus>(())
    /// ```
    /// *This outputs:*
    /// ```text
    /// ["iid1", "iid2", "iid3"], shape=[3], strides=[1], layout=CFcf (0xf), const ndim=1
    /// ["sid1", "sid2", "sid3", "sid4"], shape=[4], strides=[1], layout=CFcf (0xf), const ndim=1  
    /// ```
    ///
    /// Open the file and read data for one SNP (variant)
    /// at index position 2.
    /// ```
    /// # use ndarray as nd;
    /// # use bed_reader::Bed;
    /// # use bed_reader::assert_eq_nan;
    /// # let file_name = "bed_reader/tests/data/small.bed";
    /// use bed_reader::ReadOptions;
    ///
    /// let mut bed = Bed::builder(file_name).build()?;
    /// let val = ReadOptions::builder().sid_index(2).f64().read(&mut bed)?;
    ///
    /// assert_eq_nan(&val, &nd::array![[f64::NAN], [f64::NAN], [2.0]]);
    /// # use bed_reader::BedErrorPlus;
    /// # Ok::<(), BedErrorPlus>(())
    /// ```
    ///
    /// Replace [`iid`](struct.Bed.html#method.iid).
    /// ```
    /// # use ndarray as nd;
    /// # use bed_reader::Bed;
    /// # use bed_reader::assert_eq_nan;
    /// # let file_name = "bed_reader/tests/data/small.bed";
    /// use bed_reader::ReadOptions;
    ///
    /// let mut bed = Bed::builder(file_name)
    ///    .iid(["sample1", "sample2", "sample3"])
    ///    .build()?;
    /// println!("{:?}", bed.iid()?); // replaced
    /// println!("{:?}", bed.sid()?); // same as before
    /// # use bed_reader::BedErrorPlus;
    /// # Ok::<(), BedErrorPlus>(())
    /// ```
    /// *This outputs:*
    /// ```text
    /// ["sample1", "sample2", "sample3"], shape=[3], strides=[1], layout=CFcf (0xf), const ndim=1    
    /// ["sid1", "sid2", "sid3", "sid4"], shape=[4], strides=[1], layout=CFcf (0xf), const ndim=1
    /// ```
    /// Give the number of individuals (samples) and SNPs (variants) so that the .fam and
    /// .bim files need never be opened.
    /// ```
    /// # use ndarray as nd;
    /// # use bed_reader::Bed;
    /// # use bed_reader::assert_eq_nan;
    /// # let file_name = "bed_reader/tests/data/small.bed";
    /// # use bed_reader::ReadOptions;
    /// let mut bed = Bed::builder(file_name).iid_count(3).sid_count(4).build()?;
    /// let val = bed.read::<f64>()?;
    ///
    /// assert_eq_nan(
    ///     &val,
    ///     &nd::array![
    ///         [1.0, 0.0, f64::NAN, 0.0],
    ///         [2.0, 0.0, f64::NAN, 2.0],
    ///         [0.0, 1.0, 2.0, 0.0]
    ///     ],
    /// );
    /// # use bed_reader::BedErrorPlus;
    /// # Ok::<(), BedErrorPlus>(())
    /// ```
    /// Mark some properties as "don’t read or offer".
    /// ```
    /// # use ndarray as nd;
    /// # use bed_reader::Bed;
    /// # use bed_reader::assert_eq_nan;
    /// # let file_name = "bed_reader/tests/data/small.bed";
    /// # use bed_reader::ReadOptions;
    /// let mut bed = Bed::builder(file_name)
    ///     .skip_father()
    ///     .skip_mother()
    ///     .skip_sex()
    ///     .skip_pheno()
    ///     .skip_allele_1()
    ///     .skip_allele_2()
    ///     .build()?;
    /// println!("{:?}", bed.iid()?);
    /// bed.allele_2().expect_err("Can't be read");
    /// # use bed_reader::BedErrorPlus;
    /// # Ok::<(), BedErrorPlus>(())
    /// ```
    /// *This outputs:*
    /// ```text
    /// ["iid1", "iid2", "iid3"], shape=[3], strides=[1], layout=CFcf (0xf), const ndim=1
    /// ```
    pub fn builder<P: AsRef<Path>>(path: P) -> BedBuilder {
        BedBuilder::new(path)
    }

    /// Attempts to open a PLINK .bed file for reading. Does not support options.
    ///
    /// > Also see [`Bed::builder`](struct.Bed.html#method.builder), which does support options.
    ///
    /// # Errors
    /// By default, this method will return an error if the file is missing or its header
    /// is ill-formed. See [`BedError`](enum.BedError.html) and [`BedErrorPlus`](enum.BedErrorPlus.html)
    /// for all possible errors.
    ///
    /// # Examples
    /// List individual (sample) [`iid`](struct.Bed.html#method.iid) and
    /// SNP (variant) [`sid`](struct.Bed.html#method.sid),
    /// then [`read`](struct.Bed.html#method.read) the whole file.
    ///
    /// ```
    /// use ndarray as nd;
    /// use bed_reader::Bed;
    /// use bed_reader::assert_eq_nan;
    ///
    /// let file_name = "bed_reader/tests/data/small.bed";
    /// let mut bed = Bed::new(file_name)?;
    /// println!("{:?}", bed.iid()?);
    /// println!("{:?}", bed.sid()?);
    /// let val = bed.read::<f64>()?;
    ///
    /// assert_eq_nan(
    ///     &val,
    ///     &nd::array![
    ///         [1.0, 0.0, f64::NAN, 0.0],
    ///         [2.0, 0.0, f64::NAN, 2.0],
    ///         [0.0, 1.0, 2.0, 0.0]
    ///     ],
    /// );
    /// # use bed_reader::BedErrorPlus;
    /// # Ok::<(), BedErrorPlus>(())
    /// ```
    /// *This outputs:*
    /// ```text
    /// ["iid1", "iid2", "iid3"], shape=[3], strides=[1], layout=CFcf (0xf), const ndim=1
    /// ["sid1", "sid2", "sid3", "sid4"], shape=[4], strides=[1], layout=CFcf (0xf), const ndim=1  
    /// ```
    ///
    /// Open the file and read data for one SNP (variant)
    /// at index position 2.
    /// ```
    /// # use ndarray as nd;
    /// # use bed_reader::Bed;
    /// # use bed_reader::assert_eq_nan;
    /// # let file_name = "bed_reader/tests/data/small.bed";
    /// use bed_reader::ReadOptions;
    ///
    /// let mut bed = Bed::new(file_name)?;
    /// let val = ReadOptions::builder().sid_index(2).f64().read(&mut bed)?;
    ///
    /// assert_eq_nan(&val, &nd::array![[f64::NAN], [f64::NAN], [2.0]]);
    /// # use bed_reader::BedErrorPlus;
    /// # Ok::<(), BedErrorPlus>(())
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, BedErrorPlus> {
        Bed::builder(path).build()
    }

    /// Write genotype data with default metadata.
    ///
    /// > Also see [`WriteOptions::builder`](struct.WriteOptions.html#method.builder), which supports metadata and options.
    ///
    /// # Errors
    /// See [`BedError`](enum.BedError.html) and [`BedErrorPlus`](enum.BedErrorPlus.html)
    /// for all possible errors.
    ///
    /// # Example
    /// In this example, write genotype data using default metadata.
    /// ```
    /// use ndarray as nd;
    /// use bed_reader::{Bed, WriteOptions, tmp_path};
    ///
    /// let output_folder = tmp_path()?;
    /// let output_file = output_folder.join("small.bed");
    ///
    /// let val = nd::array![[1, 0, -127, 0], [2, 0, -127, 2], [0, 1, 2, 0]];
    /// Bed::write(&val, &output_file)?;
    ///
    /// // If we then read the new file and list the chromosome property,
    /// // it is an array of '0's, the default chromosome value.
    /// let mut bed2 = Bed::new(&output_file)?;
    /// println!("{:?}", bed2.chromosome()?);
    /// // ["0", "0", "0", "0"], shape=[4], strides=[1], layout=CFcf (0xf), const ndim=1
    /// # use bed_reader::BedErrorPlus;
    /// # Ok::<(), BedErrorPlus>(())
    /// ```
    pub fn write<S: nd::Data<Elem = TVal>, TVal: BedVal>(
        val: &nd::ArrayBase<S, nd::Ix2>,
        path: &Path,
    ) -> Result<(), BedErrorPlus> {
        WriteOptions::builder(path).write(val)
    }
    pub fn write_with_options<S, TVal>(
        val: &nd::ArrayBase<S, nd::Ix2>,
        write_options: &WriteOptions<TVal>,
    ) -> Result<(), BedErrorPlus>
    where
        S: nd::Data<Elem = TVal>,
        TVal: BedVal,
    {
        // !!!cmk later can this be done in one step??
        let shape = val.shape();
        let iid_count = shape[0];
        let sid_count = shape[1];
        let path = &write_options.path;

        let num_threads = compute_num_threads(write_options.num_threads)?;
        write_val(
            path,
            val,
            write_options.is_a1_counted,
            write_options.missing_value,
            num_threads,
        )?;

        let fam_path = to_metadata_path(path, &write_options.fam_path, "fam");
        if let Err(e) = fam_write_internal(write_options, iid_count, &fam_path) {
            // Clean up the file
            let _ = fs::remove_file(fam_path);
            return Err(e);
        }

        let bim_path = to_metadata_path(path, &write_options.bim_path, "bim");
        if let Err(e) = bim_write_internal(write_options, sid_count, &bim_path) {
            // Clean up the file
            let _ = fs::remove_file(bim_path);
            return Err(e);
        }

        Ok(())
    }

    pub fn fam_path(&mut self) -> Result<PathBuf, BedErrorPlus> {
        if let Some(path) = &self.fam_path {
            Ok(path.clone())
        } else {
            let path = to_metadata_path(&self.path, &self.fam_path, "fam");
            self.fam_path = Some(path.clone());
            Ok(path)
        }
    }

    pub fn bim_path(&mut self) -> Result<PathBuf, BedErrorPlus> {
        if let Some(path) = &self.bim_path {
            Ok(path.clone())
        } else {
            let path = to_metadata_path(&self.path, &self.bim_path, "bim");
            self.bim_path = Some(path.clone());
            Ok(path)
        }
    }

    pub fn iid_count(&mut self) -> Result<usize, BedErrorPlus> {
        if let Some(iid_count) = self.iid_count {
            Ok(iid_count)
        } else {
            let fam_path = self.fam_path()?;
            let iid_count = count_lines(fam_path)?;
            self.iid_count = Some(iid_count);
            Ok(iid_count)
        }
    }
    pub fn sid_count(&mut self) -> Result<usize, BedErrorPlus> {
        if let Some(sid_count) = self.sid_count {
            Ok(sid_count)
        } else {
            let bim_path = self.bim_path()?;
            let sid_count = count_lines(bim_path)?;
            self.sid_count = Some(sid_count);
            Ok(sid_count)
        }
    }

    pub fn dim(&mut self) -> Result<(usize, usize), BedErrorPlus> {
        Ok((self.iid_count()?, self.sid_count()?))
    }

    pub fn metadata(&mut self) -> Result<Metadata, BedErrorPlus> {
        self.fam()?;
        self.bim()?;

        let metadata = Metadata {
            fid: to_skippable(&self.fid),
            iid: to_skippable(&self.iid),
            father: to_skippable(&self.father),
            mother: to_skippable(&self.mother),
            sex: to_skippable(&self.sex),
            pheno: to_skippable(&self.pheno),

            chromosome: to_skippable(&self.chromosome),
            sid: to_skippable(&self.sid),
            cm_position: to_skippable(&self.cm_position),
            bp_position: to_skippable(&self.bp_position),
            allele_1: to_skippable(&self.allele_1),
            allele_2: to_skippable(&self.allele_2),
        };
        Ok(metadata)
    }

    fn fam(&mut self) -> Result<(), BedErrorPlus> {
        let mut field_vec: Vec<usize> = Vec::new();
        if self.fid.is_lazy() {
            field_vec.push(0);
        }
        if self.iid.is_lazy() {
            field_vec.push(1);
        }
        if self.father.is_lazy() {
            field_vec.push(2);
        }
        if self.mother.is_lazy() {
            field_vec.push(3);
        }
        if self.sex.is_lazy() {
            field_vec.push(4);
        }
        if self.pheno.is_lazy() {
            field_vec.push(5);
        }

        let fam_path = self.fam_path()?;
        let (mut vec_of_vec, count) = self.read_fam_or_bim(&field_vec, &fam_path)?;
        match self.iid_count {
            Some(iid_count) => {
                if iid_count != count {
                    return Err(
                        BedError::InconsistentCount("iid".to_string(), iid_count, count).into(),
                    );
                }
            }
            None => {
                self.iid_count = Some(count);
            }
        }

        // unwraps are safe because we pop once for every push
        if self.pheno.is_lazy() {
            self.pheno = LazyOrSkip::Some(nd::Array::from_vec(vec_of_vec.pop().unwrap()));
        }
        if self.sex.is_lazy() {
            let vec = vec_of_vec.pop().unwrap();
            let array = vec
                .iter()
                .map(|s| s.parse::<i32>())
                .collect::<Result<nd::Array1<i32>, _>>()?; // !!!cmk later test this error
            self.sex = LazyOrSkip::Some(array);
        }
        if self.mother.is_lazy() {
            self.mother = LazyOrSkip::Some(nd::Array::from_vec(vec_of_vec.pop().unwrap()));
        }
        if self.father.is_lazy() {
            self.father = LazyOrSkip::Some(nd::Array::from_vec(vec_of_vec.pop().unwrap()));
        }
        if self.iid.is_lazy() {
            self.iid = LazyOrSkip::Some(nd::Array::from_vec(vec_of_vec.pop().unwrap()));
        }
        if self.fid.is_lazy() {
            self.fid = LazyOrSkip::Some(nd::Array::from_vec(vec_of_vec.pop().unwrap()));
        }

        Ok(())
    }
    fn bim(&mut self) -> Result<(), BedErrorPlus> {
        let mut field_vec: Vec<usize> = Vec::new();
        if self.chromosome.is_lazy() {
            field_vec.push(0);
        }
        if self.sid.is_lazy() {
            field_vec.push(1);
        }
        if self.cm_position.is_lazy() {
            field_vec.push(2);
        }
        if self.bp_position.is_lazy() {
            field_vec.push(3);
        }
        if self.allele_1.is_lazy() {
            field_vec.push(4);
        }
        if self.allele_2.is_lazy() {
            field_vec.push(5);
        }

        let bim_path = self.bim_path()?;
        let (mut vec_of_vec, count) = self.read_fam_or_bim(&field_vec, &bim_path)?;
        match self.sid_count {
            Some(sid_count) => {
                if sid_count != count {
                    return Err(
                        BedError::InconsistentCount("sid".to_string(), sid_count, count).into(),
                    );
                }
            }
            None => {
                self.sid_count = Some(count);
            }
        }

        // unwraps are safe because we pop once for every push
        if self.allele_2.is_lazy() {
            self.allele_2 = LazyOrSkip::Some(nd::Array::from_vec(vec_of_vec.pop().unwrap()));
        }
        if self.allele_1.is_lazy() {
            self.allele_1 = LazyOrSkip::Some(nd::Array::from_vec(vec_of_vec.pop().unwrap()));
        }
        if self.bp_position.is_lazy() {
            let vec = vec_of_vec.pop().unwrap();
            let array = vec
                .iter()
                .map(|s| s.parse::<i32>())
                .collect::<Result<nd::Array1<i32>, _>>()?; // !!!cmk later test this error
            self.bp_position = LazyOrSkip::Some(array);
        }
        if self.cm_position.is_lazy() {
            let vec = vec_of_vec.pop().unwrap();
            let array = vec
                .iter()
                .map(|s| s.parse::<f32>())
                .collect::<Result<nd::Array1<f32>, _>>()?; // !!!cmk later test this error
            self.cm_position = LazyOrSkip::Some(array);
        }

        if self.sid.is_lazy() {
            self.sid = LazyOrSkip::Some(nd::Array::from_vec(vec_of_vec.pop().unwrap()));
        }
        if self.chromosome.is_lazy() {
            self.chromosome = LazyOrSkip::Some(nd::Array::from_vec(vec_of_vec.pop().unwrap()));
        }

        Ok(())
    }

    fn read_fam_or_bim(
        &self,
        field_vec: &Vec<usize>,
        path_buf: &PathBuf,
    ) -> Result<(Vec<Vec<String>>, usize), BedErrorPlus> {
        let mut vec_of_vec = vec![vec![]; field_vec.len()];

        let file = File::open(&path_buf)?;

        let reader = BufReader::new(file);
        let mut count = 0;
        for line in reader.lines() {
            let line = line?;
            count += 1;
            let field = line.split_whitespace();

            let mut field_count = 0;
            let mut of_interest_count = 0;
            for field in field {
                if field_vec.contains(&field_count) {
                    vec_of_vec[of_interest_count].push(field.to_string());
                    of_interest_count += 1;
                }
                field_count += 1;
            }
            if field_count != 6 {
                return Err(BedError::MetadataFieldCount(
                    6,
                    field_count,
                    path_buf.to_str().unwrap().to_string(),
                )
                .into());
            }
        }

        Ok((vec_of_vec, count))
    }

    fn unlazy_fam<T: FromStringArray<T>>(&mut self, is_lazy: bool) -> Result<(), BedErrorPlus> {
        if is_lazy {
            self.fam()?
        }
        Ok(())
    }
    fn unlazy_bim<T: FromStringArray<T>>(&mut self, is_lazy: bool) -> Result<(), BedErrorPlus> {
        if is_lazy {
            self.bim()?
        }
        Ok(())
    }

    fn get_some_field<'a, T: FromStringArray<T>>(
        &'a self,
        field: &'a LazyOrSkip<nd::Array1<T>>,
        name: &str,
    ) -> Result<&'a nd::Array1<T>, BedErrorPlus> {
        match field {
            LazyOrSkip::Some(array) => Ok(array),
            LazyOrSkip::Skip => Err(BedError::CannotUseSkippedMetadata(name.to_string()).into()),
            LazyOrSkip::Lazy => panic!("impossible"),
        }
    }

    pub fn fid(&mut self) -> Result<&nd::Array1<String>, BedErrorPlus> {
        self.unlazy_fam::<String>(self.fid.is_lazy())?;
        self.get_some_field(&self.fid, "fid")
    }

    pub fn iid(&mut self) -> Result<&nd::Array1<String>, BedErrorPlus> {
        self.unlazy_fam::<String>(self.iid.is_lazy())?;
        self.get_some_field(&self.iid, "iid")
    }
    pub fn father(&mut self) -> Result<&nd::Array1<String>, BedErrorPlus> {
        self.unlazy_fam::<String>(self.father.is_lazy())?;
        self.get_some_field(&self.father, "father")
    }
    pub fn mother(&mut self) -> Result<&nd::Array1<String>, BedErrorPlus> {
        self.unlazy_fam::<String>(self.mother.is_lazy())?;
        self.get_some_field(&self.mother, "mother")
    }
    pub fn sex(&mut self) -> Result<&nd::Array1<i32>, BedErrorPlus> {
        self.unlazy_fam::<String>(self.sex.is_lazy())?;
        self.get_some_field(&self.sex, "sex")
    }
    pub fn pheno(&mut self) -> Result<&nd::Array1<String>, BedErrorPlus> {
        self.unlazy_fam::<String>(self.pheno.is_lazy())?;
        self.get_some_field(&self.pheno, "pheno")
    }

    pub fn chromosome(&mut self) -> Result<&nd::Array1<String>, BedErrorPlus> {
        self.unlazy_bim::<String>(self.chromosome.is_lazy())?;
        self.get_some_field(&self.chromosome, "chromosome")
    }
    pub fn sid(&mut self) -> Result<&nd::Array1<String>, BedErrorPlus> {
        self.unlazy_bim::<String>(self.sid.is_lazy())?;
        self.get_some_field(&self.sid, "sid")
    }
    pub fn cm_position(&mut self) -> Result<&nd::Array1<f32>, BedErrorPlus> {
        self.unlazy_bim::<String>(self.cm_position.is_lazy())?;
        self.get_some_field(&self.cm_position, "cm_position")
    }
    pub fn bp_position(&mut self) -> Result<&nd::Array1<i32>, BedErrorPlus> {
        self.unlazy_bim::<String>(self.bp_position.is_lazy())?;
        self.get_some_field(&self.bp_position, "bp_position")
    }
    pub fn allele_1(&mut self) -> Result<&nd::Array1<String>, BedErrorPlus> {
        self.unlazy_bim::<String>(self.allele_1.is_lazy())?;
        self.get_some_field(&self.allele_1, "allele_1")
    }
    pub fn allele_2(&mut self) -> Result<&nd::Array1<String>, BedErrorPlus> {
        self.unlazy_bim::<String>(self.allele_2.is_lazy())?;
        self.get_some_field(&self.allele_2, "allele_2")
    }

    /// Read genotype data.
    ///
    /// > Also see [`ReadOptions::builder`](struct.ReadOptions.html#method.builder) which supports selection and options.
    ///
    /// # Errors
    /// See [`BedError`](enum.BedError.html) and [`BedErrorPlus`](enum.BedErrorPlus.html)
    /// for all possible errors.
    ///
    /// # Examples
    /// Read all data in a .bed file.
    ///
    /// ```
    /// use ndarray as nd;
    /// use bed_reader::{Bed, ReadOptions};
    /// use bed_reader::assert_eq_nan;
    ///
    /// let file_name = "bed_reader/tests/data/small.bed";
    /// let mut bed = Bed::new(file_name)?;
    /// let val = bed.read::<f64>()?;
    ///
    /// assert_eq_nan(
    ///     &val,
    ///     &nd::array![
    ///         [1.0, 0.0, f64::NAN, 0.0],
    ///         [2.0, 0.0, f64::NAN, 2.0],
    ///         [0.0, 1.0, 2.0, 0.0]
    ///     ],
    /// );
    ///
    /// // Your output array can be f32, f64, or i8
    /// let val = bed.read::<i8>()?;
    /// assert_eq_nan(
    ///     &val,
    ///     &nd::array![
    ///         [1, 0, -127, 0],
    ///         [2, 0, -127, 2],
    ///         [0, 1, 2, 0]
    ///     ],
    /// );
    /// # use bed_reader::BedErrorPlus;
    /// # Ok::<(), BedErrorPlus>(())
    /// ```    
    pub fn read<TVal: BedVal>(&mut self) -> Result<nd::Array2<TVal>, BedErrorPlus> {
        let read_options = ReadOptions::<TVal>::builder().build()?;
        self.read_with_options(&read_options)
    }

    // !!!cmk later document that any .f() or .c() in read options is ignored
    pub fn read_and_fill_with_options<TVal: BedVal>(
        &mut self,
        val: &mut nd::ArrayViewMut2<'_, TVal>, //mutable slices additionally allow to modify elements. But slices cannot grow - they are just a view into some vector.,
        read_options: &ReadOptions<TVal>,
    ) -> Result<(), BedErrorPlus> {
        let iid_count = self.iid_count()?;
        let sid_count = self.sid_count()?;

        let num_threads = compute_num_threads(read_options.num_threads)?;

        let iid_index = read_options.iid_index.to_vec(iid_count)?;
        let sid_index = read_options.sid_index.to_vec(sid_count)?;

        let shape = val.shape();
        if shape.len() != 2 || (shape[0], shape[1]) != (iid_index.len(), sid_index.len()) {
            return Err(BedError::InvalidShape(
                iid_index.len(),
                sid_index.len(),
                shape[0],
                shape[1],
            )
            .into());
        }

        read_no_alloc(
            &self.path,
            iid_count,
            sid_count,
            read_options.is_a1_counted,
            &iid_index,
            &sid_index,
            read_options.missing_value,
            num_threads,
            &mut val.view_mut(),
        )?;

        Ok(())
    }

    pub fn read_and_fill<TVal: BedVal>(
        &mut self,
        val: &mut nd::ArrayViewMut2<'_, TVal>, //mutable slices additionally allow to modify elements. But slices cannot grow - they are just a view into some vector.,
    ) -> Result<(), BedErrorPlus> {
        let read_options = ReadOptions::<TVal>::builder().build()?;
        let num_threads = compute_num_threads(read_options.num_threads)?;

        let iid_count = self.iid_count()?;
        let sid_count = self.sid_count()?;
        let iid_index = read_options.iid_index.to_vec(iid_count)?;
        let sid_index = read_options.sid_index.to_vec(sid_count)?;

        read_no_alloc(
            &self.path,
            iid_count,
            sid_count,
            read_options.is_a1_counted,
            &iid_index,
            &sid_index,
            read_options.missing_value,
            num_threads,
            &mut val.view_mut(),
        )?;

        Ok(())
    }

    pub fn read_with_options<TVal: BedVal>(
        &mut self,
        read_options: &ReadOptions<TVal>,
    ) -> Result<nd::Array2<TVal>, BedErrorPlus> {
        let iid_count_in = self.iid_count()?;
        let sid_count_in = self.sid_count()?;
        let iid_count_out = read_options.iid_index.len(iid_count_in)?;
        let sid_count_out = read_options.sid_index.len(sid_count_in)?;
        let shape = ShapeBuilder::set_f((iid_count_out, sid_count_out), read_options.is_f);
        let mut val = nd::Array2::<TVal>::default(shape);

        self.read_and_fill_with_options(&mut val.view_mut(), read_options)?;

        Ok(val)
    }
}

fn compute_num_threads(option_num_threads: Option<usize>) -> Result<usize, BedErrorPlus> {
    let num_threads = if let Some(num_threads) = option_num_threads {
        num_threads
    } else {
        if let Ok(num_threads) = env::var("BED_READER_NUM_THREADS") {
            num_threads.parse::<usize>()?
        } else if let Ok(num_threads) = env::var("NUM_THREADS") {
            num_threads.parse::<usize>()?
        } else {
            0
        }
    };
    Ok(num_threads)
}

impl Index {
    // !!!cmk later test every case
    // We can't define a 'From' because we want to add count at the last moment.
    // Would be nice to not always allocate a new vec, maybe with Rc<[T]>?
    // Even better would be to support an iterator from Index (an enum with fields).
    pub fn to_vec(&self, count: usize) -> Result<Vec<usize>, BedErrorPlus> {
        match self {
            Index::All => Ok((0..count).collect()),
            Index::Vec(vec) => Ok(vec.to_vec()),
            Index::NDArrayBool(nd_array_bool) => {
                if nd_array_bool.len() != count {
                    return Err(
                        BedError::BoolArrayVectorWrongLength(count, nd_array_bool.len()).into(),
                    );
                }
                Ok(nd_array_bool
                    .iter()
                    .enumerate()
                    .filter(|(_, b)| **b)
                    .map(|(i, _)| i)
                    .collect())
            }
            // !!!cmk later can we implement this without two allocations?
            Index::NDSliceInfo(nd_slice_info) => {
                Ok(RangeNdSlice::new(nd_slice_info, count)?.to_vec())
            }
            Index::RangeAny(range_any) => {
                let range = range_any.to_range(count)?;
                Ok(range.collect::<Vec<usize>>())
            }
            Index::NDArray(nd_array) => Ok(nd_array.to_vec()),
            Index::One(one) => Ok(vec![*one]),
            Index::VecBool(vec_bool) => {
                if vec_bool.len() != count {
                    return Err(BedError::BoolArrayVectorWrongLength(count, vec_bool.len()).into());
                }
                Ok(vec_bool
                    .iter()
                    .enumerate()
                    .filter(|(_, b)| **b)
                    .map(|(i, _)| i)
                    .collect())
            }
        }
    }
}

pub(crate) type SliceInfo1 =
    nd::SliceInfo<[nd::SliceInfoElem; 1], nd::Dim<[usize; 1]>, nd::Dim<[usize; 1]>>;

/// A specification of which individuals (samples) or SNPs (variants) to read.
///
/// By default, all individuals or SNPs are read.
/// The indices can be specified as:
///   * an unsigned index
///   * a vector or ndarray of unsigned indices
///   * a range
///   * a vector or ndarray of booleans
///   * an ndarray slice (which supports negative indexing and steps)
///
/// # Examples
/// ```
/// use ndarray as nd;
/// use bed_reader::{Bed, ReadOptions};
/// use bed_reader::assert_eq_nan;
///
/// let file_name = "bed_reader/tests/data/some_missing.bed";
/// let mut bed = Bed::new(file_name)?;
///
/// // Read all individuals and all SNPs
/// let val = ReadOptions::builder().f64().read(&mut bed)?;
/// assert!(val.dim() == (50, 10));
///
/// // Read the individual at index position 10 and all SNPs
/// let val = ReadOptions::builder().iid_index(10).f64().read(&mut bed)?;
/// assert!(val.dim() == (1, 10));
///
/// // Read the individuals at index positions 0,5,10 and
/// // the SNP at index position 3
/// let val = ReadOptions::builder()
///     .iid_index([0,5,10])
///     .sid_index(5)
///     .f64()
///     .read(&mut bed)?;
/// assert!(val.dim() == (3, 1));
/// // Repeat, but with an ndarray
/// let val = ReadOptions::builder()
///     .iid_index(nd:array![0,5,10])
///     .sid_index(5)
///     .f64()
///     .read(&mut bed)?;
/// assert!(val.dim() == (3, 1));
///
/// // Create a boolean ndarray identifying SNPs in chromosome 5,
/// // then select those SNPs.
/// let snp_5 = bed3.chromosome()?.map(|elem| elem == "5"));
/// let val = ReadOptions::builder()
///     .sid_index(snp_5)
///     .f64()
///     .read(&mut bed)?;
/// assert!(val.dim() == (50, 3));
///
/// Use ndarray's slice macro, [`s!`](https://docs.rs/ndarray/latest/ndarray/macro.s.html),
/// to select every 2nd individual and every 3rd SNP.
/// let val = ReadOptions::builder()
///     .iid_index(s![..;2])
///     .sid_index(s![..;3])
///     .f64()
///     .read(&mut bed)?;
/// assert!(val.dim() == (3, 1));
/// Use ndarray's slice macro, [`s!`](https://docs.rs/ndarray/latest/ndarray/macro.s.html),
/// to select the 2nd to last individual (via negative indexing)
/// and every 3rd SNP in reverse order.)
/// let val = ReadOptions::builder()
///     .iid_index(s![-2])
///     .sid_index(s![..;-3])
///     .f64()
///     .read(&mut bed)?;
/// assert!(val.dim() == (3, 1));

#[derive(Debug, Clone)]
pub enum Index {
    // Could implement an enumerator, but it is complex and requires a 'match' on each next()
    //     https://stackoverflow.com/questions/65272613/how-to-implement-intoiterator-for-an-enum-of-iterable-variants
    // !!!cmk later add docs to type typedbuilder stuff: https://docs.rs/typed-builder/latest/typed_builder/derive.TypedBuilder.html#customisation-with-attributes
    All,
    One(usize),
    Vec(Vec<usize>),
    NDArray(nd::Array1<usize>),
    VecBool(Vec<bool>),
    NDArrayBool(nd::Array1<bool>),
    NDSliceInfo(SliceInfo1),
    RangeAny(RangeAny),
}

#[derive(Debug, Clone)]
pub struct RangeAny {
    start: Option<usize>,
    end: Option<usize>,
}

impl RangeAny {
    // https://stackoverflow.com/questions/55925523/array-cannot-be-indexed-by-rangefull
    fn to_range(&self, count: usize) -> Result<Range<usize>, BedErrorPlus> {
        let start = if let Some(start) = self.start {
            start
        } else {
            0
        };
        let end = if let Some(end) = self.end { end } else { count };
        if start > end {
            Err(BedError::StartGreaterThanEnd(start, end).into())
        } else {
            Ok(Range {
                start: start,
                end: end,
            })
        }
    }

    fn len(&self, count: usize) -> Result<usize, BedErrorPlus> {
        let range = self.to_range(count)?;
        Ok(range.end - range.start)
    }
}

#[derive(Debug, Clone)]
pub struct RangeNdSlice {
    start: usize,
    end: usize,
    step: usize,
    is_reversed: bool,
}

// https://www.geeksforgeeks.org/find-ceil-ab-without-using-ceil-function/
fn div_ceil(a: usize, b: usize) -> usize {
    (a + b - 1) / b
}

impl RangeNdSlice {
    fn len(&self) -> usize {
        if self.start > self.end {
            0
        } else {
            div_ceil(self.end - self.start, self.step)
        }
    }

    // https://docs.rs/ndarray/0.15.4/ndarray/struct.ArrayBase.html#slicing
    fn to_vec(&self) -> Vec<usize> {
        if self.start > self.end {
            Vec::new()
        } else {
            let mut vec: Vec<usize> = (self.start..self.end).step_by(self.step).collect();
            if self.is_reversed {
                vec.reverse();
            }
            vec
        }
    }

    fn new(nd_slice_info: &SliceInfo1, count: usize) -> Result<Self, BedErrorPlus> {
        //  self.to_vec(count).len(),
        // https://docs.rs/ndarray/0.15.4/ndarray/struct.ArrayBase.html#method.slice_collapse
        // Error in the following cases
        // * SliceInfo is not a 1-dimensional or is a NewAxis
        // * Step is 0
        // * Start is greater than count
        // * End is greater than count
        // As with ndarray, Start can be greater than End is allowed
        // and means the slice is empty.
        if nd_slice_info.in_ndim() != 1 || nd_slice_info.out_ndim() != 1 {
            return Err(BedError::NdSliceInfoNot1D.into());
        }

        let slice_info_elem = nd_slice_info[0];
        match slice_info_elem {
            nd::SliceInfoElem::Slice { start, end, step } => {
                // https://docs.rs/ndarray/0.15.4/ndarray/enum.SliceInfoElem.html
                // s![..], 0,None,1
                // s![a..b;2] a,b,2
                // s![a..;-1], from a to end in reverse order
                // start index; negative are counted from the back of the axis
                // end index; negative are counted from the back of the axis; when not present the default is the full length of the axis.
                // step size in elements; the default is 1, for every element.
                // A range with step size. end is an exclusive index. Negative start or end indexes are counted from the back of the axis. If end is None, the slice extends to the end of the axis.
                let step2: usize;
                let is_reverse2: bool;
                if step > 0 {
                    step2 = step as usize;
                    is_reverse2 = false;
                } else if step < 0 {
                    step2 = (-step) as usize;
                    is_reverse2 = true;
                } else {
                    return Err(BedError::StepZero.into());
                }

                let start2 = if start >= 0 {
                    let start3 = start as usize;
                    if start3 > count {
                        return Err(BedError::StartGreaterThanCount(start3, count).into());
                    } else {
                        start3
                    }
                } else {
                    let start3 = (-start) as usize;
                    if start3 > count {
                        return Err(BedError::StartGreaterThanCount(start3, count).into());
                    }
                    count - start3
                };

                let end2 = if let Some(end) = end {
                    if end >= 0 {
                        let end3 = end as usize;
                        if end3 > count {
                            return Err(BedError::EndGreaterThanCount(end3, count).into());
                        } else {
                            end3
                        }
                    } else {
                        let end3 = (-end) as usize;
                        if end3 > count {
                            return Err(BedError::EndGreaterThanCount(end3, count).into());
                        }
                        count - end3
                    }
                } else {
                    count
                };

                Ok(RangeNdSlice {
                    start: start2,
                    end: end2,
                    step: step2,
                    is_reversed: is_reverse2,
                })
            }
            nd::SliceInfoElem::Index(index) => Ok(RangeNdSlice {
                start: index as usize,
                end: index as usize + 1,
                step: 1,
                is_reversed: false,
            }),
            nd::SliceInfoElem::NewAxis => {
                return Err(BedError::NewAxis.into());
            }
        }
    }
}

impl Index {
    pub fn len(&self, count: usize) -> Result<usize, BedErrorPlus> {
        match self {
            Index::All => Ok(count),
            Index::One(_) => Ok(1),
            Index::Vec(vec) => Ok(vec.len()),
            Index::NDArray(nd_array) => Ok(nd_array.len()),
            Index::VecBool(vec_bool) => Ok(vec_bool.iter().filter(|&b| *b).count()),
            Index::NDArrayBool(nd_array_bool) => Ok(nd_array_bool.iter().filter(|&b| *b).count()),
            Index::NDSliceInfo(nd_slice_info) => Ok(RangeNdSlice::new(nd_slice_info, count)?.len()),
            Index::RangeAny(range_any) => range_any.len(count),
        }
    }
}

// !!!cmk later see if what ref conversions. See https://ricardomartins.cc/2016/08/03/convenient_and_idiomatic_conversions_in_rust
impl From<SliceInfo1> for Index {
    fn from(slice_info: SliceInfo1) -> Index {
        Index::NDSliceInfo(slice_info)
    }
}

fn to_range_any<T: RangeBounds<usize>>(range_thing: T) -> RangeAny {
    let start_bound = range_thing.start_bound();
    let start = match start_bound {
        Bound::Included(&start) => Some(start),
        Bound::Excluded(&start) => Some(start + 1),
        Bound::Unbounded => None,
    };

    let end_bound = range_thing.end_bound();
    let end = match end_bound {
        Bound::Included(&end) => Some(end + 1),
        Bound::Excluded(&end) => Some(end),
        Bound::Unbounded => None,
    };
    RangeAny { start, end }
}

impl From<RangeFull> for RangeAny {
    fn from(range_thing: RangeFull) -> RangeAny {
        to_range_any(range_thing)
    }
}

impl From<RangeFull> for Index {
    fn from(range_thing: RangeFull) -> Index {
        Index::RangeAny(range_thing.into())
    }
}

impl From<Range<usize>> for RangeAny {
    fn from(range_thing: Range<usize>) -> RangeAny {
        to_range_any(range_thing)
    }
}

impl From<Range<usize>> for Index {
    fn from(range_thing: Range<usize>) -> Index {
        Index::RangeAny(range_thing.into())
    }
}

impl From<RangeFrom<usize>> for RangeAny {
    fn from(range_thing: RangeFrom<usize>) -> RangeAny {
        to_range_any(range_thing)
    }
}

impl From<RangeFrom<usize>> for Index {
    fn from(range_thing: RangeFrom<usize>) -> Index {
        Index::RangeAny(range_thing.into())
    }
}

impl From<RangeInclusive<usize>> for RangeAny {
    fn from(range_thing: RangeInclusive<usize>) -> RangeAny {
        to_range_any(range_thing)
    }
}

impl From<RangeInclusive<usize>> for Index {
    fn from(range_thing: RangeInclusive<usize>) -> Index {
        Index::RangeAny(range_thing.into())
    }
}

impl From<RangeTo<usize>> for RangeAny {
    fn from(range_thing: RangeTo<usize>) -> RangeAny {
        to_range_any(range_thing)
    }
}

impl From<RangeTo<usize>> for Index {
    fn from(range_thing: RangeTo<usize>) -> Index {
        Index::RangeAny(range_thing.into())
    }
}

impl From<RangeToInclusive<usize>> for RangeAny {
    fn from(range_thing: RangeToInclusive<usize>) -> RangeAny {
        to_range_any(range_thing)
    }
}

impl From<RangeToInclusive<usize>> for Index {
    fn from(range_thing: RangeToInclusive<usize>) -> Index {
        Index::RangeAny(range_thing.into())
    }
}

impl From<&[usize]> for Index {
    fn from(array: &[usize]) -> Index {
        Index::Vec(array.to_vec())
    }
}

impl<const N: usize> From<[usize; N]> for Index {
    fn from(array: [usize; N]) -> Index {
        Index::Vec(array.to_vec())
    }
}

impl<const N: usize> From<&[usize; N]> for Index {
    fn from(array: &[usize; N]) -> Index {
        Index::Vec(array.to_vec())
    }
}

impl From<&nd::ArrayView1<'_, usize>> for Index {
    fn from(view: &nd::ArrayView1<usize>) -> Index {
        Index::NDArray(view.to_owned())
    }
}

impl From<&Vec<usize>> for Index {
    fn from(vec_ref: &Vec<usize>) -> Index {
        Index::Vec(vec_ref.clone())
    }
}

// !!!cmk later is ref &ndarray const array and bool OK
impl From<&nd::ArrayView1<'_, bool>> for Index {
    fn from(view: &nd::ArrayView1<bool>) -> Index {
        Index::NDArrayBool(view.to_owned())
    }
}

impl From<&Vec<bool>> for Index {
    fn from(vec_ref: &Vec<bool>) -> Index {
        Index::VecBool(vec_ref.clone())
    }
}

impl From<&[bool]> for Index {
    fn from(array: &[bool]) -> Index {
        Index::VecBool(array.to_vec())
    }
}

impl<const N: usize> From<[bool; N]> for Index {
    fn from(array: [bool; N]) -> Index {
        Index::VecBool(array.to_vec())
    }
}

impl<const N: usize> From<&[bool; N]> for Index {
    fn from(array: &[bool; N]) -> Index {
        Index::VecBool(array.to_vec())
    }
}

impl From<usize> for Index {
    fn from(one: usize) -> Index {
        Index::One(one)
    }
}

impl From<Vec<usize>> for Index {
    fn from(vec: Vec<usize>) -> Index {
        Index::Vec(vec)
    }
}
impl From<nd::Array1<usize>> for Index {
    fn from(nd_array: nd::Array1<usize>) -> Index {
        Index::NDArray(nd_array)
    }
}

impl From<nd::Array1<bool>> for Index {
    fn from(nd_array_bool: nd::Array1<bool>) -> Index {
        Index::NDArrayBool(nd_array_bool)
    }
}

impl From<Vec<bool>> for Index {
    fn from(vec_bool: Vec<bool>) -> Index {
        Index::VecBool(vec_bool)
    }
}

impl From<()> for Index {
    fn from(_: ()) -> Index {
        Index::All
    }
}

// See https://nullderef.com/blog/rust-parameters/
#[derive(Debug, Clone, Builder)]
pub struct ReadOptions<TVal: BedVal> {
    #[builder(default = "TVal::missing()")]
    missing_value: TVal,

    /// cmk 0 testing iid_index
    #[builder(default = "Index::All")]
    #[builder(setter(into))]
    iid_index: Index,

    #[builder(default = "Index::All")]
    #[builder(setter(into))]
    sid_index: Index,

    #[builder(default = "true")]
    is_f: bool,

    #[builder(default = "true")]
    is_a1_counted: bool,

    #[builder(default, setter(strip_option))]
    pub num_threads: Option<usize>,
}

impl<TVal: BedVal> ReadOptions<TVal> {
    /// Read genotype data. Supports selection and options.
    ///
    /// > Also see [`Bed::read`](struct.Bed.html#method.read), which does not support options.
    ///
    /// The options, [listed here](struct.ReadOptionsBuilder.html#implementations), can specify the:
    ///  * type of the array values ([`i8`](struct.ReadOptionsBuilder.html#method.i8), [`f32`](struct.ReadOptionsBuilder.html#method.f32), [`f64`](struct.ReadOptionsBuilder.html#method.f64))
    ///  * individuals (samples) to read with [`iid_index`](struct.ReadOptionsBuilder.html#method.iid_index)
    ///  * SNPs (variants) to read with [`sid_index`](struct.ReadOptionsBuilder.html#method.sid_index)
    ///  * order of the output array, either Fortran (default) or C
    ///  * value for missing data
    ///  * number of threads used for reading
    ///  * whether to count the first allele (default) or the second
    ///
    /// # Indexing
    /// To read selected individuals (samples)
    /// use [`iid_index`](struct.ReadOptionsBuilder.html#method.iid_index).
    /// To read selected SNPs (variants)
    /// use [`sid_index`](struct.ReadOptionsBuilder.html#method.sid_index).
    /// The indices can be specified as:
    ///   * an unsigned index
    ///   * a vector or ndarray of unsigned indices
    ///   * a range
    ///   * a vector or ndarray of booleans
    ///   * an ndarray slice (which supports negative indexing and steps)
    /// cmk 0 somewhere describe all the indexing possibilities
    ///
    /// # Errors
    /// See [`BedError`](enum.BedError.html) and [`BedErrorPlus`](enum.BedErrorPlus.html)
    /// for all possible errors.
    ///
    /// # Examples
    /// Read all data in a .bed file.
    ///
    /// ```
    /// use ndarray as nd;
    /// use bed_reader::{Bed, ReadOptions};
    /// use bed_reader::assert_eq_nan;
    ///
    /// let file_name = "bed_reader/tests/data/small.bed";
    /// let mut bed = Bed::new(file_name)?;
    /// let val = ReadOptions::builder().f64().read(&mut bed)?;
    ///
    /// assert_eq_nan(
    ///     &val,
    ///     &nd::array![
    ///         [1.0, 0.0, f64::NAN, 0.0],
    ///         [2.0, 0.0, f64::NAN, 2.0],
    ///         [0.0, 1.0, 2.0, 0.0]
    ///     ],
    /// );
    ///
    /// // Read the SNPs indexed by 2.
    /// let val = ReadOptions::builder().sid_index(2).f64().read(&mut bed)?;
    /// assert_eq_nan(&val, &nd::array![[f64::NAN], [f64::NAN], [2.0]]);
    ///
    /// // Read the SNPs indexed by 2, 3, and 0.
    /// let val = ReadOptions::builder()
    ///     .sid_index([2, 3, 0])
    ///     .f64()
    ///     .read(&mut bed)?;
    /// assert_eq_nan(
    ///     &val,
    ///     &nd::array![[f64::NAN, 0.0, 1.0], [f64::NAN, 2.0, 2.0], [2.0, 0.0, 0.0]],
    /// );
    ///
    /// //  Read SNPs from 1 (inclusive) to 4 (exclusive).
    /// let val = ReadOptions::builder()
    ///     .sid_index(1..4)
    ///     .f64()
    ///     .read(&mut bed)?;
    /// assert_eq_nan(
    ///     &val,
    ///     &nd::array![[0.0, f64::NAN, 0.0], [0.0, f64::NAN, 2.0], [1.0, 2.0, 0.0]],
    /// );
    ///
    /// // Print unique chrom values. Then, read all SNPs in chrom 5.
    /// use std::collections::HashSet;
    ///
    /// println!("{:?}", bed.chromosome()?.iter().collect::<HashSet<_>>());
    /// // This outputs: {"1", "5", "Y"}.
    /// let val = ReadOptions::builder()
    ///     .sid_index(bed.chromosome()?.map(|elem| elem == "5"))
    ///     .f64()
    ///     .read(&mut bed)?;
    /// assert_eq_nan(&val, &nd::array![[f64::NAN], [f64::NAN], [2.0]]);
    ///
    /// // Read 1st individual (across all SNPs).
    /// let val = ReadOptions::builder().iid_index(0).f64().read(&mut bed)?;
    /// assert_eq_nan(&val, &nd::array![[1.0, 0.0, f64::NAN, 0.0]]);
    ///
    /// // Read every 2nd individual.
    /// use ndarray::s;
    ///
    /// let val = ReadOptions::builder()
    ///     .iid_index(s![..;2])
    ///     .f64()
    ///     .read(&mut bed)?;
    /// assert_eq_nan(
    ///     &val,
    ///     &nd::array![[1.0, 0.0, f64::NAN, 0.0], [0.0, 1.0, 2.0, 0.0]],
    /// );
    ///
    /// // Read last and 2nd-to-last individuals and the last SNP
    /// let val = ReadOptions::builder()
    ///     .iid_index(s![-2..=-1;-1])
    ///     .sid_index(s![-1..=-1])
    ///     .f64()
    ///     .read(&mut bed)?;
    /// assert_eq_nan(&val, &nd::array![[0.0],[2.0]]);
    ///
    /// // Your output array can be f32, f64, or i8
    /// let val = ReadOptions::builder().i8().read(&mut bed)?;
    /// assert_eq_nan(
    ///     &val,
    ///     &nd::array![
    ///         [1, 0, -127, 0],
    ///         [2, 0, -127, 2],
    ///         [0, 1, 2, 0]
    ///     ],
    /// );
    /// # use bed_reader::BedErrorPlus;
    /// # Ok::<(), BedErrorPlus>(())
    /// ```
    pub fn builder() -> ReadOptionsBuilder<TVal> {
        ReadOptionsBuilder::default()
    }
}

impl<TVal: BedVal> ReadOptionsBuilder<TVal> {
    /// > See [`ReadOptions::builder`](struct.ReadOptions.html#method.builder)
    pub fn read(&self, bed: &mut Bed) -> Result<nd::Array2<TVal>, BedErrorPlus> {
        let read_options = self.build()?;
        bed.read_with_options(&read_options)
    }
    pub fn read_and_fill(
        &self,
        bed: &mut Bed,
        val: &mut nd::ArrayViewMut2<'_, TVal>, //mutable slices additionally allow to modify elements. But slices cannot grow - they are just a view into some vector.
    ) -> Result<(), BedErrorPlus> {
        let read_options = self.build()?;
        bed.read_and_fill_with_options(val, &read_options)
    }

    pub fn f(&mut self) -> &mut Self {
        self.is_f(true);
        self
    }

    pub fn c(&mut self) -> &mut Self {
        self.is_f(false);
        self
    }

    pub fn count_a1(&mut self) -> &mut Self {
        self.is_a1_counted = Some(true);
        self
    }

    pub fn count_a2(&mut self) -> &mut Self {
        self.is_a1_counted = Some(false);
        self
    }
}

impl ReadOptionsBuilder<i8> {
    pub fn i8(&mut self) -> &mut Self {
        self
    }
}

impl ReadOptionsBuilder<f32> {
    pub fn f32(&mut self) -> &mut Self {
        self
    }
}

impl ReadOptionsBuilder<f64> {
    pub fn f64(&mut self) -> &mut Self {
        self
    }
}

#[derive(Clone, Debug, Builder)]
#[builder(build_fn(private, name = "write_no_file_check", error = "BedErrorPlus"))]
pub struct WriteOptions<TVal>
where
    TVal: BedVal,
{
    #[builder(setter(custom))]
    pub path: PathBuf, // !!!cmk later always clone?

    #[builder(setter(custom))]
    #[builder(default = "None")]
    fam_path: Option<PathBuf>,

    #[builder(setter(custom))]
    #[builder(default = "None")]
    bim_path: Option<PathBuf>,

    #[builder(setter(custom))]
    #[builder(default = "Skippable::Skip")]
    fid: Skippable<nd::Array1<String>>,

    #[builder(setter(custom))]
    #[builder(default = "Skippable::Skip")]
    iid: Skippable<nd::Array1<String>>,

    #[builder(setter(custom))]
    #[builder(default = "Skippable::Skip")]
    father: Skippable<nd::Array1<String>>,

    #[builder(setter(custom))]
    #[builder(default = "Skippable::Skip")]
    mother: Skippable<nd::Array1<String>>,

    #[builder(setter(custom))]
    #[builder(default = "Skippable::Skip")]
    sex: Skippable<nd::Array1<i32>>,

    #[builder(setter(custom))]
    #[builder(default = "Skippable::Skip")]
    pheno: Skippable<nd::Array1<String>>,

    #[builder(setter(custom))]
    #[builder(default = "Skippable::Skip")]
    chromosome: Skippable<nd::Array1<String>>,

    #[builder(setter(custom))]
    #[builder(default = "Skippable::Skip")]
    sid: Skippable<nd::Array1<String>>,

    #[builder(setter(custom))]
    #[builder(default = "Skippable::Skip")]
    cm_position: Skippable<nd::Array1<f32>>,

    #[builder(setter(custom))]
    #[builder(default = "Skippable::Skip")]
    bp_position: Skippable<nd::Array1<i32>>,

    #[builder(setter(custom))]
    #[builder(default = "Skippable::Skip")]
    allele_1: Skippable<nd::Array1<String>>,

    #[builder(setter(custom))]
    #[builder(default = "Skippable::Skip")]
    allele_2: Skippable<nd::Array1<String>>,

    #[builder(default = "true")]
    is_a1_counted: bool,

    #[builder(default, setter(strip_option))]
    pub num_threads: Option<usize>,

    #[builder(default = "TVal::missing()")]
    missing_value: TVal,
}

impl<TVal> WriteOptions<TVal>
where
    TVal: BedVal,
{
    /// Write values to a file in PLINK .bed format. Supports metadata and options.
    ///
    /// > Also see [`Bed::write`](struct.Bed.html#method.write), which does not support metadata or options.
    ///
    /// The options, [listed here](struct.WriteOptionsBuilder.html#implementations), can specify the:
    ///  * metadata, including the individual ids and the SNP ids
    ///  * a non-default path for the .fam and/or .bim files
    ///  * a non-default value that represents missing data
    ///  * whether the first allele was counted (default) or the second
    ///  * number of threads to use for writing
    ///
    /// # Errors
    /// See [`BedError`](enum.BedError.html) and [`BedErrorPlus`](enum.BedErrorPlus.html)
    /// for all possible errors.
    ///
    /// # Examples
    /// In this example, all metadata is given.
    /// ```
    /// use ndarray as nd;
    /// use bed_reader::{Bed, WriteOptions, tmp_path};
    ///
    /// let output_folder = tmp_path()?;
    /// let output_file = output_folder.join("small.bed");
    /// let val = nd::array![
    ///     [1.0, 0.0, f64::NAN, 0.0],
    ///     [2.0, 0.0, f64::NAN, 2.0],
    ///     [0.0, 1.0, 2.0, 0.0]
    /// ];
    /// WriteOptions::builder(output_file)
    ///     .fid(["fid1", "fid1", "fid2"])
    ///     .iid(["iid1", "iid2", "iid3"])
    ///     .father(["iid23", "iid23", "iid22"])
    ///     .mother(["iid34", "iid34", "iid33"])
    ///     .sex([1, 2, 0])
    ///     .pheno(["red", "red", "blue"])
    ///     .chromosome(["1", "1", "5", "Y"])
    ///     .sid(["sid1", "sid2", "sid3", "sid4"])
    ///     .cm_position([100.4, 2000.5, 4000.7, 7000.9])
    ///     .bp_position([1, 100, 1000, 1004])
    ///     .allele_1(["A", "T", "A", "T"])
    ///     .allele_2(["A", "C", "C", "G"])
    ///     .write(&val)?;
    /// # use bed_reader::BedErrorPlus;
    /// # Ok::<(), BedErrorPlus>(())
    /// ```
    /// Here, no metadata is given, so default values are assigned.
    /// If we then read the new file and list the chromosome property,
    /// it is an array of '0's, the default chromosome value.
    /// ```
    /// # use ndarray as nd;
    /// # use bed_reader::{Bed, WriteOptions, tmp_path};
    /// # let output_folder = tmp_path()?;
    /// let output_file2 = output_folder.join("small2.bed");
    /// let val = nd::array![[1, 0, -127, 0], [2, 0, -127, 2], [0, 1, 2, 0]];
    ///
    /// WriteOptions::builder(&output_file2).write(&val)?;
    ///
    /// let mut bed2 = Bed::new(&output_file2)?;
    /// println!("{:?}", bed2.chromosome()?);
    /// // ["0", "0", "0", "0"], shape=[4], strides=[1], layout=CFcf (0xf), const ndim=1
    /// # use bed_reader::BedErrorPlus;
    /// # Ok::<(), BedErrorPlus>(())
    /// ```
    pub fn builder<P: AsRef<Path>>(path: P) -> WriteOptionsBuilder<TVal> {
        WriteOptionsBuilder::new(path)
    }

    pub fn fid(&self) -> Result<&nd::Array1<String>, BedErrorPlus> {
        match self.fid {
            Skippable::Some(ref fid) => Ok(fid),
            Skippable::Skip => Err(BedError::CannotUseSkippedMetadata("fid".to_string()).into()),
        }
    }
    pub fn iid(&self) -> Result<&nd::Array1<String>, BedErrorPlus> {
        match self.iid {
            Skippable::Some(ref iid) => Ok(iid),
            Skippable::Skip => Err(BedError::CannotUseSkippedMetadata("iid".to_string()).into()),
        }
    }
    pub fn father(&self) -> Result<&nd::Array1<String>, BedErrorPlus> {
        match self.father {
            Skippable::Some(ref father) => Ok(father),
            Skippable::Skip => Err(BedError::CannotUseSkippedMetadata("father".to_string()).into()),
        }
    }
    pub fn mother(&self) -> Result<&nd::Array1<String>, BedErrorPlus> {
        match self.mother {
            Skippable::Some(ref mother) => Ok(mother),
            Skippable::Skip => Err(BedError::CannotUseSkippedMetadata("mother".to_string()).into()),
        }
    }
    pub fn sex(&self) -> Result<&nd::Array1<i32>, BedErrorPlus> {
        match self.sex {
            Skippable::Some(ref sex) => Ok(sex),
            Skippable::Skip => Err(BedError::CannotUseSkippedMetadata("sex".to_string()).into()),
        }
    }
    pub fn pheno(&self) -> Result<&nd::Array1<String>, BedErrorPlus> {
        match self.pheno {
            Skippable::Some(ref pheno) => Ok(pheno),
            Skippable::Skip => Err(BedError::CannotUseSkippedMetadata("pheno".to_string()).into()),
        }
    }

    pub fn chromosome(&self) -> Result<&nd::Array1<String>, BedErrorPlus> {
        match self.chromosome {
            Skippable::Some(ref chromosome) => Ok(chromosome),
            Skippable::Skip => {
                Err(BedError::CannotUseSkippedMetadata("chromosome".to_string()).into())
            }
        }
    }
    pub fn sid(&self) -> Result<&nd::Array1<String>, BedErrorPlus> {
        match self.sid {
            Skippable::Some(ref sid) => Ok(sid),
            Skippable::Skip => Err(BedError::CannotUseSkippedMetadata("sid".to_string()).into()),
        }
    }
    pub fn cm_position(&self) -> Result<&nd::Array1<f32>, BedErrorPlus> {
        match self.cm_position {
            Skippable::Some(ref cm_position) => Ok(cm_position),
            Skippable::Skip => {
                Err(BedError::CannotUseSkippedMetadata("cm_position".to_string()).into())
            }
        }
    }

    pub fn bp_position(&self) -> Result<&nd::Array1<i32>, BedErrorPlus> {
        match self.bp_position {
            Skippable::Some(ref bp_position) => Ok(bp_position),
            Skippable::Skip => {
                Err(BedError::CannotUseSkippedMetadata("bp_position".to_string()).into())
            }
        }
    }
    pub fn allele_1(&self) -> Result<&nd::Array1<String>, BedErrorPlus> {
        match self.allele_1 {
            Skippable::Some(ref allele_1) => Ok(allele_1),
            Skippable::Skip => {
                Err(BedError::CannotUseSkippedMetadata("allele_1".to_string()).into())
            }
        }
    }
    pub fn allele_2(&self) -> Result<&nd::Array1<String>, BedErrorPlus> {
        match self.allele_2 {
            Skippable::Some(ref allele_2) => Ok(allele_2),
            Skippable::Skip => {
                Err(BedError::CannotUseSkippedMetadata("allele_2".to_string()).into())
            }
        }
    }

    pub fn is_a1_counted(&self) -> bool {
        self.is_a1_counted
    }
}

impl<TVal> WriteOptionsBuilder<TVal>
where
    TVal: BedVal,
{
    // !!! cmk later just use the default builder?
    pub fn build(&self) -> Result<WriteOptions<TVal>, BedErrorPlus> {
        let write_options = self.write_no_file_check()?;

        Ok(write_options)
    }

    // !!!cmk later should check that metadata agrees with val size
    // !!!cmk later maybe use the default builder?
    pub fn write<S: nd::Data<Elem = TVal>>(
        &self,
        val: &nd::ArrayBase<S, nd::Ix2>,
    ) -> Result<(), BedErrorPlus> {
        let write_options = self.build()?;
        Bed::write_with_options(val, &write_options)?;

        Ok(())
    }

    fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: Some(path.as_ref().into()),
            fam_path: None,
            bim_path: None,

            fid: None,
            iid: None,
            father: None,
            mother: None,
            sex: None,
            pheno: None,

            chromosome: None,
            sid: None,
            cm_position: None,
            bp_position: None,
            allele_1: None,
            allele_2: None,

            is_a1_counted: None,
            num_threads: None,
            missing_value: None,
        }
    }

    pub fn fam_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.fam_path = Some(Some(path.as_ref().into()));
        self
    }

    pub fn bim_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.bim_path = Some(Some(path.as_ref().into()));
        self
    }

    // !!!cmk later can we also extract a metadata property from write options?
    pub fn metadata(mut self, metadata: &Metadata) -> Self {
        if let Skippable::Some(fid) = &metadata.fid {
            self.fid = Some(Skippable::Some((*fid).clone()));
        }
        if let Skippable::Some(iid) = &metadata.iid {
            self.iid = Some(Skippable::Some((*iid).clone()));
        }
        if let Skippable::Some(father) = &metadata.father {
            self.father = Some(Skippable::Some((*father).clone()));
        }
        if let Skippable::Some(mother) = &metadata.mother {
            self.mother = Some(Skippable::Some((*mother).clone()));
        }
        if let Skippable::Some(sex) = &metadata.sex {
            self.sex = Some(Skippable::Some((*sex).clone()));
        }
        if let Skippable::Some(pheno) = &metadata.pheno {
            self.pheno = Some(Skippable::Some((*pheno).clone()));
        }

        if let Skippable::Some(chromosome) = &metadata.chromosome {
            self.chromosome = Some(Skippable::Some((*chromosome).clone()));
        }
        if let Skippable::Some(sid) = &metadata.sid {
            self.sid = Some(Skippable::Some((*sid).clone()));
        }
        if let Skippable::Some(cm_position) = &metadata.cm_position {
            self.cm_position = Some(Skippable::Some((*cm_position).clone()));
        }
        if let Skippable::Some(bp_position) = &metadata.bp_position {
            self.bp_position = Some(Skippable::Some((*bp_position).clone()));
        }
        if let Skippable::Some(allele_1) = &metadata.allele_1 {
            self.allele_1 = Some(Skippable::Some((*allele_1).clone()));
        }
        if let Skippable::Some(allele_2) = &metadata.allele_2 {
            self.allele_2 = Some(Skippable::Some((*allele_2).clone()));
        }
        self
    }

    pub fn fid<I: IntoIterator<Item = T>, T: AsRef<str>>(mut self, fid: I) -> Self {
        self.fid = Some(Skippable::Some(
            fid.into_iter().map(|s| s.as_ref().to_string()).collect(),
        ));
        self
    }

    pub fn iid<I: IntoIterator<Item = T>, T: AsRef<str>>(mut self, iid: I) -> Self {
        self.iid = Some(Skippable::Some(
            iid.into_iter().map(|s| s.as_ref().to_string()).collect(),
        ));
        self
    }
    pub fn father<I: IntoIterator<Item = T>, T: AsRef<str>>(mut self, father: I) -> Self {
        self.father = Some(Skippable::Some(
            father.into_iter().map(|s| s.as_ref().to_string()).collect(),
        ));
        self
    }
    pub fn mother<I: IntoIterator<Item = T>, T: AsRef<str>>(mut self, mother: I) -> Self {
        self.mother = Some(Skippable::Some(
            mother.into_iter().map(|s| s.as_ref().to_string()).collect(),
        ));
        self
    }

    pub fn sex<I: IntoIterator<Item = i32>>(mut self, sex: I) -> Self {
        self.sex = Some(Skippable::Some(sex.into_iter().map(|i| i).collect()));
        self
    }

    pub fn pheno<I: IntoIterator<Item = T>, T: AsRef<str>>(mut self, pheno: I) -> Self {
        self.pheno = Some(Skippable::Some(
            pheno.into_iter().map(|s| s.as_ref().to_string()).collect(),
        ));
        self
    }

    pub fn chromosome<I: IntoIterator<Item = T>, T: AsRef<str>>(mut self, chromosome: I) -> Self {
        self.chromosome = Some(Skippable::Some(
            chromosome
                .into_iter()
                .map(|s| s.as_ref().to_string())
                .collect(),
        ));
        self
    }

    pub fn sid<I: IntoIterator<Item = T>, T: AsRef<str>>(mut self, sid: I) -> Self {
        self.sid = Some(Skippable::Some(
            sid.into_iter().map(|s| s.as_ref().to_string()).collect(),
        ));
        self
    }

    pub fn cm_position<I: IntoIterator<Item = f32>>(mut self, cm_position: I) -> Self {
        self.cm_position = Some(Skippable::Some(
            cm_position.into_iter().map(|s| s).collect(),
        ));
        self
    }

    pub fn bp_position<I: IntoIterator<Item = i32>>(mut self, bp_position: I) -> Self {
        self.bp_position = Some(Skippable::Some(
            bp_position.into_iter().map(|s| s).collect(),
        ));
        self
    }

    pub fn allele_1<I: IntoIterator<Item = T>, T: AsRef<str>>(mut self, allele_1: I) -> Self {
        self.allele_1 = Some(Skippable::Some(
            allele_1
                .into_iter()
                .map(|s| s.as_ref().to_string())
                .collect(),
        ));
        self
    }

    pub fn allele_2<I: IntoIterator<Item = T>, T: AsRef<str>>(mut self, allele_2: I) -> Self {
        self.allele_2 = Some(Skippable::Some(
            allele_2
                .into_iter()
                .map(|s| s.as_ref().to_string())
                .collect(),
        ));
        self
    }

    pub fn count_a1(&mut self) -> &mut Self {
        self.is_a1_counted = Some(true);
        self
    }

    pub fn count_a2(&mut self) -> &mut Self {
        self.is_a1_counted = Some(false);
        self
    }
}

// !!!cmk later do this without a "clone"
fn compute_field<T, F>(field: &Skippable<nd::Array1<T>>, count: usize, lambda: F) -> nd::Array1<T>
where
    T: Clone + Default + Debug,
    F: Fn(usize) -> T,
{
    match field {
        Skippable::Some(array) => array.clone(),
        Skippable::Skip => (0..count).map(|_| lambda(0)).collect::<nd::Array1<T>>(),
    }
}

fn bim_write_internal<TVal: BedVal>(
    write_options: &WriteOptions<TVal>,
    sid_count: usize,
    bim_path: &PathBuf,
) -> Result<(), BedErrorPlus> {
    let chromosome = compute_field(&write_options.chromosome, sid_count, |_| "0".to_string());
    let sid = compute_field(&write_options.sid, sid_count, |i| format!("sid{}", i + 1));
    let cm_position = compute_field(&write_options.cm_position, sid_count, |_| 0.0);
    let bp_position = compute_field(&write_options.bp_position, sid_count, |_| 0);
    let allele_1 = compute_field(&write_options.allele_1, sid_count, |_| "A1".to_string());
    let allele_2 = compute_field(&write_options.allele_2, sid_count, |_| "A2".to_string());
    let file = File::create(bim_path)?;
    let mut writer = BufWriter::new(file);
    let mut result: Result<(), BedErrorPlus> = Ok(());
    nd::azip!((chromosome in &chromosome, sid in &sid, cm_position in &cm_position, bp_position in &bp_position, allele_1 in &allele_1, allele_2 in &allele_2)
    {
        // !!!cmk later should these be \t?
        if result.is_ok() {
            if let Err(e) = writeln!(
            writer,
            "{}\t{}\t{}\t{}\t{}\t{}",
            *chromosome, *sid, *cm_position, *bp_position, *allele_1, *allele_2
        )
        {
         result = Err(BedErrorPlus::IOError(e)); // !!!cmk later test this
        }
     }});
    result?;
    Ok(())
}

fn fam_write_internal<TVal: BedVal>(
    write_options: &WriteOptions<TVal>,
    iid_count: usize,
    fam_path: &PathBuf,
) -> Result<(), BedErrorPlus> {
    let fid = compute_field(&write_options.fid, iid_count, |_| "0".to_string());
    let iid = compute_field(&write_options.iid, iid_count, |i| format!("iid{}", i + 1));
    let father = compute_field(&write_options.father, iid_count, |_| "0".to_string());
    let mother = compute_field(&write_options.mother, iid_count, |_| "0".to_string());
    let sex = compute_field(&write_options.sex, iid_count, |_| 0);
    let pheno = compute_field(&write_options.pheno, iid_count, |_| "0".to_string());
    let file = File::create(fam_path)?;
    let mut writer = BufWriter::new(file);
    let mut result: Result<(), BedErrorPlus> = Ok(());
    nd::azip!((fid in &fid, iid in &iid, father in &father, mother in &mother, sex in &sex, pheno in &pheno)
    {
        if result.is_ok() {
            if let Err(e) = writeln!(
            writer,
            "{}\t{}\t{}\t{}\t{}\t{}",
            *fid, *iid, *father, *mother, *sex, *pheno
        )
        {
         result = Err(BedErrorPlus::IOError(e)); // !!!cmk later test this
        }
     }});
    result?;
    Ok(())
}

trait FromStringArray<T> {
    fn from_string_array(
        string_array: nd::Array1<String>,
    ) -> Result<nd::Array1<Self>, BedErrorPlus>
    where
        Self: Sized;
}

impl FromStringArray<String> for String {
    fn from_string_array(
        string_array: nd::Array1<String>,
    ) -> Result<nd::Array1<String>, BedErrorPlus> {
        Ok(string_array)
    }
}

// !!!cmk later test these
impl FromStringArray<f32> for f32 {
    fn from_string_array(
        string_array: nd::Array1<String>,
    ) -> Result<nd::Array1<f32>, BedErrorPlus> {
        let result = string_array
            .iter()
            .map(|s| s.parse::<f32>())
            .collect::<Result<nd::Array1<f32>, _>>();
        match result {
            Ok(array) => Ok(array),
            Err(e) => Err(BedErrorPlus::ParseFloatError(e)),
        }
    }
}
impl FromStringArray<i32> for i32 {
    fn from_string_array(
        string_array: nd::Array1<String>,
    ) -> Result<nd::Array1<i32>, BedErrorPlus> {
        let result = string_array
            .iter()
            .map(|s| s.parse::<i32>())
            .collect::<Result<nd::Array1<i32>, _>>();
        match result {
            Ok(array) => Ok(array),
            Err(e) => Err(BedErrorPlus::ParseIntError(e)),
        }
    }
}

pub fn assert_eq_nan<T: 'static + Copy + PartialEq + PartialOrd + Signed + From<i8>>(
    val: &nd::ArrayBase<nd::OwnedRepr<T>, nd::Dim<[usize; 2]>>,
    answer: &nd::ArrayBase<nd::OwnedRepr<T>, nd::Dim<[usize; 2]>>,
) {
    assert!(allclose::<T, T>(
        &val.view(),
        &answer.view(),
        0.into(),
        true
    ));
}

pub fn allclose<
    T1: 'static + Copy + PartialEq + PartialOrd + Signed,
    T2: 'static + Copy + PartialEq + PartialOrd + Signed + Into<T1>,
>(
    val1: &nd::ArrayView2<'_, T1>,
    val2: &nd::ArrayView2<'_, T2>,
    atol: T1,
    equal_nan: bool,
) -> bool {
    assert!(val1.dim() == val2.dim());
    // Could be run in parallel

    nd::Zip::from(val1)
        .and(val2)
        .fold(true, |acc, ptr_a, ptr_b| -> bool {
            if !acc {
                return false;
            }
            // x != x is a generic nan check
            #[allow(clippy::eq_op)]
            let a_nan = *ptr_a != *ptr_a;
            #[allow(clippy::eq_op)]
            let b_nan = *ptr_b != *ptr_b;

            if a_nan || b_nan {
                if equal_nan {
                    a_nan == b_nan
                } else {
                    false
                }
            } else {
                let c: T1 = abs(*ptr_a - T2::into(*ptr_b));
                c <= atol
            }
        })
}

// cmk 0 document
pub fn tmp_path() -> Result<PathBuf, BedErrorPlus> {
    let output_path = PathBuf::from(TempDir::default().as_ref());
    fs::create_dir(&output_path)?;
    Ok(output_path)
}
