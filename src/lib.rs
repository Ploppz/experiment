use std::fs;
use serde::{Serialize, de::DeserializeOwned, Deserialize};
use chrono::prelude::*;
use failure::Error;
use ndarray::{Array1, Array2};
use plotlib::{
    repr::{Line, LineStyle, Scatter},
    view::ContinuousView,
};


pub struct Page {
    pub plots: Vec<Plot>,
    config: ContinuousView,
    name: String,

}
impl Page {
    pub fn new<S: Into<String>>(name: S, config: ContinuousView) -> Self {
        Page {
            plots: Vec::new(),
            config,
            name: name.into()
        }
    }
    pub fn add_plot(mut self, plot: Plot) -> Self {
        self.plots.push(plot);
        self
    }


}

#[derive(Debug, Clone)]
pub enum Style {
    Points,
    Lines,
}

#[derive(Debug, Clone)]
pub struct Plot {
    x: Array1<f64>,
    y: Array1<f64>,
    legend: Option<String>,
    style: Style,
    width: f32,
    color: Option<String,>
}
impl Plot {
    pub fn scatter_plot(x: Array1<f64>, y: Array1<f64>) -> Plot {
        Plot {
            x,
            y,
            legend: None,
            style: Style::Points,
            width: 1.0,
            color: None,
        }
    }
    pub fn line_plot(x: Array1<f64>, y: Array1<f64>) -> Plot {
        Plot {
            x,
            y,
            legend: None,
            style: Style::Lines,
            width: 1.0,
            color: None,
        }
    }

    pub fn with_legend(mut self, legend: String) -> Self {
        self.legend = Some(legend);
        self
    }
    pub fn with_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
    pub fn with_color(mut self, color: String) -> Self {
        self.color = Some(color);
        self
    }
}

pub trait Experiment: Serialize + DeserializeOwned {

    /// Returns information which helps create a plot
    fn plot(experiments: &[Self]) -> Vec<Page>;

    // Merge several experiments into one, if applicable.
    // fn merge(experiments: &'a [Self]) -> Self;

    // fn params(&self) -> Vec<T>;

    /// Present writes plot images to path directory.
    /// It takes many rather than one experiment to open up for the possibility of for example
    /// plotting experiments in the same graph.
    /// `path` is the path to the directory.
    fn write_plot(experiments: &[Self], path: &str) -> Result<(), Error> {
        println!("Writing to {}", path);
        let colors = vec!["olivedrab", "lightcoral", "royalblue", "peru", "darkcyan", "saddlebrown", "darkmagenta"];
        use plotlib::repr::{Line, LineStyle};

        for page in Self::plot(experiments) {

            let mut view = page.config;

            for (i, plot) in page.plots.into_iter().enumerate() {
                // let data: Vec<_> = Iterator::zip(plot.x, plot.y).map(|(x,y)| (*x as f64, *y)).collect();
                let data: Vec<_> = Iterator::zip(plot.x.iter().cloned(), plot.y.iter().cloned()).collect();
                if let Style::Lines = plot.style {
                    let color = plot.color.unwrap_or(colors[i % colors.len()].into());
                    let mut line = Line::new(data).style(LineStyle::new().colour(color).width(plot.width));
                    if let Some(ref legend) = plot.legend {
                        line = line.legend(legend.clone());
                    }
                    view = view.add(Box::new(line))
                } else {
                    view = view.add(Box::new(Scatter::from_slice(&data)))
                }
            }

            println!(" - {}/{}.svg", path, page.name);
            plotlib::page::Page::single(&view)
                .save(&format!("{}/{}.svg", path, page.name))?;
        }
        Ok(())
    }
    // fn merge(&self, other: &Self) -> Self;

    /// Both saves and plots (calls `present`)
    fn save(experiments: Vec<Self>, path: &str) -> Result<(), Error> {
        let date = Local::now();
        let _ = fs::create_dir_all(path);
        Self::write_plot(&experiments, path)?;

        let mut out_file = fs::File::create(format!("{}/data.cbor", path))?;

        // bincode::serialize_into(out_file, &experiments)?;
        serde_cbor::ser::to_writer(&mut out_file, &experiments)?;

        Ok(())
    }
    /// `path` should be to a file `data.cbor`
    fn load(path: &str) -> Result<Vec<Self>, Error> {
        let file = fs::File::open(path)?;

        // Ok(bincode::deserialize_from(file)?)
        Ok(serde_cbor::de::from_reader(file)?)
    }
    /// Replot in a directory that already has a serialized experiment
    fn replot(path: &str) -> Result<(), Error> {
        let e = Self::load(&format!("{}/data.cbor", path))?;
        Self::write_plot(&e, path)?;
        Ok(())
    }

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
