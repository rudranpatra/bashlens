(command
  name: (command_name (word) @outer)
  argument: (process_substitution
    (command name: (command_name (word) @inner))))
