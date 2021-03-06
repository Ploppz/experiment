use std::fs;
use serde::{Serialize, de::DeserializeOwned, Deserialize};
use chrono::prelude::*;
use failure::Error;
use failure::{bail, format_err};
use ndarray::{Array1, Array2};
use plotlib::{
    repr,
    style::*,
    view::ContinuousView,
};

pub trait Experiment: Serialize + DeserializeOwned {

    /// Returns information which helps create a plot
    fn plot(&self, path: &str) -> Result<(), Error>;

    /*
    /// Present writes plot images to path directory.
    /// It takes many rather than one experiment to open up for the possibility of for example
    /// plotting experiments in the same graph.
    /// `path` is the path to the directory.
    fn write_plot(&self, path: &str) -> Result<(), Error> {
        println!("Working directory: {}", path);
        println!("Writing parameters to {}/params.txt", path);
        std::fs::write(format!("{}/params.txt", path), self.print_params())?;
        let colors = vec!["olivedrab", "lightcoral", "royalblue", "peru", "darkcyan", "saddlebrown", "darkmagenta"];

        for page in Self::plot(self) {

            let mut view = page.config;

            for (i, plot) in page.plots.into_iter().enumerate() {
                // let data: Vec<_> = Iterator::zip(plot.x, plot.y).map(|(x,y)| (*x as f64, *y)).collect();
                let data: Vec<_> = Iterator::zip(plot.x.iter().cloned(), plot.y.iter().cloned()).collect();
                if let Style::Lines = plot.style {
                    let color = plot.color.unwrap_or(colors[i % colors.len()].into());
                    let mut line = repr::Plot::new(data).line_style(LineStyle::new().colour(color).width(plot.width));
                    if let Some(ref legend) = plot.legend {
                        line = line.legend(legend.clone());
                    }
                    view = view.add(line)
                } else {
                    let mut scatter = repr::Plot::new(data).point_style(PointStyle::new());
                    view = view.add(scatter)
                }
            }

            println!(" - {}/{}.svg", path, page.name);
            match plotlib::page::Page::single(&view)
                    .dimensions(page.dimensions.0, page.dimensions.1)
                    .save(&format!("{}/{}.svg", path, page.name)) {
                Ok(()) => {},
                Err(e) => bail!(format_err!("Error in plot {}: {:?}", page.name, e)),
            }
        }
        Ok(())
    }
    */

    /// Both saves and plots (calls `present`)
    fn save(&self, path: &str) -> Result<(), Error> {
        let date = Local::now();
        let _ = fs::create_dir_all(path);

        // Serialize
        let mut out_file = fs::File::create(format!("{}/data.cbor", path))?;
        serde_cbor::ser::to_writer(&mut out_file, self)?;

        // Plot
        self.plot(path)?;

        Ok(())
    }
    /// `path` should be to a file `data.cbor`
    fn load(path: &str) -> Result<Self, Error> {
        let file = fs::File::open(path)?;

        // Ok(bincode::deserialize_from(file)?)
        Ok(serde_cbor::de::from_reader(file)?)
    }
    /// Replot in a directory that already has a serialized experiment
    fn replot(path: &str) -> Result<(), Error> {
        let e = Self::load(&format!("{}/data.cbor", path))?;
        e.plot(path)?;
        Ok(())
    }
    fn print_params(&self) -> String;

}

// TODO: params
/// Makes a path if the form <crate>/data/<name>/<date>
pub fn make_path(name: &str) -> String {
    let date = Local::now();
    format!("data/{}/{}", name, date.format("%Y.%m.%d-%Hh%M"))
}



/*
#[cfg(test)]
mod tests {
    impl crate::Present for i32 {
        fn present(&self, _: String) {}
    }
    #[test]
    fn it_works() {
        crate::save("name".to_string(), "".to_string(), 1 as i32, 1 as i32);
    }
}
*/
