import json
from typing import Mapping, Any

program = open('main.py', 'r').read()

executor_context: Mapping[str, Any] = { '__name__': 'executor_context' }

exec(program, executor_context)

resource_class = executor_context['Resource']

print(json.dumps(resource_class.get_all(), indent=2))
