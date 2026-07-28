(command
  name: (command_name (word) @outer)
  argument: (string (command_substitution (command name: (command_name (word) @inner)))))

(command
  name: (command_name (word) @outer)
  argument: (command_substitution (command name: (command_name (word) @inner))))
