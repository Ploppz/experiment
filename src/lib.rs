use std::fs;
use serde::{Serialize, de::DeserializeOwned, Deserialize};
use chrono::prelude::*;
use failure::Error;
use ndarray::{Array1, Array2};


pub struct Page<'a> {
    plots: Vec<Plot<'a>>,
    x_label: &'a str,
    y_label: &'a str,
    name: &'a str,

}
impl<'a> Page<'a> {
    pub fn new(name: &'a str, x_label: &'a str, y_label: &'a str) -> Self {
        Page {
            plots: Vec::new(),
            x_label,
            y_label,
            name
        }
    }
    pub fn add_plot(mut self, plot: Plot<'a>) -> Self {
        self.plots.push(plot);
        self
    }


}
pub struct Plot<'a> {
    x: Box<dyn Iterator<Item = f64> + 'a>,
    y: Box<dyn Iterator<Item = f64> + 'a>,
    legend: Option<String>,
}
impl<'a> Plot<'a> {
    pub fn new<I, J>(x: I, y: J) -> Plot<'a>
        where I: Iterator<Item = f64> + 'a,
              J: Iterator<Item = f64> + 'a
    {
        Plot {
            x: Box::new(x),
            y: Box::new(y),
            legend: None
        }
    }

    pub fn with_legend(mut self, legend: String) -> Self {
        self.legend = Some(legend);
        self
    }
}

pub trait Experiment: Serialize + DeserializeOwned {

    /// Returns information which helps create a plot
    fn plot<'a>(experiments: &'a [Self]) -> Vec<Page<'a>>;

    /// Present writes plot images to path directory.
    /// It takes many rather than one experiment to open up for the possibility of for example
    /// plotting experiments in the same graph.
    /// `path` is the path to the directory.
    fn write_plot<'a>(experiments: &'a [Self], path: &str) -> Result<(), Error> {
        const PRINT: bool = true;
        let colors = vec!["olivedrab", "lightcoral", "royalblue", "peru", "darkcyan", "saddlebrown", "darkmagenta"];
        use plotlib::repr::{Line, LineStyle};

        for mut page in Self::plot(experiments) {

            let mut view = plotlib::view::ContinuousView::new() // TODO maybe put this in `Page`
                .x_label(page.x_label)
                .y_label(page.y_label)
                .y_range(0.0, 0.06)
                .y_max_ticks(16);

            for (i, plot) in page.plots.iter_mut().enumerate() {
                // let data: Vec<_> = Iterator::zip(plot.x, plot.y).map(|(x,y)| (*x as f64, *y)).collect();
                let data: Vec<_> = plot.x.by_ref().zip(plot.y.by_ref()).collect();
                let mut line = Line::new(data.clone())
                    .style(LineStyle::new().colour(colors[i % colors.len()]));
                if let Some(ref legend) = plot.legend {
                    line = line.legend(legend.clone());
                }
                view = view.add(Box::new(line));
                if PRINT {
                    println!("= data =");
                    for (x, y) in data {
                        println!("  {}, {}", x, y);
                    }
                }
            }

            plotlib::page::Page::single(&view)
                .save(&format!("{}/{}.svg", path, page.name))?;
        }
        Ok(())
    }

    /// Both saves and plots (calls `present`)
    fn save(experiments: &[Self], path: &str) -> Result<(), Error>
    {
        let date = Local::now();
        let _ = fs::create_dir_all(path);
        Self::write_plot(experiments, path)?;
        let out_file = fs::File::create(format!("{}/data.bincode", path))?;
        bincode::serialize_into(out_file, experiments)?;
        Ok(())
    }
    /// `path` should be to a file `data.bincode`
    fn load(path: &str) -> Result<Vec<Self>, Error> {
        let file = fs::File::open(path)?;
        Ok(bincode::deserialize_from(file)?)
    }
    /// Replot in a directory that already has a serialized experiment
    fn replot(path: &str) -> Result<(), Error> {
        let e = Self::load(&format!("{}/data.bincode", path))?;
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
