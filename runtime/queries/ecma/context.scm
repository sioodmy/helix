(arrow_function
	parameters: (_) @context.params
) @context

(function_declaration
	parameters: (_) @context.params
) @context

(function
	parameters: (_) @context.params
) @context

(expression_statement
		parameters: (_) @context.params
) @context

[
	(call_expression)
	(class_declaration)
  (else_clause)
  (for_statement)
  ; (interface_declaration) ; not usable in javascript
  (lexical_declaration)
  (method_definition)
  (object)
  (pair)
  (while_statement)
] @context

