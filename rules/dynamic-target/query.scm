(command
  name: (command_name (word) @cmd)
  argument: (simple_expansion) @dyn)

(command
  name: (command_name (word) @cmd)
  argument: (expansion) @dyn)

(command
  name: (command_name (word) @cmd)
  argument: (string (simple_expansion) @dyn))

(command
  name: (command_name (word) @cmd)
  argument: (string (expansion) @dyn))

(command
  name: (command_name (word) @cmd)
  argument: (string (command_substitution) @dyn))

(command
  name: (command_name (word) @cmd)
  argument: (command_substitution) @dyn)
