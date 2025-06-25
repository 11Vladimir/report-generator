mod monitoing_report;

use monitoing_report::WordReport;
use pyo3::prelude::*;

#[pymodule]
fn _report_generator(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<WordReport>()?;
    Ok(())
}
