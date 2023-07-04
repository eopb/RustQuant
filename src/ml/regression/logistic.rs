// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// RustQuant: A Rust library for quantitative finance tools.
// Copyright (C) 2023 https://github.com/avhz
// See LICENSE or <https://www.gnu.org/licenses/>.
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

//! Module for logistic regression (classification) algorithms.
//!
//! BROKEN: This module is currently broken and does not work.
//! The problem is that the diagonal weights matrix W becomes singular
//! as the diagonal elements approach 0.
//! If you know how to fix this, submit a pull request!

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// IMPORTS
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

use crate::ml::ActivationFunction;
use nalgebra::{DMatrix, DVector};
// use crate::autodiff::{Graph, Variable};

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// STRUCTS, ENUMS, AND TRAITS
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

/// Struct to hold the input data for a logistic regression.
#[derive(Clone, Debug)]
pub struct LogisticRegressionInput<T> {
    /// The input data matrix, also known as the design matrix.
    pub x: DMatrix<T>,
    /// The output data vector, also known as the response vector.
    /// The values of the response vector should be either 0 or 1.
    pub y: DVector<T>,
}

/// Struct to hold the output data for a logistic regression.
#[derive(Clone, Debug)]
pub struct LogisticRegressionOutput<T> {
    /// The coefficients of the logistic regression,
    /// often denoted as b0, b1, b2, ..., bn.
    /// The first coefficient is the intercept (aka. b0 or alpha).
    pub coefficients: DVector<T>,
    /// Number of iterations required to converge.
    pub iterations: usize,
}

/// Algorithm to use for logistic regression.
pub enum LogisticRegressionAlgorithm {
    /// Maximum Likelihood Estimation using Algorithmic Adjoint Differentiation
    /// See: https://en.wikipedia.org/wiki/Logistic_regression#Maximum_likelihood_estimation_(MLE)
    MLE,
    /// Iterative Reweighted Least Squares
    /// From Wikipedia (https://en.wikipedia.org/wiki/Logistic_regression#Iteratively_reweighted_least_squares_(IRLS)):
    /// """
    /// Binary logistic regression can, be calculated using
    /// iteratively reweighted least squares (IRLS), which is equivalent to
    /// maximizing the log-likelihood of a Bernoulli
    /// distributed process using Newton's method.
    /// """
    ///
    /// References:
    ///     - Elements of Statistical Learning (Hastie, Tibshirani, Friedman 2009)
    ///     - Machine Learning: A Probabilistic Perspective (Murphy, Kevin P. 2012)
    IRLS,
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// IMPLEMENTATIONS
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

impl LogisticRegressionInput<f64> {
    /// Create a new `LogisticRegressionInput` struct.
    pub fn new(x: DMatrix<f64>, y: DVector<f64>) -> Self {
        assert!(x.nrows() == y.len());

        Self { x, y }
    }

    /// Function to validate and prepare the input data.
    fn prepare_input(&self) -> Result<(DMatrix<f64>, DMatrix<f64>, DVector<f64>), &'static str> {
        // Check that the response vector is either 0 or 1.
        if self.y.iter().any(|&x| x != 0. && x != 1.) {
            return Err("The elements of the response vector should be either 0 or 1.");
        }

        // Check dimensions match.
        let (n_rows, _) = self.x.shape();

        if n_rows != self.y.len() {
            return Err("The number of rows in the design matrix should match the length of the response vector.");
        }

        // Check the input data is finite.
        if self.x.iter().any(|&x| !x.is_finite()) || self.y.iter().any(|&x| !x.is_finite()) {
            return Err("The input data should be finite.");
        }

        // Add a column of ones to the design matrix.
        let x = self.x.clone().insert_column(0, 1.0);

        // Also return the transpose of the design matrix.
        Ok((x.clone(), x.transpose(), self.y.clone()))
    }

    /// Function to validate and prepare the output data.
    fn prepare_output(&self) -> Result<LogisticRegressionOutput<f64>, &'static str> {
        // Initial guess for the coefficients.
        let guess: f64 = (self.y.mean() / (1. - self.y.mean())).ln();

