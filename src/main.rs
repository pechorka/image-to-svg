use image::{GenericImageView, ImageBuffer, ImageReader};
use std::env;
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

type Pixel = image::Rgba<u8>;
type Pixels = Vec<Pixel>;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    args.next().expect("should always have program name");
    let image_path = args.next().expect("expect image path");
    let k: usize = args
        .next()
        .expect("should provide k")
        .parse()
        .expect("k should be usize");
    let image = ImageReader::open(image_path)?.decode()?;

    let pixels: Pixels = image
        .pixels()
        .map(|loc_and_pixel| loc_and_pixel.2)
        .collect();

    let reduced_pixels = reduce_colors(&pixels, k, 10);

    let mut out_img = ImageBuffer::new(image.width(), image.height());

    image
        .pixels()
        .zip(reduced_pixels)
        .for_each(|(loc, pixel)| out_img.put_pixel(loc.0, loc.1, pixel));

    out_img.save("output.png").expect("failed to save image");

    Ok(())
}

fn reduce_colors(pixels: &Pixels, k: usize, max_iterations: usize) -> Pixels {
    let mut centers = select_centers(pixels, k);

    for iter in 0..max_iterations {
        println!("[INFO] {} iteration", iter + 1);
        let mut clusters: Vec<Pixels> = Vec::with_capacity(k);
        for _ in 0..k {
            clusters.push(Vec::with_capacity(pixels.len() / k));
        }

        for pixel in pixels.iter() {
            let closest_center = select_closest_center(&centers, pixel);
            clusters[closest_center].push(*pixel);
        }

        let mut new_centers = Vec::with_capacity(k);
        for cluster in clusters.iter() {
            let mean = cluster_mean(cluster);
            new_centers.push(mean);
        }

        // TODO: check if new_centers are very close already and don't require additional iterations
        centers = new_centers;
    }

    let mut reduced_colors = Vec::with_capacity(pixels.len());
    for pixel in pixels.iter() {
        let closest_center = select_closest_center(&centers, pixel);
        reduced_colors.push(centers[closest_center]);
    }

    reduced_colors
}

fn select_centers(pixels: &Pixels, k: usize) -> Pixels {
    let mut centers = Vec::with_capacity(k);
    let mut rng = Rng::new(get_unix_timestamp());
    for _ in 0..k {
        let random_pixel_index = rng.next() as usize % pixels.len();
        let pixel = pixels[random_pixel_index];
        centers.push(pixel);
    }

    centers
}

fn select_closest_center(centers: &Pixels, pixel: &Pixel) -> usize {
    centers
        .iter()
        .enumerate()
        .min_by_key(|(_, c)| distance(pixel, c))
        .expect("centers should't be empty")
        .0
}

fn distance(pixel: &Pixel, center: &Pixel) -> usize {
    let rsq = ((pixel[0].abs_diff(center[0])) as usize).pow(2);
    let gsq = ((pixel[1].abs_diff(center[1])) as usize).pow(2);
    let bsq = ((pixel[2].abs_diff(center[2])) as usize).pow(2);
    (rsq + gsq + bsq).isqrt()
}

fn cluster_mean(cluster: &Pixels) -> Pixel {
    let (rs, gs, bs, alphas) = cluster.iter().fold(
        (0usize, 0usize, 0usize, 0usize),
        |(acc_r, acc_g, acc_b, acc_a), rgba| {
            (
                acc_r + rgba[0] as usize,
                acc_g + rgba[1] as usize,
                acc_b + rgba[2] as usize,
                acc_a + rgba[3] as usize,
            )
        },
    );

    let n = cluster.len();
    image::Rgba([
        (rs / n) as u8,
        (gs / n) as u8,
        (bs / n) as u8,
        (alphas / n) as u8,
    ])
}

fn get_unix_timestamp() -> u64 {
    let start = SystemTime::now();
    let since_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards?");

    since_epoch.as_secs()
}

struct Rng {
    seed: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Rng { seed }
    }

    fn next(&mut self) -> u64 {
        // Constants for the LCG (these are common values)
        const A: u64 = 6364136223846793005;
        const C: u64 = 1442695040888963407;
        const M: u64 = 2u64.pow(63);

        // Update the seed using the LCG formula
        self.seed = (A.wrapping_mul(self.seed).wrapping_add(C)) % M;

        self.seed
    }
}
