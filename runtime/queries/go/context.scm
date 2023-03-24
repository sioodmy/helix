; Credits to nvim-treesitter/nvim-treesitter-context

(function_declaration
	body: (_) @context.end
) @context

(method_declaration
	body: (_) @context.end
) @context

([
  (const_declaration)
	(for_statement)
	(if_statement)
  (import_declaration)
  (type_declaration)
  (var_declaration)
] @context)
