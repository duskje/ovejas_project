use pyo3::prelude::*;

pub fn python_executor(execution_path: String) ->PyResult<String> {
    // Include in binary
    let python_language_executor = include_str!("runtime/language_executor.py");

    pyo3::prepare_freethreaded_python();

    Python::with_gil(|py| {
        let fun = PyModule::from_code_bound(
            py,
            python_language_executor,
            "",
            "",
        )?
        .getattr("execute")?;

        // Set executor arguments
        let args = (format!("{execution_path}/main.py"),
                    execution_path);

        let json = fun.call1(args)?;
        let json = json.extract::<String>()?;

        Ok(json)
    })
}
