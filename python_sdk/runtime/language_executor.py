from typing import Any
import sys
import os

def execute(command: str, main_path: str, lib_path: str):
    sys.path.append(lib_path)

    os.environ['__command'] = command

    program = open(main_path, 'r').read()

    executor_context: dict[str, Any] = { 
        '__name__': 'executor_context',
    }

    exec(program, executor_context)

    resource_class = executor_context['ResourceRegistry']

    print(resource_class.as_json())

if __name__ == '__main__':
    execute(command='up',
            main_path='main.py',
            lib_path='.')