        // Return the output struct, with the initial guess for the coefficients.
        Ok(LogisticRegressionOutput {
            coefficients: DVector::from_element(self.x.ncols() + 1, guess),
            iterations: 0,
        })
    }

    /// Fit a logistic regression model to the input data.
    pub fn fit(
        &self,
        method: LogisticRegressionAlgorithm,
        tolerance: f64,
    ) -> Result<LogisticRegressionOutput<f64>, &'static str> {
        // Validate and prepare the input data.
        let (X, X_T, y) = self.prepare_input()?;

        // Prepare the output data.
        let mut output = self.prepare_output()?;

        // Number of rows and columns in the design matrix.
        let (n_rows, n_cols) = X.shape();

        // Vector of ones.
        let ones: DVector<f64> = DVector::from_element(n_rows, 1.);

        // Diagonal matrix  of lambdas (tolerance).
        // let lambda = DMatrix::from_diagonal(&DVector::from_element(n_cols, 1e-6));

        // Vector of coefficients that we update each iteration.
        let mut coefs: DVector<f64> = DVector::zeros(n_cols);

        match method {
            // MAXIMUM LIKELIHOOD ESTIMATION
            // Using Algorithmic Adjoint Differentiation (AAD)
            // from the `autodiff` module.
            LogisticRegressionAlgorithm::MLE => unimplemented!(),

            // ITERATIVELY RE-WEIGHTED LEAST SQUARES
            // References:
            //      - Elements of Statistical Learning (Hastie, Tibshirani, Friedman 2009)
            //      - Machine Learning: A Probabilistic Perspective (Murphy, Kevin P. 2012)
            LogisticRegressionAlgorithm::IRLS => {
                let mut eta: DVector<f64>;
                let mut mu: DVector<f64>;
                let mut W: DMatrix<f64>;

                // While not converged.
                // Convergence is defined as the norm of the change in
                // the weights being less than the tolerance.
                while (&coefs - &output.coefficients).norm() >= tolerance {
                    eta = &X * &output.coefficients;
                    mu = ActivationFunction::logistic(&eta);
                    W = DMatrix::from_diagonal(&mu.component_mul(&(&ones - &mu)));
                    let X_T_W = &X_T * &W;
                    let hessian = &X_T_W * &X;

                    // println!("W = {:.4}", W.norm());

                    // let working_response = match (W + &lambda).clone().try_inverse() {
                    let working_response = match W.clone().try_inverse() {
                        Some(inv) => eta + inv * (&y - &mu),
                        // None => return Err("Weights matrix (W) is singular (non-invertible)."),
                        None => {
                            // result.intercept = result.coefficients[0];
                            break;
                        }
                    };

                    coefs = match hessian.try_inverse() {
                        // Keep this bracketed to improve performance since
                        // `working_response` is a vector and not a matrix.
                        Some(inv) => inv * (&X_T_W * working_response),
                        None => {
                            return Err("Hessian matrix (X^T W X) is singular (non-invertible).")
                        }
                    };
                    output.iterations += 1;
                    // result.intercept = result.coefficients[0];

                    // println!("iter = {}", result.iterations);
                    println!("w_curr = {:.4}", output.coefficients);

                    std::mem::swap(&mut output.coefficients, &mut coefs);
                }
            }
        }

        Ok(output)
    }
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// UNIT TESTS
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[cfg(test)]
mod tests_logistic_regression {
    use super::*;
    use std::time::Instant;

    // use crate::assert_approx_equal;

    #[test]
    fn test_logistic_regression() {
        // PROFILE THIS UNIT TEST WITH (on MacOS):
        // sudo -E cargo flamegraph --release --freq 5000 --unit-test -- tests_logistic_regression::test_logistic_regression

        // TEST DATA GENERATED FROM THE FOLLOWING R v4.3.0 CODE:
        //
        // set.seed(1234)
        //
        // features    <- c("x1", "x2", "x3")
        //
        // (x_train  <- data.frame(matrix(rnorm(12), 4, 3))); colnames(x_train) <- features
        // (x_test   <- data.frame(matrix(rnorm(12), 4, 3))); colnames(x_test)  <- features
        //
        // (response <- sample(c(0,1), 4, replace = TRUE))
        //
        // (model <- glm(response ~ ., data = x_train, family = binomial))
        // (preds <- predict(model, newdata = x_test, type = "response"))

        #[rustfmt::skip]
        let x_train = DMatrix::from_row_slice(
            4, // rows
            3, // columns
            &[-1.2070657,  0.4291247, -0.5644520,
               0.2774292,  0.5060559, -0.8900378,
               1.0844412, -0.5747400, -0.4771927,
              -2.3456977, -0.5466319, -0.9983864],
        );

        #[rustfmt::skip]
        let _x_test = DMatrix::from_row_slice(
            4, // rows
            3, // columns
            &[-0.77625389, -0.5110095,  0.1340882,
               0.06445882, -0.9111954, -0.4906859,
               0.95949406, -0.8371717, -0.4405479,
              -0.11028549,  2.4158352,  0.4595894],
        );

        let response = DVector::from_row_slice(&[0., 1., 1., 1.]);

        // Fit the model to the training data.
        let input = LogisticRegressionInput {
            x: x_train,
            y: response,
        };

        let start_none = Instant::now();
        let output = input.fit(LogisticRegressionAlgorithm::IRLS, f64::EPSILON.sqrt());
        let elapsed_none = start_none.elapsed();

        match output {
            Ok(output) => {
                println!("Iterations: \t{}", output.iterations);
                println!("Time taken: \t{:?}", elapsed_none);
                // println!("Intercept: \t{:?}", output.intercept);
                println!("Coefficients: \t{:?}", output.coefficients);
            }
            Err(err) => {
                panic!("Failed to fit logistic regression model: {}", err);
            }
        }

        // // Predict the response for the test data.
        // let preds = output.predict(x_test);

        // // Check intercept.
        // assert_approx_equal!(output.intercept, 0.45326734, 1e-6);

        // // Check coefficients.
        // for (i, coefficient) in output.coefficients.iter().enumerate() {
        //     assert_approx_equal!(
        //         coefficient,
        //         &[0.45326734, 1.05986612, -0.16909348, 2.29605328][i],
        //         1e-6
        //     );
        // }

        // // Check predictions.
        // for (i, pred) in preds.iter().enumerate() {
        //     assert_approx_equal!(
        //         pred,
        //         &[0.0030197504, 0.4041016953, 2.4605541127, 1.6571889522][i],
        //         1e-3
        //     );
        // }
    }

