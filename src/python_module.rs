#![cfg(feature = "extension-module")]

use std::collections::HashMap;

use numpy::{PyArray1, PyArray2, PyArray3};
use object_store::{path::Path as StorePath, ObjectStore};

use crate::{
    BedError, BedErrorPlus, Dist, _file_ata_piece_internal, create_pool, file_aat_piece,
    file_ata_piece, file_b_less_aatbx, impute_and_zero_mean_snps, matrix_subset_no_alloc,
    read_into_f32, read_into_f64, Bed, BedCloud, ObjectPath, ReadOptions, WriteOptions,
};
use pyo3::{
    exceptions::PyIOError,
    exceptions::PyIndexError,
    exceptions::PyValueError,
    prelude::{pymodule, PyModule, PyResult, Python},
    PyErr,
};
use tokio::runtime;
use url::Url;

#[pymodule]
#[allow(clippy::too_many_lines, clippy::items_after_statements)]
fn bed_reader(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    // See User's guide: https://pyo3.rs/v0.15.1/
    // mutable example (no return) see https://github.com/PyO3/rust-numpy
    // https://pyo3.rs/v0.13.1/exception.html

    mod io {
        pyo3::import_exception!(io, UnsupportedOperation);
    }

    impl std::convert::From<Box<BedErrorPlus>> for PyErr {
        fn from(err: Box<BedErrorPlus>) -> PyErr {
            match *err {
                BedErrorPlus::BedError(
                    BedError::IidIndexTooBig(_)
                    | BedError::SidIndexTooBig(_)
                    | BedError::IndexMismatch(_, _, _, _)
                    | BedError::IndexesTooBigForFiles(_, _)
                    | BedError::SubsetMismatch(_, _, _, _),
                ) => PyIndexError::new_err(err.to_string()),

                BedErrorPlus::IOError(_) => PyIOError::new_err(err.to_string()),

                _ => PyValueError::new_err(err.to_string()),
            }
        }
    }

    #[pyfn(m)]
    fn url_to_bytes(location: &str, options: HashMap<&str, String>) -> Result<Vec<u8>, PyErr> {
        let rt = runtime::Runtime::new()?;

        let url = Url::parse(location).unwrap(); // cmk return a BedReader URL parse error
        let (object_store, store_path): (Box<dyn ObjectStore>, StorePath) =
            object_store::parse_url_opts(&url, options).unwrap(); // cmk return a BedReader URL parse error
        let object_path: ObjectPath<Box<dyn ObjectStore>> = (object_store, store_path).into();

        rt.block_on(async {
            let get_result = object_path.get().await?;
            let bytes = get_result.bytes().await.unwrap(); // cmk ???
            let vec: Vec<u8> = bytes.to_vec();
            Ok(vec)
        })
    }

    #[pyfn(m)]
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::needless_pass_by_value)]
    fn read_f64(
        filename: &str,
        _ignore: HashMap<&str, String>,
        iid_count: usize,
        sid_count: usize,
        is_a1_counted: bool,
        iid_index: &PyArray1<isize>,
        sid_index: &PyArray1<isize>,
        val: &PyArray2<f64>,
        num_threads: usize,
    ) -> Result<(), PyErr> {
        let iid_index = iid_index.readonly();
        let sid_index = sid_index.readonly();
        let ii = &iid_index.as_slice()?;
        let si = &sid_index.as_slice()?;

        let mut val = val.readwrite();
        let mut val = val.as_array_mut();

        let mut bed = Bed::builder(filename)
            .iid_count(iid_count)
            .sid_count(sid_count)
            .build()?;

        ReadOptions::builder()
            .iid_index(*ii)
            .sid_index(*si)
            .is_a1_counted(is_a1_counted)
            .num_threads(num_threads)
            .read_and_fill(&mut bed, &mut val.view_mut())?;

        Ok(())
    }

    #[pyfn(m)]
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::needless_pass_by_value)]
    fn read_f32(
        filename: &str,
        _ignore: HashMap<&str, String>,
        iid_count: usize,
        sid_count: usize,
        is_a1_counted: bool,
        iid_index: &PyArray1<isize>,
        sid_index: &PyArray1<isize>,
        val: &PyArray2<f32>,
        num_threads: usize,
    ) -> Result<(), PyErr> {
        let iid_index = iid_index.readonly();
        let sid_index = sid_index.readonly();
        let ii = &iid_index.as_slice()?;
        let si = &sid_index.as_slice()?;

        let mut val = val.readwrite();
        let mut val = val.as_array_mut();

        let mut bed = Bed::builder(filename)
            .iid_count(iid_count)
            .sid_count(sid_count)
            .build()?;

        ReadOptions::builder()
            .iid_index(*ii)
            .sid_index(*si)
            .is_a1_counted(is_a1_counted)
            .num_threads(num_threads)
            .read_and_fill(&mut bed, &mut val.view_mut())?;

        Ok(())
    }

    #[pyfn(m)]
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::needless_pass_by_value)]
    fn read_i8(
        filename: &str,
        _ignore: HashMap<&str, String>,
        iid_count: usize,
        sid_count: usize,
        is_a1_counted: bool,
        iid_index: &PyArray1<isize>,
        sid_index: &PyArray1<isize>,
        val: &PyArray2<i8>,
        num_threads: usize,
    ) -> Result<(), PyErr> {
        let iid_index = iid_index.readonly();
        let sid_index = sid_index.readonly();
        let ii = &iid_index.as_slice()?;
        let si = &sid_index.as_slice()?;

        let mut val = val.readwrite();
        let mut val = val.as_array_mut();

        let mut bed = Bed::builder(filename)
            .iid_count(iid_count)
            .sid_count(sid_count)
            .build()?;

        ReadOptions::builder()
            .iid_index(*ii)
            .sid_index(*si)
            .is_a1_counted(is_a1_counted)
            .num_threads(num_threads)
            .read_and_fill(&mut bed, &mut val.view_mut())?;

        Ok(())
    }

    #[pyfn(m)]
    #[allow(clippy::too_many_arguments)]
    fn check_file_cloud(url: &str, options: HashMap<&str, String>) -> Result<(), PyErr> {
        let rt = runtime::Runtime::new().unwrap(); // cmk unwrap?

        let url = Url::parse(url).unwrap(); // cmk return a BedReader URL parse error
        let (object_store, store_path): (Box<dyn ObjectStore>, StorePath) =
            object_store::parse_url_opts(&url, options).unwrap(); // cmk return a BedReader URL parse error
        let object_path: ObjectPath<Box<dyn ObjectStore>> = (object_store, store_path).into();

        rt.block_on(async {
            BedCloud::builder(object_path).build().await?;
            Ok(())
        })
    }

    #[pyfn(m)]
    #[allow(clippy::too_many_arguments)]
    fn read_cloud_i8(
        url: &str,
        options: HashMap<&str, String>,
        iid_count: usize,
        sid_count: usize,
        is_a1_counted: bool,
        iid_index: &PyArray1<isize>,
        sid_index: &PyArray1<isize>,
        val: &PyArray2<i8>,
        num_threads: usize,
    ) -> Result<(), PyErr> {
        let iid_index = iid_index.readonly();
        let sid_index = sid_index.readonly();
        let ii = &iid_index.as_slice()?;
        let si = &sid_index.as_slice()?;

        let mut val = val.readwrite();
        let mut val = val.as_array_mut();

        let rt = runtime::Runtime::new().unwrap(); // cmk unwrap?

        let url = Url::parse(url).unwrap(); // cmk return a BedReader URL parse error
        let (object_store, store_path): (Box<dyn ObjectStore>, StorePath) =
            object_store::parse_url_opts(&url, options).unwrap(); // cmk return a BedReader URL parse error
        let object_path: ObjectPath<Box<dyn ObjectStore>> = (object_store, store_path).into();

        rt.block_on(async {
            let mut bed_cloud = BedCloud::builder(object_path)
                .iid_count(iid_count)
                .sid_count(sid_count)
                .build()
                .await?;

            ReadOptions::builder()
                .iid_index(*ii)
                .sid_index(*si)
                .is_a1_counted(is_a1_counted)
                .num_threads(num_threads)
                .read_and_fill_cloud(&mut bed_cloud, &mut val.view_mut())
                .await?;

            Ok(())
        })
    }

    #[pyfn(m)]
    #[allow(clippy::too_many_arguments)]
    fn read_cloud_f32(
        url: &str,
        options: HashMap<&str, String>,
        iid_count: usize,
        sid_count: usize,
        is_a1_counted: bool,
        iid_index: &PyArray1<isize>,
        sid_index: &PyArray1<isize>,
        val: &PyArray2<f32>,
        num_threads: usize,
    ) -> Result<(), PyErr> {
        let iid_index = iid_index.readonly();
        let sid_index = sid_index.readonly();
        let ii = &iid_index.as_slice()?;
        let si = &sid_index.as_slice()?;

        let mut val = val.readwrite();
        let mut val = val.as_array_mut();

        let rt = runtime::Runtime::new().unwrap(); // cmk unwrap?

        let url = Url::parse(url).unwrap(); // cmk return a BedReader URL parse error
        let (object_store, store_path): (Box<dyn ObjectStore>, StorePath) =
            object_store::parse_url_opts(&url, options).unwrap(); // cmk return a BedReader URL parse error
        let object_path: ObjectPath<Box<dyn ObjectStore>> = (object_store, store_path).into();

        rt.block_on(async {
            let mut bed_cloud = BedCloud::builder(object_path)
                .iid_count(iid_count)
                .sid_count(sid_count)
                .build()
                .await?;

            ReadOptions::builder()
                .iid_index(*ii)
                .sid_index(*si)
                .is_a1_counted(is_a1_counted)
                .num_threads(num_threads)
                .read_and_fill_cloud(&mut bed_cloud, &mut val.view_mut())
                .await?;

            Ok(())
        })
    }

    #[pyfn(m)]
    #[allow(clippy::too_many_arguments)]
    fn read_cloud_f64(
        url: &str,
        options: HashMap<&str, String>,
        iid_count: usize,
        sid_count: usize,
        is_a1_counted: bool,
        iid_index: &PyArray1<isize>,
        sid_index: &PyArray1<isize>,
        val: &PyArray2<f64>,
        num_threads: usize,
    ) -> Result<(), PyErr> {
        let iid_index = iid_index.readonly();
        let sid_index = sid_index.readonly();
        let ii = &iid_index.as_slice()?;
        let si = &sid_index.as_slice()?;

        let mut val = val.readwrite();
        let mut val = val.as_array_mut();

        let rt = runtime::Runtime::new().unwrap(); // cmk unwrap?

        let url = Url::parse(url).unwrap(); // cmk return a BedReader URL parse error
        let (object_store, store_path): (Box<dyn ObjectStore>, StorePath) =
            object_store::parse_url_opts(&url, options).unwrap(); // cmk return a BedReader URL parse error
        let object_path: ObjectPath<Box<dyn ObjectStore>> = (object_store, store_path).into();

        rt.block_on(async {
            let mut bed_cloud = BedCloud::builder(object_path)
                .iid_count(iid_count)
                .sid_count(sid_count)
                .build()
                .await?;

            ReadOptions::builder()
                .iid_index(*ii)
                .sid_index(*si)
                .is_a1_counted(is_a1_counted)
                .num_threads(num_threads)
                .read_and_fill_cloud(&mut bed_cloud, &mut val.view_mut())
                .await?;

            Ok(())
        })
    }

    #[pyfn(m)]
    fn write_f64(
        filename: &str,
        is_a1_counted: bool,
        val: &PyArray2<f64>,
        num_threads: usize,
    ) -> Result<(), PyErr> {
        let mut val = val.readwrite();
        let val = val.as_array_mut();

        WriteOptions::builder(filename)
            .is_a1_counted(is_a1_counted)
            .num_threads(num_threads)
            .skip_fam()
            .skip_bim()
            .write(&val)?;

        Ok(())
    }

    #[pyfn(m)]
    fn write_f32(
        filename: &str,
        is_a1_counted: bool,
        val: &PyArray2<f32>,
        num_threads: usize,
    ) -> Result<(), PyErr> {
        let mut val = val.readwrite();
        let val = val.as_array_mut();

        WriteOptions::builder(filename)
            .is_a1_counted(is_a1_counted)
            .num_threads(num_threads)
            .skip_fam()
            .skip_bim()
            .write(&val)?;

        Ok(())
    }

    #[pyfn(m)]
    fn write_i8(
        filename: &str,
        is_a1_counted: bool,
        val: &PyArray2<i8>,
        num_threads: usize,
    ) -> Result<(), PyErr> {
        let mut val = val.readwrite();
        let val = val.as_array_mut();

        WriteOptions::builder(filename)
            .is_a1_counted(is_a1_counted)
            .num_threads(num_threads)
            .skip_fam()
            .skip_bim()
            .write(&val)?;

        Ok(())
    }

    #[pyfn(m)]
    fn subset_f64_f64(
        val_in: &PyArray3<f64>,
        iid_index: &PyArray1<usize>,
        sid_index: &PyArray1<usize>,
        val_out: &PyArray3<f64>,
        num_threads: usize,
    ) -> Result<(), PyErr> {
        let iid_index = iid_index.readonly();
        let sid_index = sid_index.readonly();

        let val_in = val_in.readwrite();
        let val_in = val_in.as_array();
        let mut val_out = val_out.readwrite();
        let mut val_out = val_out.as_array_mut();

        let ii = &iid_index.as_slice()?;
        let si = &sid_index.as_slice()?;

        create_pool(num_threads)?
            .install(|| matrix_subset_no_alloc(&val_in, ii, si, &mut val_out))?;

        Ok(())
    }

    #[pyfn(m)]
    fn subset_f32_f64(
        val_in: &PyArray3<f32>,
        iid_index: &PyArray1<usize>,
        sid_index: &PyArray1<usize>,
        val_out: &PyArray3<f64>,
        num_threads: usize,
    ) -> Result<(), PyErr> {
        let iid_index = iid_index.readonly();
        let sid_index = sid_index.readonly();

        let val_in = val_in.readwrite();
        let val_in = val_in.as_array();
        let mut val_out = val_out.readwrite();
        let mut val_out = val_out.as_array_mut();

        let ii = &iid_index.as_slice()?;
        let si = &sid_index.as_slice()?;

        create_pool(num_threads)?
            .install(|| matrix_subset_no_alloc(&val_in, ii, si, &mut val_out))?;

        Ok(())
    }

    #[pyfn(m)]
    fn subset_f32_f32(
        val_in: &PyArray3<f32>,
        iid_index: &PyArray1<usize>,
        sid_index: &PyArray1<usize>,
        val_out: &PyArray3<f32>,
        num_threads: usize,
    ) -> Result<(), PyErr> {
        let iid_index = iid_index.readonly();
        let sid_index = sid_index.readonly();

        let val_in = val_in.readwrite();
        let val_in = val_in.as_array();
        let mut val_out = val_out.readwrite();
        let mut val_out = val_out.as_array_mut();

        let ii = &iid_index.as_slice()?;
        let si = &sid_index.as_slice()?;

        create_pool(num_threads)?
            .install(|| matrix_subset_no_alloc(&val_in, ii, si, &mut val_out))?;

        Ok(())
    }

    #[pyfn(m)]
    #[allow(clippy::too_many_arguments)]
    fn standardize_f32(
        val: &PyArray2<f32>,
        beta_not_unit_variance: bool,
        beta_a: f64,
        beta_b: f64,
        apply_in_place: bool,
        use_stats: bool,
        stats: &PyArray2<f32>,
        num_threads: usize,
    ) -> Result<(), PyErr> {
        let mut val = val.readwrite();
        let mut val = val.as_array_mut();
        let mut stats = stats.readwrite();
        let mut stats = stats.as_array_mut();
        let dist = create_dist(beta_not_unit_variance, beta_a, beta_b);
        create_pool(num_threads)?.install(|| {
            impute_and_zero_mean_snps(
                &mut val.view_mut(),
                &dist,
                apply_in_place,
                use_stats,
                &mut stats.view_mut(),
            )
        })?;
        Ok(())
    }

    fn create_dist(beta_not_unit_variance: bool, a: f64, b: f64) -> Dist {
        if beta_not_unit_variance {
            Dist::Beta { a, b }
        } else {
            Dist::Unit
        }
    }

    #[pyfn(m)]
    #[allow(clippy::too_many_arguments)]
    fn standardize_f64(
        val: &PyArray2<f64>,
        beta_not_unit_variance: bool,
        beta_a: f64,
        beta_b: f64,
        apply_in_place: bool,
        use_stats: bool,
        stats: &PyArray2<f64>,
        num_threads: usize,
    ) -> Result<(), PyErr> {
        let mut val = val.readwrite();
        let mut val = val.as_array_mut();
        let mut stats = stats.readwrite();
        let mut stats = stats.as_array_mut();
        let dist = create_dist(beta_not_unit_variance, beta_a, beta_b);

        create_pool(num_threads)?.install(|| {
            impute_and_zero_mean_snps(
                &mut val.view_mut(),
                &dist,
                apply_in_place,
                use_stats,
                &mut stats.view_mut(),
            )
        })?;
        Ok(())
    }

    #[pyfn(m)]
    #[allow(clippy::too_many_arguments)]
    fn file_ata_piece_f32_orderf(
        filename: &str,
        offset: u64,
        row_count: usize,
        col_count: usize,
        col_start: usize,
        ata_piece: &PyArray2<f32>,
        num_threads: usize,
        log_frequency: usize,
    ) -> Result<(), PyErr> {
        let mut ata_piece = ata_piece.readwrite();
        let mut ata_piece = ata_piece.as_array_mut();

        create_pool(num_threads)?.install(|| {
            file_ata_piece(
                filename,
                offset,
                row_count,
                col_count,
                col_start,
                &mut ata_piece,
                log_frequency,
                read_into_f32,
            )
        })?;

        Ok(())
    }

    #[pyfn(m)]
    #[allow(clippy::too_many_arguments)]
    fn file_ata_piece_f64_orderf(
        filename: &str,
        offset: u64,
        row_count: usize,
        col_count: usize,
        col_start: usize,
        ata_piece: &PyArray2<f64>,
        num_threads: usize,
        log_frequency: usize,
    ) -> Result<(), PyErr> {
        let mut ata_piece = ata_piece.readwrite();
        let mut ata_piece = ata_piece.as_array_mut();

        create_pool(num_threads)?.install(|| {
            file_ata_piece(
                filename,
                offset,
                row_count,
                col_count,
                col_start,
                &mut ata_piece,
                log_frequency,
                read_into_f64,
            )
        })?;

        Ok(())
    }

    // Old version of function for backwards compatibility
    #[pyfn(m)]
    fn file_dot_piece(
        filename: &str,
        offset: u64,
        row_count: usize,
        col_start: usize,
        ata_piece: &PyArray2<f64>,
        num_threads: usize,
        log_frequency: usize,
    ) -> Result<(), PyErr> {
        let mut ata_piece = ata_piece.readwrite();
        let mut ata_piece = ata_piece.as_array_mut();

        create_pool(num_threads)?.install(|| {
            _file_ata_piece_internal(
                filename,
                offset,
                row_count,
                col_start,
                &mut ata_piece,
                log_frequency,
                read_into_f64,
            )
        })?;

        Ok(())
    }

    #[pyfn(m)]
    #[allow(clippy::too_many_arguments)]
    fn file_aat_piece_f32_orderf(
        filename: &str,
        offset: u64,
        row_count: usize,
        col_count: usize,
        row_start: usize,
        aat_piece: &PyArray2<f32>,
        num_threads: usize,
        log_frequency: usize,
    ) -> Result<(), PyErr> {
        let mut aat_piece = aat_piece.readwrite();
        let mut aat_piece = aat_piece.as_array_mut();

        create_pool(num_threads)?.install(|| {
            file_aat_piece(
                filename,
                offset,
                row_count,
                col_count,
                row_start,
                &mut aat_piece,
                log_frequency,
                read_into_f32,
            )
        })?;

        Ok(())
    }

    #[pyfn(m)]
    #[allow(clippy::too_many_arguments)]
    fn file_aat_piece_f64_orderf(
        filename: &str,
        offset: u64,
        row_count: usize,
        col_count: usize,
        row_start: usize,
        aat_piece: &PyArray2<f64>,
        num_threads: usize,
        log_frequency: usize,
    ) -> Result<(), PyErr> {
        let mut aat_piece = aat_piece.readwrite();
        let mut aat_piece = aat_piece.as_array_mut();

        create_pool(num_threads)?.install(|| {
            file_aat_piece(
                filename,
                offset,
                row_count,
                col_count,
                row_start,
                &mut aat_piece,
                log_frequency,
                read_into_f64,
            )
        })?;

        Ok(())
    }

    #[pyfn(m)]
    #[pyo3(name = "file_b_less_aatbx")]
    #[allow(clippy::too_many_arguments)]
    fn file_b_less_aatbx_translator(
        a_filename: &str,
        offset: u64,
        iid_count: usize,
        b1: &PyArray2<f64>,
        aatb: &PyArray2<f64>,
        atb: &PyArray2<f64>,
        num_threads: usize,
        log_frequency: usize,
    ) -> Result<(), PyErr> {
        let mut b1 = b1.readwrite();
        let mut b1 = b1.as_array_mut();
        let mut aatb = aatb.readwrite();
        let mut aatb = aatb.as_array_mut();
        let mut atb = atb.readwrite();
        let mut atb = atb.as_array_mut();

        create_pool(num_threads)?.install(|| {
            file_b_less_aatbx(
                a_filename,
                offset,
                iid_count,
                &mut b1,
                &mut aatb,
                &mut atb,
                log_frequency,
            )
        })?;

        Ok(())
    }
    Ok(())
}

// cmk on both rust and python side, when counting bim and fam files, also parse them -- don't read them twice.
