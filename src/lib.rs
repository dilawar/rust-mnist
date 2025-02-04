#![warn(clippy::pedantic)]
//! A simple struct build by parsing the MNIST dataset.

use log::info;
use std::convert::TryFrom;
use std::fs;
use std::io;
use std::io::Read;
use std::path::PathBuf;

// Filenames
#[allow(dead_code)]
const TRAIN_DATA_FILENAME: &str = "train-images-idx3-ubyte";
const TEST_DATA_FILENAME: &str = "t10k-images-idx3-ubyte";
const TRAIN_LABEL_FILENAME: &str = "train-labels-idx1-ubyte";
const TEST_LABEL_FILENAME: &str = "t10k-labels-idx1-ubyte";

// Constants relating to the MNIST dataset. All usize for array/vec indexing.
const IMAGES_MAGIC_NUMBER: usize = 2051;
const LABELS_MAGIC_NUMBER: usize = 2049;
const NUM_TRAIN_IMAGES: usize = 60_000;
const NUM_TEST_IMAGES: usize = 10_000;
const IMAGE_ROWS: usize = 28;
const IMAGE_COLUMNS: usize = 28;

pub struct Mnist {
    // Arrays of images.
    pub train_data: Vec<[u8; IMAGE_ROWS * IMAGE_COLUMNS]>,
    pub test_data: Vec<[u8; IMAGE_ROWS * IMAGE_COLUMNS]>,

    // Arrays of labels.
    pub train_labels: Vec<u8>,
    pub test_labels: Vec<u8>,
}

impl Mnist {
    /// Load MNIST dataset.
    ///
    /// # Panics
    ///
    /// Panics if the MNIST dataset is not present at the specified path, or if the dataset is
    /// malformed.
    #[must_use]
    pub fn new(mnist_path: &PathBuf) -> Mnist {
        // Get Training Data.
        info!("Reading MNIST training data.");
        let data_filepath = mnist_path.join(TRAIN_LABEL_FILENAME);
        let train_data = parse_images(&data_filepath).expect(
            &format!(
                "Training data file \"{}\" not found; did you \
                     remember to download and extract it?",
                data_filepath.to_string_lossy(),
            )[..],
        );

        // Assert that numbers extracted from the file were as expected.
        assert_eq!(
            train_data.magic_number, IMAGES_MAGIC_NUMBER,
            "Magic number for training data does not match expected value."
        );
        assert_eq!(
            train_data.num_images, NUM_TRAIN_IMAGES,
            "Number of images in training data does not match expected value."
        );
        assert_eq!(
            train_data.num_rows, IMAGE_ROWS,
            "Number of rows per image in training data does not match expected value."
        );
        assert_eq!(
            train_data.num_cols, IMAGE_COLUMNS,
            "Number of columns per image in training data does not match expected value."
        );

        // Get Testing Data.
        info!("Reading MNIST testing data.");
        let test_filepath = mnist_path.join(TEST_DATA_FILENAME);
        let test_data = parse_images(&test_filepath).expect(
            &format!(
                "Test data file \"{}\" not found; did you \
                     remember to download and extract it?",
                test_filepath.display()
            )[..],
        );

        // Assert that numbers extracted from the file were as expected.
        assert_eq!(
            test_data.magic_number, IMAGES_MAGIC_NUMBER,
            "Magic number for testing data does not match expected value."
        );
        assert_eq!(
            test_data.num_images, NUM_TEST_IMAGES,
            "Number of images in testing data does not match expected value."
        );
        assert_eq!(
            test_data.num_rows, IMAGE_ROWS,
            "Number of rows per image in testing data does not match expected value."
        );
        assert_eq!(
            test_data.num_cols, IMAGE_COLUMNS,
            "Number of columns per image in testing data does not match expected value."
        );

        // Get Training Labels.
        info!("Reading MNIST training labels.");
        let train_filepath = mnist_path.join(TRAIN_LABEL_FILENAME);
        let (magic_number, num_labels, train_labels) = parse_labels(&train_filepath).expect(
            &format!(
                "Training label file \"{}\" not found; did you \
                     remember to download and extract it?",
                train_filepath.display()
            )[..],
        );

        // Assert that numbers extracted from the file were as expected.
        assert_eq!(
            magic_number, LABELS_MAGIC_NUMBER,
            "Magic number for training labels does not match expected value."
        );
        assert_eq!(
            num_labels, NUM_TRAIN_IMAGES,
            "Number of labels in training labels does not match expected value."
        );

        // Get Testing Labels.
        info!("Reading MNIST testing labels.");
        let test_filepath = mnist_path.join(TEST_LABEL_FILENAME);
        let (magic_number, num_labels, test_labels) = parse_labels(&test_filepath).expect(
            &format!(
                "Test labels file \"{}\" not found; did you \
                     remember to download and extract it?",
                test_filepath.to_string_lossy()
            )[..],
        );

        // Assert that numbers extracted from the file were as expected.
        assert_eq!(
            magic_number, LABELS_MAGIC_NUMBER,
            "Magic number for testing labels does not match expected value."
        );
        assert_eq!(
            num_labels, NUM_TEST_IMAGES,
            "Number of labels in testing labels does not match expected value."
        );

        Mnist {
            train_data: train_data.images,
            test_data: test_data.images,
            train_labels,
            test_labels,
        }
    }
}