    #[test]
    fn test_logistic_regression2() {
        // cargo test --release   tests_logistic_regression::test_logistic_regression2 -- --nocapture

        // The test generates sample data in the following way:
        // - For each of the N samples (train/test) draw K feature values each from a uniform distribution over (-1.,1.) and arrange as design matrix "X".
        // - For the coefficients of the generating distribution draw K values from surface of the unit sphere S_(K-1)  and a bias from uniform(-0.5,0.5); arrange as DVector "coefs"
        // - compute vector of probabilities(target=1) as sigmoid(X_ext * coefs)
        // - compute target values:for each sample i draw from Bernouilli(prob_i)

        use rand::prelude::*;
        use rand_distr::{Bernoulli, StandardNormal, Uniform};

        let N_train = 500; //Number of training samples
        let N_test = 80; //Number of test samples
        let K = 2; //Number of Features

        //generate random coefficients which will be used to generate target values for the x_i (direction uniform from sphere, bias uniform between -0.5 and 0.5 ) scaled by steepness
        let it_normal = rand::thread_rng().sample_iter(StandardNormal).take(K);
        let bias = rand::thread_rng().sample(Uniform::new(-0.5, 0.5));
        let steepness = rand::thread_rng().sample(Uniform::new(1., 5.));
        let coefs = DVector::<f64>::from_iterator(K, it_normal)
            .normalize()
            .insert_row(0, bias)
            .scale(steepness);

        //generate random design matrix for train/test
        let distr_uniform = Uniform::new(-1., 1.);
        let it_uniform_train = rand::thread_rng()
            .sample_iter(distr_uniform)
            .take(N_train * K);
        let x_train = DMatrix::<f64>::from_iterator(N_train, K, it_uniform_train);
        let it_uniform_test = rand::thread_rng()
            .sample_iter(distr_uniform)
            .take(N_test * K);
        let x_test = DMatrix::<f64>::from_iterator(N_test, K, it_uniform_test);

        //extend each feature vector by 1. so that coefs_train[0] acts as bias
        let x_train_extended = x_train.clone().insert_column(0, 1.0);
        let x_test_extended = x_test.clone().insert_column(0, 1.0);

        let eta_train = &x_train_extended * &coefs;
        let eta_test = &x_test_extended * &coefs;

        //compute probabilities for each sample x_i
        let probs_train = ActivationFunction::logistic(&eta_train);
        let probs_test = ActivationFunction::logistic(&eta_test);

        // sample from Bernoulli distribution with p=p_i for each sample i
        let y_train = probs_train
            .map(|p| Bernoulli::new(p).unwrap().sample(&mut rand::thread_rng()) as i32 as f64);
        let y_test = probs_test
            .map(|p| Bernoulli::new(p).unwrap().sample(&mut rand::thread_rng()) as i32 as f64);

        // Fit the model to the training data.
        let input = LogisticRegressionInput {
            x: x_train,
            y: y_train,
        };

        let start_none = Instant::now();
        let output = input.fit(LogisticRegressionAlgorithm::IRLS, f64::EPSILON.sqrt());
        let elapsed_none = start_none.elapsed();

        match output {
            Ok(output) => {
                let eta_hat = &x_test_extended * &output.coefficients;
                let y_hat =
                    ActivationFunction::logistic(&eta_hat).map(|p| if p > 0.5 { 1. } else { 0. });
                let missclassification_rate = (y_hat - y_test).abs().sum() / N_test as f64;
                println!(
                    "number of samples N_train={}, N_test={}, number of Features K={}",
                    N_train, N_test, K
                );
                println!(
                    "missclassification_rate(out of sample): \t{}",
                    missclassification_rate
                );
                println!("Iterations: \t{}", output.iterations);
                println!("Time taken: \t{:?}", elapsed_none);
                // println!("Intercept: \t{:?}", output.intercept);
                // print computed coeffs and original coeffs
                println!("Coefficients found by IRLS:\n{:?}", &output.coefficients);
                println!(
                    "Coefficients used for the generation of the training data:\n{:?}",
                    &coefs
                );
            }
            Err(err) => {
                panic!("Failed to fit logistic regression model: {}", err);
            }
        }
    }
}
