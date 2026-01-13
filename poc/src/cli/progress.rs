use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::time::Duration;

pub struct CliProgressReporter {
    multi: MultiProgress,
    main_bar: ProgressBar,
    sub_bar: Option<ProgressBar>,
}

impl Default for CliProgressReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl CliProgressReporter {
    pub fn new() -> Self {
        let multi = MultiProgress::new();
        let main_bar = multi.add(ProgressBar::new_spinner());
        main_bar.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} [{elapsed_precise}] {msg}")
                .unwrap(),
        );
        main_bar.enable_steady_tick(Duration::from_millis(100));

        Self {
            multi,
            main_bar,
            sub_bar: None,
        }
    }

    pub fn set_stage(&mut self, stage: &str) {
        self.main_bar.set_message(stage.to_string());
        if let Some(bar) = self.sub_bar.take() {
            bar.finish_and_clear();
        }
    }

    pub fn start_progress(&mut self, total: u64) {
        let bar = self.multi.add(ProgressBar::new(total));
        bar.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
                )
                .unwrap()
                .progress_chars("#>-"),
        );
        bar.enable_steady_tick(Duration::from_millis(100));
        self.sub_bar = Some(bar);
    }

    pub fn update_progress(&self, pos: u64) {
        if let Some(bar) = &self.sub_bar {
            bar.set_position(pos);
        }
    }

    pub fn finish_stage(&mut self) {
        if let Some(bar) = self.sub_bar.take() {
            bar.finish_and_clear();
        }
    }

    pub fn finish_all(&self) {
        self.main_bar.finish_with_message("Done!");
    }
}
