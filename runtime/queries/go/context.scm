; Credits to nvim-treesitter/nvim-treesitter-context

(function_declaration
	result: (_) @context.end
) @context

(method_declaration
	result: (_) @context.end
) @context

[
  (const_declaration)
  (if_statement)
  (import_declaration)
  (type_declaration)
  (var_declaration)
] @context
