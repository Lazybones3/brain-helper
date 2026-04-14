use std::sync::Arc;

use brain_api::Settings;
use pyo3::{
    prelude::*,
    types::{PyDict, PyList},
};
use pyo3_async_runtimes::tokio::future_into_py;
use pythonize::{depythonize, pythonize};

use crate::app::BrainAppCore;
use brain_common::logger;

mod app;

#[pyclass]
struct BrainApp {
    core: Arc<BrainAppCore>,
}

#[pymethods]
impl BrainApp {
    #[staticmethod]
    pub fn new<'p>(py: Python<'p>, config: &Bound<'_, PyDict>) -> PyResult<Bound<'p, PyAny>> {
        logger::init();
        let instrument_type: String = config
            .get_item("instrumentType")?
            .ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyKeyError, _>("'instrumentType' is required")
            })?
            .extract()?;
        let region: String = config
            .get_item("region")?
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("'region' is required"))?
            .extract()?;
        let universe: String = config
            .get_item("universe")?
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("'universe' is required"))?
            .extract()?;
        let delay: i32 = config
            .get_item("delay")?
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("'delay' is required"))?
            .extract()?;
        let decay: i32 = config
            .get_item("decay")?
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("'decay' is required"))?
            .extract()?;
        let neutralization: String = config
            .get_item("neutralization")?
            .ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyKeyError, _>("'neutralization' is required")
            })?
            .extract()?;
        let truncation: f64 = config
            .get_item("truncation")?
            .ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyKeyError, _>("'truncation' is required")
            })?
            .extract()?;
        let pasteurization: String = config
            .get_item("pasteurization")?
            .ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyKeyError, _>("'pasteurization' is required")
            })?
            .extract()?;
        let test_period: String = config
            .get_item("testPeriod")?
            .ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyKeyError, _>("'testPeriod' is required")
            })?
            .extract()?;
        let unit_handling: String = config
            .get_item("unitHandling")?
            .ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyKeyError, _>("'unitHandling' is required")
            })?
            .extract()?;
        let nan_handling: String = config
            .get_item("nanHandling")?
            .ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyKeyError, _>("'nanHandling' is required")
            })?
            .extract()?;
        let max_trade: String = config
            .get_item("maxTrade")?
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("'maxTrade' is required"))?
            .extract()?;
        let language: String = config
            .get_item("language")?
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("'language' is required"))?
            .extract()?;
        let visualization: bool = config
            .get_item("visualization")?
            .ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyKeyError, _>("'visualization' is required")
            })?
            .extract()?;
        future_into_py(py, async move {
            let settings = Settings {
                instrument_type,
                region,
                universe,
                delay,
                decay,
                neutralization,
                truncation,
                pasteurization,
                language,
                nan_handling,
                unit_handling,
                max_trade,
                visualization,
                test_period,
                selection_handling: None,
                selection_limit: None,
            };
            let core = BrainAppCore::new(settings)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            Ok(BrainApp {
                core: Arc::new(core),
            })
        })
    }

    pub fn get_fields_by_dataset<'p>(
        &self,
        py: Python<'p>,
        dataset_name: String,
    ) -> PyResult<Bound<'p, PyAny>> {
        let core_arc = self.core.clone();
        future_into_py(py, async move {
            let datafields = core_arc
                .get_fields_by_dataset(&dataset_name)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Python::attach(|py| {
                let result = pythonize(py, &datafields).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
                })?;
                Ok(result.unbind())
            })
        })
    }

    pub fn simulation<'p>(
        &self,
        py: Python<'p>,
        dataset_name: String,
        alphas: &Bound<'_, PyList>,
    ) -> PyResult<Bound<'p, PyAny>> {
        let alpha_list = depythonize(&alphas)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        let core_arc = self.core.clone();
        future_into_py(py, async move {
            core_arc
                .simulation(&dataset_name, alpha_list)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }
}

#[pymodule]
mod brain_helper {
    #[pymodule_export]
    use super::BrainApp;
}
