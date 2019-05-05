pub mod par_map;
pub use crate::par_map::par_map;

#[cfg(test)]
mod tests
{
    use super::*;
    use test::Bencher;

    #[test]
    fn par_map()
    {
        use super::par_map;

        // solve ode by euler method for example
        fn difficult_computation(y1: i32) -> f32
        {
            let dy = |y: f32, t: f32| 3.0*t - y;
            let step = 0.01;
            let mut t = 0.0;
            let mut y = y1 as f32;

            for _ in 0..100
            {
                y += step*dy(y, t);
                t += step;
            }

            return y;
        }

        let ys: Vec<_> = par_map(0..1_000_000, difficult_computation).collect();
    }
}

