extern crate rand; // For initializing weights.
extern crate rust_mnist;

use rand::distributions::{Distribution, Uniform};
use rust_mnist::{print_image, Mnist};
use std::io::{self, Write};
use std::path::PathBuf;

// Hyperparameter
const LEARNING_RATE: f64 = 0.0001;
const BIAS: f64 = 1.0;

fn main() {
    // Load the dataset into an "Mnist" object. If on windows, replace the forward slashes with
    // backslashes.
    let mnist = Mnist::new(&PathBuf::from("examples").join("MNIST_data"));

    // Print one image (the one at index 5) for verification.
    print_image(&mnist.train_data[5], mnist.train_labels[5]);

    // Generate an array of random weights.
    let mut weights = generate_weights();

    // Training.
    let mut accuracy = 0.0;
    for iter in 0..5 {
        for training_pair in mnist
            .train_data
            .iter()
            .zip(mnist.train_labels.iter())
            .enumerate()
        {
            let (i, pair) = training_pair;
            print!("Epoch: {:2}  Iter: {:5}  ", iter, i);

            // Seperate the image and the label.
            let (image, &label) = pair;

            // Normalize the image.
            let image = normalize(image);

            // Calculate the outputs.
            let mut outputs = dot_product(&image, weights);
            outputs = softmax(&outputs);

            // Calculate the error.
            let error: [f64; 10] = subtract(outputs, one_hot(label));

            // Update rolling-average accuracy.
            accuracy = {
                (accuracy * 999.0 + {
                    if largest(&outputs) == usize::from(label) {
                        1.0
                    } else {
                        0.0
                    }
                }) / 1000.0
            };
            print!("Accuracy: {:.2}\r", accuracy);
            io::stdout().flush().unwrap();

            // Update weights.
            update(&mut weights, &error, &image);
        }
    }
    println!("\nFinal Accuracy: {:.2}", accuracy);
}

fn update(weights: &mut [[f64; 785]; 10], error: &[f64; 10], image: &[f64]) {
    for class_index in 0..error.len() {
        for (input_index, pixel) in image.iter().enumerate() {
            weights[class_index][input_index] -= LEARNING_RATE * error[class_index] * pixel;
            weights[class_index][784] -= LEARNING_RATE * error[class_index] * BIAS;
        }
    }
}

fn generate_weights() -> [[f64; 785]; 10] {
    // Preparing the random number generator before initializing weights.
    let mut rng = rand::thread_rng();
    let dist = Uniform::new_inclusive(0.0, 1.0);

    // Creating a weight array.
    let mut weights: [[f64; 785]; 10] = [[0.0; 785]; 10];

    // Initializing the weights.
    for class_weights in weights.iter_mut() {
        for weight in class_weights.iter_mut() {
            *weight = dist.sample(&mut rng);
        }
    }
    weights
}

fn dot_product(image: &[f64], weights: [[f64; 785]; 10]) -> [f64; 10] {
    let mut outputs: [f64; 10] = [0.0; 10];
    for output_index in 0..outputs.len() {
        for (pixel_index, pixel) in image.iter().enumerate() {
            outputs[output_index] += pixel * weights[output_index][pixel_index];
            outputs[output_index] += BIAS * weights[output_index][784];
        }
    }
    outputs
}

fn subtract(lhs: [f64; 10], rhs: [f64; 10]) -> [f64; 10] {
    let mut difference: [f64; 10] = [0.0; 10];
    for index in 0..difference.len() {
        difference[index] = lhs[index] - rhs[index];
    }
    difference
}

fn one_hot(value: u8) -> [f64; 10] {
    let mut arr: [f64; 10] = [0.0; 10];
    arr[usize::from(value)] = 1.0;
    arr
}

fn normalize(image: &[u8]) -> Vec<f64> {
    // Normalize the image.
    image
        .iter()
        .map(|pixel| 2.0 * f64::from(*pixel) / 255.0 - 1.0)
        .collect()
}

fn largest(arr: &[f64; 10]) -> usize {
    arr.iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|(index, _)| index)
        .unwrap()
}

fn softmax(arr: &[f64; 10]) -> [f64; 10] {
    let exp: Vec<f64> = arr.iter().map(|x| x.exp()).collect();
    let sum_exp: f64 = exp.iter().sum();
    let mut softmax_arr: [f64; 10] = [0.0; 10];
    for index in 0..softmax_arr.len() {
        softmax_arr[index] = exp[index] / sum_exp;
    }
    softmax_arr
}
