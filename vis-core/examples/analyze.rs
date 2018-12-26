extern crate log;
extern crate vis_core;

#[derive(Debug, Default)]
pub struct AnalyzerResult {
    spectrum: vis_core::analyzer::Spectrum<Vec<f32>>,
    volume: f32,
    beat: f32,
}

fn main() {
    vis_core::default_log();
    vis_core::default_config();

    let mut analyzer = vis_core::analyzer::FourierBuilder::new()
        .length(512)
        .window(vis_core::analyzer::window::nuttall)
        .plan();

    let mut spectra = [
        vis_core::analyzer::Spectrum::new(vec![0.0; analyzer.buckets], 0.0, 1.0),
        vis_core::analyzer::Spectrum::new(vec![0.0; analyzer.buckets], 0.0, 1.0),
    ];

    let spectrum = vis_core::analyzer::Spectrum::new(vec![0.0; analyzer.buckets], 0.0, 1.0);

    let mut frames = vis_core::Visualizer::new(
        AnalyzerResult {
            spectrum,
            ..Default::default()
        },
        move |info, samples| {
            vis_core::analyzer::average_spectrum(
                &mut info.spectrum,
                analyzer.analyze(samples, &mut spectra),
            );

            info.volume = samples.volume(0.3);
            info.beat = info.spectrum.slice(50.0, 100.0).max();
            info
        },
    )
    .recorder(
        vis_core::recorder::pulse::PulseBuilder::new()
            .rate(8000)
            .read_size(64)
            .buffer_size(16000)
            .build(),
    )
    .frames();

    for frame in frames.iter() {
        frame.lock_info(|info| {
            for _ in 0..(0.01 * info.beat) as usize {
                print!("#");
            }
            println!("");
        });
        std::thread::sleep_ms(30);
    }
}
