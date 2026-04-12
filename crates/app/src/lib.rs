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
        let region: String = config
            .get_item("region")?
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("'region' is required"))?
            .extract()?;
        let universe: String = config
            .get_item("universe")?
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("'universe' is required"))?
            .extract()?;
        future_into_py(py, async move {
            let mut settings = Settings::default();
            settings.region = region;
            settings.universe = universe;
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
