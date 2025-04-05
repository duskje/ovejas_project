from typing import Any, Optional
import sys
import os

debug = os.environ.get('OVEJAS_EXECUTOR_DEBUG')
debug = debug == 'true' if debug is not None else False

def hide_output(debug_mode_enabled: bool):
    if not debug_mode_enabled:
        def decorator(function):
            def wrapper(*args, **kwargs):
                stderr_tmp = sys.stderr
                stdout_tmp = sys.stdout

                sys.stdout = open(os.devnull, 'w')
                sys.stderr = open(os.devnull, 'w')

                try:
                    result = function(*args, **kwargs)
                except:
                    sys.stderr = stderr_tmp
                    sys.stdout = stdout_tmp

                    raise

                sys.stderr = stderr_tmp
                sys.stdout = stdout_tmp

                return result

            return wrapper
    else:
        def decorator(function):
            def wrapper(*args, **kwargs):
                return function(*args, **kwargs)

            return wrapper

    return decorator

@hide_output(debug_mode_enabled=debug)
def execute(main_path: str, lib_path: str):
    sys.path.append(lib_path)

    program = open(main_path, 'r').read()

    executor_context: dict[str, Any] = { 
        '__name__': '__main__',
    }

    exec("import ovejas; ovejas._EXECUTION_CONTEXT = 'CLI_TOOL';")
    exec(program, executor_context)

    resource_class = executor_context['ResourceRegistry']

    return resource_class.as_json()

if __name__ == '__main__':
    import os

    main_path = ''
    source_root = ''

    result = execute(main_path='/home/david/Projects/ovejas_project/python_example_project/main.py',
                     lib_path='/home/david/Projects/ovejas_project/python_example_project/')

    print(result)
