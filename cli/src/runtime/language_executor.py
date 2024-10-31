from typing import Any
import sys
import os

def hide_output(f):
    def wrapper(*args, **kwargs):
        stderr_tmp = sys.stderr
        stdout_tmp = sys.stdout

        sys.stdout = open(os.devnull, 'w')
        sys.stderr = open(os.devnull, 'w')

        try:
            result = f(*args, **kwargs)
        except:
            sys.stderr = stderr_tmp
            sys.stdout = stdout_tmp

            raise

        sys.stderr = stderr_tmp
        sys.stdout = stdout_tmp

        return result

    return wrapper


@hide_output
def execute(main_path: str, lib_path: str):
    sys.path.append(lib_path)

    program = open(main_path, 'r').read()

    executor_context: dict[str, Any] = { 
        '__name__': '__main__',
    }

    exec(program, executor_context)

    resource_class = executor_context['ResourceRegistry']

    return resource_class.as_json()

if __name__ == '__main__':
#    execute(command='up',
#            main_path='main.py',
#            lib_path='.')

    result = execute(main_path='/home/duskje/Projects/ovejas_project/python_example_project/main.py',
                     lib_path='/home/duskje/Projects/ovejas_project/python_example_project/')

    print(result)
