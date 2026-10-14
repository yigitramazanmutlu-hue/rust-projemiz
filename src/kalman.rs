use nalgebra::{SMatrix, SVector};

/// Innovation covariance was not positive definite, so the gain was not applied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SingularInnovation;

macro_rules! impl_cv_kalman {
    ($name:ident, $d:expr, $n:expr) => {
        /// Constant-velocity linear Kalman filter.
        ///
        /// State layout: `[p_0 .. p_{D-1}, v_0 .. v_{D-1}]`, `f64`.
        /// `A` and `Q` are rebuilt only when `dt` changes.
        #[derive(Clone, Debug)]
        pub struct $name {
            x: SVector<f64, $n>,
            p: SMatrix<f64, $n, $n>,
            a: SMatrix<f64, $n, $n>,
            h: SMatrix<f64, $d, $n>,
            q: SMatrix<f64, $n, $n>,
            r: SMatrix<f64, $d, $d>,
            dt: f64,
            /// Acceleration variance q = sigma_a^2 (units^2 / s^4).
            accel_var: f64,
        }

        impl $name {
            /// `accel_std` is sigma_a (units/s^2). `meas_std` is the per-axis
            /// position noise sigma. Initial `P` is diagonal and large relative
            /// to `R`, so the first updates lock onto the measurement.
            pub fn new(dt: f64, accel_std: f64, meas_std: f64) -> Self {
                assert!(dt > 0.0, "dt must be positive");
                assert!(accel_std >= 0.0, "accel_std must be non-negative");
                assert!(meas_std > 0.0, "meas_std must be positive");

                let meas_var = meas_std * meas_std;
                let mut filter = Self {
                    x: SVector::zeros(),
                    p: SMatrix::zeros(),
                    a: SMatrix::identity(),
                    h: SMatrix::zeros(),
                    q: SMatrix::zeros(),
                    r: SMatrix::from_diagonal_element(meas_var),
                    dt,
                    accel_var: accel_std * accel_std,
                };
                for i in 0..$d {
                    filter.h[(i, i)] = 1.0;
                    filter.p[(i, i)] = meas_var * 100.0;
                    filter.p[($d + i, $d + i)] = (accel_std.max(1.0) * 10.0).powi(2);
                }
                filter.rebuild_transition();
                filter
            }

            #[inline]
            pub fn state(&self) -> &SVector<f64, $n> {
                &self.x
            }

            #[inline]
            pub fn covariance(&self) -> &SMatrix<f64, $n, $n> {
                &self.p
            }

            #[inline]
            pub fn position(&self) -> SVector<f64, $d> {
                self.x.fixed_rows::<$d>(0).into()
            }

            #[inline]
            pub fn velocity(&self) -> SVector<f64, $d> {
                self.x.fixed_rows::<$d>($d).into()
            }

            #[inline]
            pub fn dt(&self) -> f64 {
                self.dt
            }

            /// Replace the posterior. `P` is symmetrized.
            pub fn set_posterior(
                &mut self,
                state: SVector<f64, $n>,
                covariance: SMatrix<f64, $n, $n>,
            ) {
                self.x = state;
                self.p = symmetrize(covariance);
            }

            pub fn set_measurement_noise(&mut self, r: SMatrix<f64, $d, $d>) {
                self.r = symmetrize(r);
            }

            /// Fixed-rate path: matrices stay cached.
            #[inline]
            pub fn set_dt(&mut self, dt: f64) {
                debug_assert!(dt > 0.0);
                if (dt - self.dt).abs() > f64::EPSILON * dt.max(self.dt) {
                    self.dt = dt;
                    self.rebuild_transition();
                }
            }

            /// x = A x,  P = A P A^T + Q
            #[inline]
            pub fn predict(&mut self) {
                self.x = self.a * self.x;
                self.p = symmetrize(self.a * self.p * self.a.transpose() + self.q);
            }

            /// Joseph update. On a singular `S` the state is left unchanged.
            #[inline]
            pub fn update(&mut self, z: &SVector<f64, $d>) -> Result<(), SingularInnovation> {
                let innovation = z - self.h * self.x;
                let pht = self.p * self.h.transpose();
                let s = self.h * pht + self.r;
                let Some(chol) = s.cholesky() else {
                    return Err(SingularInnovation);
                };
                // S K^T = (P H^T)^T  =>  K = P H^T S^{-1}
                let k = chol.solve(&pht.transpose()).transpose();
                self.x += k * innovation;

                let ikh = SMatrix::<f64, $n, $n>::identity() - k * self.h;
                self.p = symmetrize(ikh * self.p * ikh.transpose() + k * self.r * k.transpose());
                Ok(())
            }

            /// One live sample at a (possibly new) period.
            #[inline]
            pub fn step(
                &mut self,
                z: &SVector<f64, $d>,
                dt: f64,
            ) -> Result<(), SingularInnovation> {
                self.set_dt(dt);
                self.predict();
                self.update(z)
            }

            fn rebuild_transition(&mut self) {
                let dt = self.dt;
                let dt2 = dt * dt;
                let dt3 = dt2 * dt;
                let dt4 = dt2 * dt2;
                let q = self.accel_var;

                self.a = SMatrix::identity();
                self.q = SMatrix::zeros();
                for i in 0..$d {
                    self.a[(i, $d + i)] = dt;
                    let pos = q * dt4 * 0.25;
                    let cross = q * dt3 * 0.5;
                    let vel = q * dt2;
                    self.q[(i, i)] = pos;
                    self.q[(i, $d + i)] = cross;
                    self.q[($d + i, i)] = cross;
                    self.q[($d + i, $d + i)] = vel;
                }
            }
        }
    };
}

impl_cv_kalman!(KalmanFilter2d, 2, 4);
impl_cv_kalman!(KalmanFilter3d, 3, 6);

#[inline]
fn symmetrize<const N: usize>(m: SMatrix<f64, N, N>) -> SMatrix<f64, N, N> {
    (m + m.transpose()) * 0.5
}