/// Print a sample image.
///
/// # Examples
/// ```
/// use std::path::PathBuf;
/// use rust_mnist::{print_image, Mnist};
///
/// let mnist = Mnist::new(&PathBuf::from("examples").join("MNIST_data"));
///
/// // Print one image (the one at index 5).
/// print_image(&mnist.train_data[5], mnist.train_labels[5]);
/// ```
pub fn print_image(image: &[u8; IMAGE_ROWS * IMAGE_COLUMNS], label: u8) {
    println!("Sample image label: {label} \nSample image:");

    // Print each row.
    for row in 0..IMAGE_ROWS {
        for col in 0..IMAGE_COLUMNS {
            if image[row * IMAGE_COLUMNS + col] == 0 {
                print!("__");
            } else {
                print!("##");
            }
        }
        println!();
    }
}

struct MnistImages {
    magic_number: usize,
    num_images: usize,
    num_rows: usize,
    num_cols: usize,
    images: Vec<[u8; IMAGE_ROWS * IMAGE_COLUMNS]>,
}

fn parse_images(filename: &PathBuf) -> io::Result<MnistImages> {
    // Open the file.
    let images_data_bytes = fs::File::open(filename)?;
    let images_data_bytes = io::BufReader::new(images_data_bytes);
    let mut buffer_32: [u8; 4] = [0; 4];

    // Get the magic number.
    images_data_bytes
        .get_ref()
        .take(4)
        .read_exact(&mut buffer_32)?;
    let magic_number = usize::try_from(u32::from_be_bytes(buffer_32)).unwrap();

    // Get number of images.
    images_data_bytes
        .get_ref()
        .take(4)
        .read_exact(&mut buffer_32)?;
    let num_images = usize::try_from(u32::from_be_bytes(buffer_32)).unwrap();

    // Get number or rows per image.
    images_data_bytes
        .get_ref()
        .take(4)
        .read_exact(&mut buffer_32)?;
    let num_rows = usize::try_from(u32::from_be_bytes(buffer_32)).unwrap();

    // Get number or columns per image.
    images_data_bytes
        .get_ref()
        .take(4)
        .read_exact(&mut buffer_32)?;
    let num_cols = usize::try_from(u32::from_be_bytes(buffer_32)).unwrap();

    // Buffer for holding image pixels.
    let mut image_buffer: [u8; IMAGE_ROWS * IMAGE_COLUMNS] = [0; IMAGE_ROWS * IMAGE_COLUMNS];

    // Vector to hold all images in the file.
    let mut images: Vec<[u8; IMAGE_ROWS * IMAGE_COLUMNS]> = Vec::with_capacity(num_images);

    // Get images from file.
    for _image in 0..num_images {
        images_data_bytes
            .get_ref()
            .take(u64::try_from(num_rows * num_cols).unwrap())
            .read_exact(&mut image_buffer)
            .unwrap();
        images.push(image_buffer);
    }

    Ok(MnistImages {
        magic_number,
        num_images,
        num_rows,
        num_cols,
        images,
    })
}

fn parse_labels(filename: &PathBuf) -> io::Result<(usize, usize, Vec<u8>)> {
    let labels_data_bytes = fs::File::open(filename)?;
    let labels_data_bytes = io::BufReader::new(labels_data_bytes);
    let mut buffer_32: [u8; 4] = [0; 4];

    // Get the magic number.
    labels_data_bytes
        .get_ref()
        .take(4)
        .read_exact(&mut buffer_32)
        .unwrap();
    let magic_number = usize::try_from(u32::from_be_bytes(buffer_32)).unwrap();

    // Get number of labels.
    labels_data_bytes
        .get_ref()
        .take(4)
        .read_exact(&mut buffer_32)
        .unwrap();
    let num_labels = usize::try_from(u32::from_be_bytes(buffer_32)).unwrap();

    // Buffer for holding image label.
    let mut label_buffer: [u8; 1] = [0; 1];

    // Vector to hold all labels in the file.
    let mut labels: Vec<u8> = Vec::with_capacity(num_labels);

    // Get labels from file.
    for _label in 0..num_labels {
        labels_data_bytes
            .get_ref()
            .take(1)
            .read_exact(&mut label_buffer)
            .unwrap();
        labels.push(label_buffer[0]);
    }
    Ok((magic_number, num_labels, labels))
}
